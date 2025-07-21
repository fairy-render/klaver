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
