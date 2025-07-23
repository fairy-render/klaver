mod backend;
mod event_target;
mod timers;

use klaver_base::{Console, Exportable, NullWriter, Registry, StdConsoleWriter};
use rquickjs::Ctx;

pub struct WinterCG;

impl<'js> klaver_modules::GlobalInfo for WinterCG {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(WinterCG);
    }
}

impl klaver_modules::Global for WinterCG {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            let registry = Registry::get(&ctx)?;

            WinterCG::export(&ctx, &registry, &ctx.globals())?;

            Ok(())
        }
    }
}

impl<'js> Exportable<'js> for WinterCG {
    fn export<T>(
        ctx: &Ctx<'js>,
        registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        klaver_base::BaseModule::export(ctx, registry, target)?;
        klaver_worker::WebWorker::export(ctx, registry, target)?;
        klaver_fetch::FetchModule::export(ctx, registry, target)?;

        // EventTarget
        crate::event_target::export(ctx, target)?;

        // Timers
        crate::timers::export(ctx, target)?;

        // Console
        let console = Console::new_with(StdConsoleWriter::default());
        target.set(ctx, "console", console)?;

        Ok(())
    }
}
