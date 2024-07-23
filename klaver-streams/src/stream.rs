use klaver::throw;
use rquickjs::{class::Trace, CatchResultExt, Class, Ctx, FromJs, Function, IntoJs, Object, Value};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Notify;

#[rquickjs::class]
pub struct ReadableStream<'js> {
    v: ReadableStreamInit<'js>,
    ctrl: Class<'js, Controller<'js>>,
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

#[rquickjs::methods]
impl<'js> ReadableStream<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        options: ReadableStreamInit<'js>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        // let (sx_ready, mut rx_ready) = futures::channel::mpsc::channel(1);
        // let (sx_wait, rx_wait) = futures::channel::mpsc::channel(1);

        let notify = Arc::new(Notify::new());
        let ready = Arc::new(Notify::new());

        let controller = Class::instance(
            ctx.clone(),
            Controller {
                queue: Default::default(),
                ready: ready.clone(),
                wait: notify.clone(),
                highwater_mark: options.highwater_mark,
                locked: false,
                done: false,
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

        let (sx, rx) = futures::channel::oneshot::channel::<()>();

        ctx.spawn(async move {
            //
            let ctrl = class_clone.borrow().ctrl.clone();
            if let Some(func) = &class_clone.borrow().v.start {
                let called = func.call::<_, rquickjs::Value>((ctrl.clone(),)).unwrap();
                if let Some(promise) = called.into_promise() {
                    promise.into_future::<()>().await.unwrap();
                }
            }

            if let Some(func) = class_clone.borrow().v.pull.clone() {
                loop {
                    if ctrl.borrow().is_filled() {
                        ready.notified().await;
                    }

                    let called = func.call::<_, rquickjs::Value>((ctrl.clone(),)).unwrap();

                    if let Some(promise) = called.into_promise() {
                        promise.into_future::<()>().await.catch(&ctx_clone).unwrap();
                    }

                    if ctrl.borrow().done {
                        ctrl.borrow().wait.notify_waiters();
                        break;
                    }
                }
            }
        });

        Ok(class)
    }

    #[qjs(rename = "getReader")]
    pub fn get_reader(&self, ctx: Ctx<'js>) -> rquickjs::Result<Reader<'js>> {
        if self.ctrl.borrow().locked {
            throw!(ctx, "Readable stream already locked")
        }

        self.ctrl.borrow_mut().locked = true;

        Ok(Reader {
            ctrl: self.ctrl.clone(),
        })
    }
}

#[rquickjs::class]
pub struct Controller<'js> {
    queue: VecDeque<Value<'js>>,
    highwater_mark: u32,
    locked: bool,
    ready: Arc<Notify>,
    wait: Arc<Notify>,
    done: bool,
}

impl<'js> Controller<'js> {
    pub fn is_filled(&self) -> bool {
        self.queue.len() > self.highwater_mark as usize
    }
}

impl<'js> Trace<'js> for Controller<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.queue.trace(tracer)
    }
}

#[rquickjs::methods]
impl<'js> Controller<'js> {
    pub fn enqueue(&mut self, chunk: Value<'js>) -> rquickjs::Result<()> {
        self.queue.push_back(chunk);
        self.wait.notify_one();
        Ok(())
    }

    pub fn close(&mut self) -> rquickjs::Result<()> {
        self.done = true;
        Ok(())
    }
}

#[derive(Trace)]
#[rquickjs::class]
pub struct Reader<'js> {
    ctrl: Class<'js, Controller<'js>>,
}

#[rquickjs::methods]
impl<'js> Reader<'js> {
    pub async fn read(&self) -> rquickjs::Result<Chunk<'js>> {
        if self.ctrl.borrow().done {
            return Ok(Chunk {
                value: None,
                done: true,
            });
        }

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

    pub async fn cancel(&self, reason: Option<String>) -> rquickjs::Result<()> {
        self.ctrl.borrow_mut().done = true;
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
