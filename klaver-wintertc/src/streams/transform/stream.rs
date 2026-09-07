use async_trait::async_trait;
use klaver_core::ExportTarget;
use rquickjs::{
    Class, Ctx, JsLifetime, Value,
    class::{JsClass, Trace},
    prelude::Opt,
};

use crate::streams::{
    ReadableStream, WritableStream,
    queue_strategy::QueuingStrategy,
    readable::NativeSource,
    transform::{controller::TransformStreamDefaultController, transformer::JsTransformer},
    writable::NativeSink,
    writable::WritableStreamDefaultController,
};

/// The readable side's underlying source: `TransformStream` doesn't pull data on demand - the
/// writable side pushes transformed chunks directly into the readable side's queue via the
/// shared `TransformStreamDefaultController` - so this never does anything itself.
pub(crate) struct PassiveSource;

impl<'js> Trace<'js> for PassiveSource {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[async_trait(?Send)]
impl<'js> NativeSource<'js> for PassiveSource {
    async fn start(
        &mut self,
        _ctx: Ctx<'js>,
        _ctrl: Class<'js, crate::streams::ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    async fn pull(
        &mut self,
        _ctx: Ctx<'js>,
        _ctrl: Class<'js, crate::streams::ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }
}

/// The writable side's underlying sink: forwards writes to the transformer's `transform()` hook
/// (or, with none given, enqueues the chunk unchanged - the spec's default identity transform),
/// and its `close`/`abort` to the transformer's `flush()` and to terminating/erroring the
/// readable side.
struct TransformSink<'js> {
    controller: Class<'js, TransformStreamDefaultController<'js>>,
    transformer: JsTransformer<'js>,
}

impl<'js> Trace<'js> for TransformSink<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.controller.trace(tracer);
        self.transformer.trace(tracer);
    }
}

#[async_trait(?Send)]
impl<'js> NativeSink<'js> for TransformSink<'js> {
    async fn start(
        &self,
        _ctx: &Ctx<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        let Some(start) = &self.transformer.start else {
            return Ok(());
        };

        let ret = start.call::<_, Value<'js>>((self.controller.clone(),))?;
        if let Some(promise) = ret.as_promise() {
            promise.clone().into_future::<Value<'js>>().await?;
        }

        Ok(())
    }

    async fn write(
        &self,
        ctx: &Ctx<'js>,
        chunk: Value<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        let result: rquickjs::Result<()> = async {
            match &self.transformer.transform {
                Some(transform) => {
                    let ret = transform.call::<_, Value<'js>>((chunk, self.controller.clone()))?;
                    if let Some(promise) = ret.as_promise() {
                        promise.clone().into_future::<Value<'js>>().await?;
                    }
                }
                None => {
                    self.controller.borrow().enqueue(ctx.clone(), chunk)?;
                }
            }
            Ok(())
        }
        .await;

        if result.is_err() {
            // A failing transform errors the whole stream, both sides - not just the writable
            // side (which the caller already handles via this same `Err`).
            let failure = ctx.catch();
            self.controller.borrow().error(ctx.clone(), failure).ok();
        }

        result
    }

    async fn close(
        &self,
        ctx: &Ctx<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        let result: rquickjs::Result<()> = async {
            if let Some(flush) = &self.transformer.flush {
                let ret = flush.call::<_, Value<'js>>((self.controller.clone(),))?;
                if let Some(promise) = ret.as_promise() {
                    promise.clone().into_future::<Value<'js>>().await?;
                }
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.controller.borrow().terminate(ctx.clone()),
            Err(_) => {
                let failure = ctx.catch();
                self.controller.borrow().error(ctx.clone(), failure).ok();
                result
            }
        }
    }

    async fn abort(&self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        let reason = reason.unwrap_or_else(|| Value::new_undefined(ctx.clone()));
        self.controller.borrow().error(ctx.clone(), reason)
    }
}

/// `TransformStream`, per <https://streams.spec.whatwg.org/#ts-class>.
#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct TransformStream<'js> {
    #[qjs(get)]
    readable: Class<'js, ReadableStream<'js>>,
    #[qjs(get)]
    writable: Class<'js, WritableStream<'js>>,
}

#[rquickjs::methods]
impl<'js> TransformStream<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        transformer: Opt<JsTransformer<'js>>,
        writable_strategy: Opt<QueuingStrategy<'js>>,
        readable_strategy: Opt<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<TransformStream<'js>> {
        // Per spec, `readableStrategy` defaults to a high water mark of 0 (not 1, unlike every
        // other stream default) - so that backpressure on the readable side is communicated to
        // the writable side as soon as a single chunk is buffered.
        let readable_strategy = match readable_strategy.0 {
            Some(strategy) => strategy,
            None => QueuingStrategy::create_with_high_water_mark(&ctx, 0)?,
        };

        let readable = Class::instance(
            ctx.clone(),
            ReadableStream::from_native(&ctx, PassiveSource, Some(readable_strategy))?,
        )?;

        let controller = Class::instance(
            ctx.clone(),
            TransformStreamDefaultController {
                readable: readable.borrow().data(),
            },
        )?;

        let sink = TransformSink {
            controller,
            transformer: transformer.0.unwrap_or_default(),
        };

        let writable = Class::instance(
            ctx.clone(),
            WritableStream::from_native(&ctx, sink, writable_strategy.0)?,
        )?;

        Ok(TransformStream { readable, writable })
    }
}

impl<'js> klaver_core::Exportable<'js> for TransformStream<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: ExportTarget<'js>,
    {
        target.set(
            ctx,
            TransformStream::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        Ok(())
    }
}
