use rquickjs::{module::ModuleDef, prelude::Func};

pub struct HttpModule;

impl ModuleDef for HttpModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("serve")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export("serve", Func::from(crate::serve::js_serve))?;
        Ok(())
    }
}
