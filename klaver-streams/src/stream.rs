use futures::stream::{BoxStream, LocalBoxStream};
use klaver::{
    shared::{
        iter::{AsyncIter, AsyncIterable},
        Static,
    },
    throw,
};
use rquickjs::{
    class::Trace, function::Opt, CatchResultExt, CaughtError, Class, Ctx, FromJs, Function, IntoJs,
    Object, Value,
};
use std::{collections::VecDeque, rc::Rc};
use tokio::sync::Notify;

#[rquickjs::class]
pub struct ReadableStream<'js> {
    v: ReadableStreamInit<'js>,
    ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
}

impl<'js> Trace<'js> for ReadableStream<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.v.trace(tracer);
        self.ctrl.trace(tracer);
    }
}

#[derive(Debug, Clone, Trace)]
pub struct ReadableStreamInit<'js> {
    start: Option<Function<'js>>,
    pull: Option<Function<'js>>,
    cancel: Option<Function<'js>>,
    highwater_mark: u32,
}

impl<'js> FromJs<'js> for ReadableStreamInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(ReadableStreamInit {
            start: obj.get("start")?,
            pull: obj.get("pull")?,
            cancel: obj.get("cancel")?,
            highwater_mark: obj.get::<_, Option<u32>>("highwaterMark")?.unwrap_or(1),
        })
    }
}

macro_rules! catch {
    ($ctrl: expr, $ctx: expr, $ret: expr) => {
        match $ret.catch(&$ctx) {
            Ok(ret) => ret,
            Err(err) => {
                $ctrl.borrow_mut().state = State::Error(err);
                return;
            }
        }
    };
}

macro_rules! call {
    ($ctrl: expr, $ctx: expr, $func: expr) => {
        catch!(
            $ctrl,
            $ctx,
            $func.call::<_, rquickjs::Value>(($ctrl.clone(),))
        )
    };
}

impl<'js> ReadableStream<'js> {
    pub fn to_stream(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<LocalBoxStream<'js, rquickjs::Result<Value<'js>>>> {
        let reader = self.get_reader(ctx.clone())?;

        let stream = async_stream::try_stream! {
            loop {
                let next = reader.read(ctx.clone()).await?;

                if  let Some(value) = next.value {
                    yield value
                } else {
                    break;
                }

            }
        };
        Ok(Box::pin(stream))
    }
}

impl<'js> AsyncIterable<'js> for ReadableStream<'js> {
    type Item = Value<'js>;

    type Error = rquickjs::Error;

    type Stream = Static<LocalBoxStream<'js, Result<Self::Item, Self::Error>>>;

    fn stream(&mut self, ctx: &Ctx<'js>) -> klaver::shared::iter::AsyncIter<Self::Stream> {
        AsyncIter::new(Static(self.to_stream(ctx.clone()).unwrap()))
    }
}

#[rquickjs::methods]
impl<'js> ReadableStream<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        options: ReadableStreamInit<'js>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        let notify = Rc::new(Notify::new());
        let ready = Rc::new(Notify::new());

        let controller = Class::instance(
            ctx.clone(),
            ReadableStreamDefaultController {
                queue: Default::default(),
                ready: ready.clone(),
                wait: notify.clone(),
                highwater_mark: options.highwater_mark,
                locked: false,
                state: State::Running,
            },
        )?;

        let class = Class::instance(
            ctx.clone(),
            ReadableStream {
                v: options,
                ctrl: controller,
            },
        )?;

        let class_clone = class.clone();

        let ctx_clone = ctx.clone();

        ctx.spawn(async move {
            let ctrl = class_clone.borrow().ctrl.clone();
            if let Some(func) = &class_clone.borrow().v.start.clone() {
                let called = call!(ctrl, ctx_clone, func);

                if let Some(promise) = called.into_promise() {
                    catch!(ctrl, ctx_clone, promise.into_future::<()>().await);
                }
            }

            if let Some(func) = class_clone.borrow().v.pull.clone() {
                loop {
                    if ctrl.borrow().is_filled() {
                        ready.notified().await;
                    }

                    if !ctrl.borrow().state.is_running() {
                        break;
                    }

                    let called = call!(ctrl, ctx_clone, func);
                    if let Some(promise) = called.into_promise() {
                        catch!(ctrl, ctx_clone, promise.into_future::<()>().await);
                    }

                    if !ctrl.borrow().state.is_running() {
                        ctrl.borrow().wait.notify_waiters();
                        break;
                    }
                }
            }
        });

        Ok(class)
    }

    #[qjs(rename = "getReader")]
    pub fn get_reader(&self, ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        if self.ctrl.borrow().locked {
            throw!(ctx, "Readable stream already locked")
        }

        self.ctrl.borrow_mut().locked = true;

        Ok(ReadableStreamDefaultReader {
            ctrl: self.ctrl.clone(),
        })
    }
}

enum State<'js> {
    Done,
    Running,
    Canceled(Option<rquickjs::String<'js>>),
    Error(CaughtError<'js>),
}

impl<'js> State<'js> {
    fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    fn as_error(&self) -> Option<&CaughtError<'js>> {
        match self {
            Self::Error(err) => Some(err),
            _ => None,
        }
    }
}

impl<'js> Trace<'js> for State<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Canceled(reason) => reason.trace(tracer),
            Self::Error(err) => match err {
                CaughtError::Exception(err) => err.trace(tracer),
                CaughtError::Value(v) => v.trace(tracer),
                _ => {}
            },
            _ => {}
        }
    }
}

#[rquickjs::class]
pub struct ReadableStreamDefaultController<'js> {
    queue: VecDeque<Value<'js>>,
    highwater_mark: u32,
    locked: bool,
    ready: Rc<Notify>,
    wait: Rc<Notify>,
    state: State<'js>,
}

impl<'js> ReadableStreamDefaultController<'js> {
    pub fn is_filled(&self) -> bool {
        self.queue.len() > self.highwater_mark as usize
    }
}

impl<'js> Trace<'js> for ReadableStreamDefaultController<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.queue.trace(tracer);
        self.state.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultController<'js> {
    pub fn enqueue(&mut self, chunk: Value<'js>) -> rquickjs::Result<()> {
        self.queue.push_back(chunk);
        self.wait.notify_one();
        Ok(())
    }

    pub fn close(&mut self) -> rquickjs::Result<()> {
        if !self.state.is_running() {
            return Ok(());
        }
        self.state = State::Done;
        Ok(())
    }
}

#[derive(Trace)]
#[rquickjs::class]
pub struct ReadableStreamDefaultReader<'js> {
    ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
}

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultReader<'js> {
    pub async fn read(&self, ctx: Ctx<'js>) -> rquickjs::Result<Chunk<'js>> {
        if let Some(err) = self.ctrl.borrow().state.as_error() {
            match err {
                CaughtError::Error(err) => {
                    return Err(ctx
                        .clone()
                        .throw(rquickjs::String::from_str(ctx, &err.to_string())?.into_value()))
                }
                CaughtError::Exception(err) => {
                    return Err(ctx.throw(Value::from_exception(err.clone())))
                }
                CaughtError::Value(value) => return Err(ctx.throw(value.clone())),
            }
        } else if !self.ctrl.borrow().state.is_running() {
            return Ok(Chunk {
                value: None,
                done: true,
            });
        }

        // Wait for new items
        if self.ctrl.borrow().queue.is_empty() {
            let waiter = self.ctrl.borrow().wait.clone();
            waiter.notified().await;
        }

        let ret = self.ctrl.borrow_mut().queue.pop_front();

        if !self.ctrl.borrow().is_filled() {
            self.ctrl.borrow().ready.notify_one();
        }

        Ok(Chunk {
            value: ret,
            done: false,
        })
    }

    pub async fn cancel(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<rquickjs::String<'js>>,
    ) -> rquickjs::Result<()> {
        if !self.ctrl.borrow().state.is_running() {
            throw!(ctx, "stream not running");
        }
        self.ctrl.borrow_mut().state = State::Canceled(reason.0);
        self.ctrl.borrow().ready.notify_one();
        Ok(())
    }
}

#[derive(Trace)]
pub struct Chunk<'js> {
    value: Option<Value<'js>>,
    done: bool,
}

impl<'js> IntoJs<'js> for Chunk<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        obj.set("value", self.value)?;
        obj.set("done", self.done)?;

        Ok(obj.into_value())
    }
}
