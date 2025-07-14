use rquickjs::module::ModuleDef;

pub struct BaseModule;

impl ModuleDef for BaseModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(stringify!(AbortController))?;
        decl.declare(stringify!(AbortSignal))?;
        decl.declare(stringify!(EventTarget))?;
        decl.declare(stringify!(DOMExpection))?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        let _ = (exports, ctx);
        Ok(())
    }
}
