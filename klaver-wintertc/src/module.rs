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
