use klaver_modules::module_info;
use rquickjs::{class::JsClass, module::ModuleDef, Class};

use crate::hbs::Handlebars;

pub struct Module;

module_info!("@klaver/hbs" @types: include_str!("../module.d.ts") => Module);

impl ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(Handlebars::NAME)?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export(
            stringify!(Handlebars),
            Class::<Handlebars>::create_constructor(ctx)?,
        )?;
        Ok(())
    }
}
