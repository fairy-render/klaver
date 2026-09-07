// pub mod backend;
use crate::console::{Console, StdConsoleWriter};
use klaver_core::{Exportable, Registry};
use rquickjs::Ctx;

pub struct WinterTC;

#[cfg(feature = "module")]
impl<'js> klaver_modules::GlobalInfo for WinterTC {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        use crate::base::BaseModule;

        builder.register(WinterTC);

        builder.global_dependency::<BaseModule>();
        #[cfg(feature = "intl")]
        builder.global_dependency::<crate::intl::IntlModule>();
        #[cfg(feature = "crypto")]
        builder.global_dependency::<crate::crypto::CryptoModule>();
        #[cfg(feature = "fetch")]
        builder.global_dependency::<crate::fetch::FetchModule>();
        #[cfg(feature = "timers")]
        builder.global_dependency::<crate::timers::TimeModule>();
        #[cfg(feature = "worker")]
        builder.global_dependency::<crate::worker::WorkerModule>();
        #[cfg(feature = "fs")]
        builder.global_dependency::<crate::fs::FsModule>();
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for WinterTC {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            let registry = Registry::instance(&ctx)?;

            WinterTC::export(&ctx, &registry, &ctx.globals())?;

            Ok(())
        }
    }
}

impl<'js> Exportable<'js> for WinterTC {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        // Console
        let console = Console::new_with(StdConsoleWriter::default());
        target.set(ctx, "console", console)?;

        Ok(())
    }
}

#[cfg(all(test, feature = "module"))]
mod tests {
    use klaver_vm::Options;
    use rquickjs::CatchResultExt;

    /// Regression test for `Vm::create_context()`: it builds a second `rquickjs::Context` on the
    /// *same* underlying `Runtime` as the `Vm`'s own context, which re-runs every registered
    /// global's setup again - including `Event`/`EventTarget`/`MessageEvent`'s hand-written
    /// prototype-accessor installation (`NativeEvent::add_event_prototype_to`). Those accessors
    /// used to be defined non-configurable, so redefining them against the (Runtime-cached,
    /// shared-across-contexts) prototype object on the second context threw a `TypeError`. This
    /// exercises both the base `Event` and the `MessageEvent` subclass path.
    #[tokio::test]
    async fn create_context_reuses_shared_prototypes_without_throwing() {
        let vm = Options::default()
            .global::<crate::WinterTC>()
            .build()
            .await
            .unwrap();

        // Used to throw here before the fix.
        let ctx2 = vm.create_context().await.unwrap();

        ctx2.async_with(async |ctx| {
            let promise: rquickjs::Promise = ctx
                .eval(
                    r#"(async () => {
                        const target = new EventTarget();
                        let seenType;
                        target.addEventListener("ping", (e) => { seenType = e.type; });
                        target.dispatchEvent(new Event("ping"));
                        if (seenType !== "ping") throw new Error(`event type was ${seenType}`);

                        const msg = new MessageEvent("message", { data: "hello" });
                        if (msg.type !== "message") throw new Error(`message type was ${msg.type}`);
                        if (msg.data !== "hello") throw new Error(`message data was ${msg.data}`);
                    })()"#,
                )
                .catch(&ctx)?;

            promise.into_future::<()>().await.catch(&ctx)?;

            Ok(())
        })
        .await
        .unwrap();
    }
}
