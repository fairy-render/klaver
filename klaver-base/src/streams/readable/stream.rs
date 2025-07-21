use std::{cell::RefCell, rc::Rc};

use rquickjs::{Class, Ctx, JsLifetime, String, Value, class::Trace, prelude::Opt};
use rquickjs_util::throw;

use crate::streams::{
    WritableStream,
    data::{StreamData, WaitWriteReady},
    queue_strategy::QueuingStrategy,
    readable::{
        NativeSource,
        controller::ReadableStreamDefaultController,
        from::from,
        reader::ReadableStreamDefaultReader,
        underlying_source::{JsUnderlyingSource, UnderlyingSource},
    },
};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStream<'js> {
    data: Class<'js, StreamData<'js>>,
}

impl<'js> ReadableStream<'js> {
    pub fn from_native<T: NativeSource<'js> + 'js>(
        ctx: Ctx<'js>,
        native: T,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        Self::from_underlying_source(
            ctx,
            UnderlyingSource::Native(Rc::new(RefCell::from(native))),
            None,
        )
    }

    pub fn from_underlying_source(
        ctx: Ctx<'js>,
        underlying_source: UnderlyingSource<'js>,
        queue_strategy: Option<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        let queue_strategy = match queue_strategy {
            Some(ret) => ret,
            None => QueuingStrategy::create_default(ctx.clone())?,
        };

        let data = Class::instance(ctx.clone(), StreamData::new(queue_strategy))?;

        let ctrl = Class::instance(
            ctx.clone(),
            ReadableStreamDefaultController { data: data.clone() },
        )?;

        pull(ctx, underlying_source, data.clone(), ctrl)?;

        Ok(ReadableStream { data })
    }

    pub fn is(value: &Value<'js>) -> bool {
        Class::<Self>::from_value(value).is_ok()
    }
}

#[rquickjs::methods]
impl<'js> ReadableStream<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        underlying_source: JsUnderlyingSource<'js>,
        queue_strategy: Opt<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        Self::from_underlying_source(
            ctx,
            UnderlyingSource::Js(underlying_source),
            queue_strategy.0,
        )
    }

    #[qjs(static)]
    pub fn from(
        ctx: Ctx<'js>,
        value: Value<'js>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        from(ctx, value)
    }

    pub fn cancel(&self, ctx: Ctx<'js>, reason: Opt<String<'js>>) -> rquickjs::Result<()> {
        let reader = self.get_reader(ctx.clone())?;

        reader.cancel(ctx, reason)?;

        Ok(())
    }

    #[qjs(rename = "getReader")]
    fn get_reader(&self, ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        if self.data.borrow().is_locked() {
            throw!(@type ctx, "The stream you are trying to create a reader for is already locked to another reader")
        }

        Ok(ReadableStreamDefaultReader {
            data: Some(self.data.clone()),
        })
    }

    #[qjs(get)]
    pub fn locked(&self) -> rquickjs::Result<bool> {
        Ok(self.data.borrow().is_locked())
    }

    #[qjs(rename = "pipeTo")]
    pub async fn pipe_to(
        &self,
        ctx: Ctx<'js>,
        stream: Class<'js, WritableStream<'js>>,
    ) -> rquickjs::Result<()> {
        let reader = self.get_reader(ctx.clone())?;
        let writer = stream.borrow().get_writer(ctx.clone())?;

        loop {
            let next = reader.read(ctx.clone()).await?;

            if next.done {
                writer.close(ctx.clone()).await?;
                break;
            }

            let Some(next) = next.value else {
                writer.close(ctx.clone()).await?;
                break;
            };

            writer.ready().await?;

            writer.write(ctx.clone(), next)?;
        }

        Ok(())
    }
}

fn pull<'js>(
    ctx: Ctx<'js>,
    mut source: UnderlyingSource<'js>,
    data: Class<'js, StreamData<'js>>,
    ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
) -> rquickjs::Result<()> {
    ctx.clone().spawn(async move {
        if let Err(err) = source.start(ctx.clone(), ctrl.clone()).await {
            if err.is_exception() {
                let failure = ctx.catch();
                data.borrow_mut().fail(ctx.clone(), failure).ok();
            }
            return;
        }

        loop {
            if data.borrow().is_aborted() {
                source
                    .cancel(ctx.clone(), data.borrow().abort_reason())
                    .await
                    .ok();
                break;
            }

            if data.borrow().is_closed() && data.borrow().queue.is_empty() {
                data.borrow_mut().finished().ok();
                break;
            }

            if data.borrow().is_failed() {
                break;
            }

            if data.borrow().is_write_ready() {
                WaitWriteReady::new(data.clone()).await.ok();
            }

            if let Err(err) = source.pull(ctx.clone(), ctrl.clone()).await {
                if err.is_exception() {
                    let failure = ctx.catch();
                    data.borrow_mut().fail(ctx.clone(), failure).ok();
                }
                break;
            }
        }
    });
    Ok(())
}
