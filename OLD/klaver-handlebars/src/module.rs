use rquickjs::{module::ModuleDef, Class};
use rquickjs_modules::module_info;

use crate::hbs::Handlebars;

pub struct Module;

module_info!("@klaver/hbs" @types: include_str!("../module.d.ts") => Module);

impl ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(stringify!(Handlebars))?;
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
