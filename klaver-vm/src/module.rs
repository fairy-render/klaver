use klaver_base::{Exportable, Registry};
use klaver_modules::module_info;
use rquickjs::module::ModuleDef;

use crate::bindings::JsVm;

pub struct VmModule;

impl ModuleDef for VmModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("Vm")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        JsVm::export(ctx, &Registry::instance(ctx)?, exports)?;
        Ok(())
    }
}

module_info!("klaver:vm" @types: include_str!("../klaver.vm.d.ts") => VmModule);
