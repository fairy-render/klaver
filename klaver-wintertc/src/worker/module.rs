use klaver_core::{Exportable, Registry};

use super::WebWorker;

pub struct WorkerModule;

impl<'js> Exportable<'js> for WorkerModule {
    fn export<T>(ctx: &rquickjs::Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        WebWorker::export(ctx, registry, target)?;
        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for WorkerModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        Self::export(
            &ctx,
            &klaver_core::Registry::instance(&ctx)?,
            &ctx.globals(),
        )?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for WorkerModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(std::borrow::Cow::Borrowed(include_str!(
            "../../types/worker.d.ts"
        )))
    }
}
