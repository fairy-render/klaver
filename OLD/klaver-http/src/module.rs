use rquickjs::{Class, module::ModuleDef, prelude::Func};

use crate::router::Router;

pub struct HttpModule;

impl ModuleDef for HttpModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("serve")?;
        decl.declare("Router")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export("serve", Func::from(crate::serve::js_serve_router))?;
        exports.export("Router", Class::<Router>::create_constructor(ctx)?)?;
        Ok(())
    }
}
