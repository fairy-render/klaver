use super::bindings::structured_clone;
use rquickjs::{module::ModuleDef, prelude::Func};

pub struct StructuredCloneModule;

impl ModuleDef for StructuredCloneModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("structuredClone")?;
        Ok(())
    }

    fn evaluate<'js>(
        _ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export("structuredClone", Func::new(structured_clone))?;
        Ok(())
    }
}
