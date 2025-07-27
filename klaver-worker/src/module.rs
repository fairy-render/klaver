use klaver_base::{Exportable, Registry};
use rquickjs::{class::JsClass, module::ModuleDef};

use crate::WebWorker;

pub struct WorkerModule;

impl ModuleDef for WorkerModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(WebWorker::NAME)?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        let reg = Registry::get(ctx)?;
        WebWorker::export(ctx, &reg, exports)?;

        Ok(())
    }
}

impl<'js> Exportable<'js> for WorkerModule {
    fn export<T>(ctx: &rquickjs::Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        WebWorker::export(ctx, registry, target)?;
        Ok(())
    }
}
