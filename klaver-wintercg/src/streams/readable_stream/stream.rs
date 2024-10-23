use futures::{stream::LocalBoxStream, TryStream};
use klaver::shared::{
    buffer::Buffer,
    iter::{AsyncIter, AsyncIterable, StreamContainer},
    Static,
};
use rquickjs::{class::Trace, prelude::Opt, Class, Ctx, FromJs, IntoJs, Value};
use std::{cell::RefCell, rc::Rc};
use tokio::sync::Notify;

use super::{
    controller::{ControllerWrap, ReadableStreamDefaultController},
    from::from,
    queue_strategy::QueuingStrategy,
    reader::ReadableStreamDefaultReader,
    underlying_source::{JsUnderlyingSource, StreamSource, UnderlyingSource},
    NativeSource,
};

#[derive(Clone)]
#[rquickjs::class]
pub struct ReadableStream<'js> {
    v: UnderlyingSource<'js>,
    pub(super) ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
}

impl<'js> Trace<'js> for ReadableStream<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.v.trace(tracer);
        self.ctrl.trace(tracer);
    }
}

impl<'js> ReadableStream<'js> {
    pub fn to_stream(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<LocalBoxStream<'js, rquickjs::Result<Value<'js>>>> {
        let reader = self.get_reader(ctx.clone())?;

        let stream = async_stream::try_stream! {
            loop {
                let next = reader.borrow_mut().read(ctx.clone()).await?;

                if  let Some(value) = next.value {
                    yield value
                } else {
                    break;
                }

            }
        };
        Ok(Box::pin(stream))
    }

    pub async fn to_bytes(&self, ctx: Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
        let reader = self.get_reader(ctx.clone())?;

        let mut output = Vec::default();

        loop {
            let next = reader.borrow_mut().read(ctx.clone()).await?;

            if let Some(chunk) = next.value {
                let buffer = Buffer::from_js(&ctx, chunk)?;
                if let Some(bytes) = buffer.as_raw() {
                    output.extend_from_slice(bytes.slice());
                }

                buffer.detach()?;
            }

            if next.done {
                break;
            }
        }

        Ok(output)
    }
}

impl<'js> AsyncIterable<'js> for ReadableStream<'js> {
    type Item = Value<'js>;

    type Error = rquickjs::Error;

    type Stream = Static<LocalBoxStream<'js, Result<Self::Item, Self::Error>>>;

    fn stream(
        &mut self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<klaver::shared::iter::AsyncIter<Self::Stream>> {
        Ok(AsyncIter::new(Static(self.to_stream(ctx.clone())?)))
    }
}

macro_rules! catch {
    ($ctrl: expr, $expr: expr) => {
        match $expr {
            Ok(ret) => ret,
            Err(err) => {
                $ctrl.borrow_mut().set_error(err);
                return;
            }
        }
    };
}

fn pull<'js>(
    ctx: Ctx<'js>,
    mut source: UnderlyingSource<'js>,
    ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ready: Rc<Notify>,
) {
    ctx.clone().spawn(async move {
        catch!(ctrl, source.start(ctx.clone(), ctrl.clone()).await);

        loop {
            if ctrl.borrow().is_canceled() {
                source
                    .cancel(ctx.clone(), ctrl.borrow().cancel_reason().cloned())
                    .await
                    .ok();
            }

            if !ctrl.borrow().is_pulling() {
                break;
            }

            if ctrl.borrow().is_filled() {
                // Wait for someone to pop the queue
                ready.notified().await;
                if !ctrl.borrow().is_pulling() {
                    break;
                }
            }

            // Pull from the underlying source or break if no pull function exists
            if !catch!(ctrl, source.pull(ctx.clone(), ctrl.clone()).await) {
                break;
            }
        }
    });
}

impl<'js> ReadableStream<'js> {
    pub fn from_stream<T>(
        ctx: Ctx<'js>,
        stream: T,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>>
    where
        T: TryStream + Trace<'js> + Unpin + 'js,
        T::Error: std::error::Error,
        T::Ok: IntoJs<'js>,
    {
        let stream = StreamContainer(stream);

        Self::from_native(ctx, StreamSource(stream))
    }

    pub fn from_native<T>(
        ctx: Ctx<'js>,
        native: T,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>>
    where
        T: NativeSource<'js> + 'js,
    {
        let ready = Rc::new(Notify::new());

        let underlying_source = UnderlyingSource::Native(Rc::new(RefCell::from(native)));

        let controller = Class::instance(
            ctx.clone(),
            ReadableStreamDefaultController::new(
                ready.clone(),
                QueuingStrategy::create_default(ctx.clone())?,
            ),
        )?;

        let class = Class::instance(
            ctx.clone(),
            ReadableStream {
                v: underlying_source,
                ctrl: controller,
            },
        )?;

        let ctrl = class.borrow().ctrl.clone();
        let source = class.borrow().v.clone();

        pull(ctx, source, ctrl, ready);

        Ok(class)
    }

    pub fn is(value: &Value<'js>) -> bool {
        Class::<Self>::from_value(value).is_ok()
    }
}

#[rquickjs::methods]
impl<'js> ReadableStream<'js> {
    #[qjs(static)]
    pub fn from(
        ctx: Ctx<'js>,
        value: Value<'js>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        from(ctx, value)
    }

    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        options: JsUnderlyingSource<'js>,
        queuing_strategy: Opt<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        let ready = Rc::new(Notify::new());

        let queuing_strategy = if let Some(queue_strategy) = queuing_strategy.0 {
            queue_strategy
        } else {
            QueuingStrategy::create_default(ctx.clone())?
        };

        let underlying_source = UnderlyingSource::Js(options);

        let controller = Class::instance(
            ctx.clone(),
            ReadableStreamDefaultController::new(ready.clone(), queuing_strategy),
        )?;

        let class = Class::instance(
            ctx.clone(),
            ReadableStream {
                v: underlying_source,
                ctrl: controller,
            },
        )?;

        let ctrl = class.borrow().ctrl.clone();
        let source = class.borrow().v.clone();

        pull(ctx, source, ctrl, ready);

        Ok(class)
    }

    #[qjs(get)]
    pub fn locked(&self) -> rquickjs::Result<bool> {
        Ok(self.ctrl.borrow().is_locked())
    }

    pub fn cancel(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<rquickjs::String<'js>>,
    ) -> rquickjs::Result<()> {
        self.ctrl.borrow_mut().cancel(&ctx, reason.0)?;
        Ok(())
    }

    #[qjs(rename = "getReader")]
    pub fn get_reader(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Class<'js, ReadableStreamDefaultReader<'js>>> {
        // Uptain the lock
        self.ctrl.borrow_mut().lock(ctx.clone())?;

        Class::instance(
            ctx,
            ReadableStreamDefaultReader {
                ctrl: ControllerWrap::new(self.ctrl.clone()),
            },
        )
    }
}
