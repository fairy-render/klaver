#[cfg(feature = "module")]
use klaver_core::Registry;
use klaver_core::{Exportable, value::structured_clone};
#[cfg(feature = "module")]
use rquickjs::Ctx;
use rquickjs::{Function, function::This, prelude::Func};

use crate::{
    abort_controller::{AbortController, AbortSignal},
    channel::ChannelModule,
    dom_exception::DOMException,
    encoding::EncodingModule,
    events::EventsModule,
    performance::Performance,
};

pub struct BaseModule;

fn queue_microtask<'js>(ctx: Ctx<'js>, callback: Function<'js>) -> rquickjs::Result<()> {
    let (promise, resolve, _reject) = ctx.promise()?;
    resolve.call::<_, ()>(())?;
    // `Promise#then` invokes its handler with the resolved value; `queueMicrotask` callbacks
    // take no arguments, so wrap it to drop that value.
    let wrapper = Function::new(ctx.clone(), move || -> rquickjs::Result<()> {
        callback.call(())
    })?;
    promise
        .then()?
        .call::<_, rquickjs::Value>((This(promise.clone()), wrapper))?;
    Ok(())
}

impl<'js> klaver_core::Exportable<'js> for BaseModule {
    fn export<T>(ctx: &Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        export!(
            ctx,
            registry,
            target,
            AbortController,
            AbortSignal,
            DOMException,
            Performance
        );

        EventsModule::export(ctx, registry, target)?;
        EncodingModule::export(ctx, registry, target)?;
        ChannelModule::export(ctx, registry, target)?;

        #[cfg(feature = "streams")]
        crate::streams::export(ctx, registry, target)?;
        #[cfg(feature = "streams")]
        crate::blob::Blob::export(ctx, registry, target)?;
        #[cfg(feature = "streams")]
        crate::blob::File::export(ctx, registry, target)?;
        target.set(
            ctx,
            "structuredClone",
            Func::from(structured_clone::structured_clone),
        )?;
        target.set(ctx, "serialize", Func::from(structured_clone::serialize))?;
        target.set(ctx, "queueMicrotask", Func::from(queue_microtask))?;
        target.set(ctx, "self", ctx.globals())?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for BaseModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        Self::export(&ctx, &Registry::instance(&ctx)?, &ctx.globals())?;
        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for BaseModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(BaseModule);
    }

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(std::borrow::Cow::Borrowed(include_str!(
            "../types/base.d.ts"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klaver_core::value::FunctionExt;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt};

    /// Runs `body` as the contents of an async IIFE, with `BaseModule` exported onto globals.
    /// `body` is expected to throw on failure (e.g. via a plain `if (...) throw ...`).
    fn run(body: &str) {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                BaseModule::export(&ctx, &Registry::instance(&ctx)?, &ctx.globals())?;

                let test_fn: Function = ctx.eval(format!("(async () => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call_async::<_, ()>(()).await.catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();
        });
    }

    #[test]
    fn self_aliases_global_object() {
        run(r#"
            if (typeof self === "undefined") throw new Error("self is not defined");
            if (self !== globalThis) throw new Error("self !== globalThis");
        "#);
    }

    #[test]
    fn queue_microtask_runs_before_later_microtasks() {
        run(r#"
            const order = [];
            queueMicrotask(() => order.push("microtask"));
            order.push("sync");
            await Promise.resolve();
            if (order.join(",") !== "sync,microtask") {
                throw new Error(`unexpected order: ${order.join(",")}`);
            }
        "#);
    }

    #[test]
    fn queue_microtask_passes_no_arguments() {
        run(r#"
            await new Promise((resolve, reject) => {
                queueMicrotask((...args) => {
                    if (args.length !== 0) {
                        reject(new Error(`expected no arguments, got ${args.length}`));
                    } else {
                        resolve();
                    }
                });
            });
        "#);
    }
}
