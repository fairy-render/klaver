use futures::stream::LocalBoxStream;
use klaver::shared::{
    iter::{AsyncIter, AsyncIterable},
    Static,
};
use rquickjs::{class::Trace, Class, Ctx, Value};
use std::rc::Rc;
use tokio::sync::Notify;

use super::{
    controller::ReadableStreamDefaultController,
    queue_strategy::QueuingStrategy,
    reader::ReadableStreamDefaultReader,
    underlying_source::{JsUnderlyingSource, UnderlyingSource},
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

fn pull<'js>(
    ctx: Ctx<'js>,
    mut source: UnderlyingSource<'js>,
    ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ready: Rc<Notify>,
) {
    ctx.clone().spawn(async move {
        if source.start(ctx.clone(), ctrl.clone()).await.is_err() {
            return;
        }

        loop {
            if !ctrl.borrow().is_running() {
                break;
            }

            if ctrl.borrow().is_filled() {
                // Wait for someone to pop the queue
                ready.notified().await;
                if !ctrl.borrow().is_running() {
                    break;
                }
            }

            // Pull from the underlying source or break if no pull function exists
            if !source
                .pull(ctx.clone(), ctrl.clone())
                .await
                .unwrap_or(false)
            {
                break;
            }
        }
    });
}

#[rquickjs::methods]
impl<'js> ReadableStream<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        options: JsUnderlyingSource<'js>,
        queuing_strategy: Option<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        let ready = Rc::new(Notify::new());

        let queuing_strategy = if let Some(queue_strategy) = queuing_strategy {
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

    #[qjs(rename = "getReader")]
    pub fn get_reader(&self, ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        // Uptain the lock
        self.ctrl.borrow_mut().lock(ctx)?;

        ReadableStreamDefaultReader::new(self.clone())
    }
}
