pub mod backend;
// mod event_target;
// mod timers;

use klaver_base::{Console, Exportable, Registry, StdConsoleWriter};
use rquickjs::Ctx;

pub struct WinterCG;

#[cfg(feature = "module")]
impl<'js> klaver_modules::GlobalInfo for WinterCG {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.global_dependency::<klaver_base::BaseModule>();

        #[cfg(feature = "intl")]
        builder.global_dependency::<klaver_intl::IntlModule>();
        #[cfg(feature = "crypto")]
        builder.global_dependency::<klaver_crypto::CryptoModule>();
        #[cfg(feature = "fetch")]
        builder.global_dependency::<klaver_fetch::FetchModule>();

        builder.global_dependency::<klaver_timers::TimeModule>();

        builder.register(WinterCG);
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for WinterCG {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            let registry = Registry::instance(&ctx)?;

            WinterCG::export(&ctx, &registry, &ctx.globals())?;

            Ok(())
        }
    }
}

impl<'js> Exportable<'js> for WinterCG {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        // Console
        let console = Console::new_with(StdConsoleWriter::default());
        target.set(ctx, "console", console)?;

        Ok(())
    }
}
