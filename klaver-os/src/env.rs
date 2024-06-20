pub type Moodule = js_env_module;

#[rquickjs::module(rename_vars = "camelCase")]
pub mod env_module {
    use klaver_base::{get_base};
    use rquickjs::{Ctx, Function};

    #[qjs(declare)]
    pub fn declare(declare: &rquickjs::module::Declarations) -> rquickjs::Result<()> {
        declare.declare("cwd")?;
        declare.declare("args")?;
        Ok(())
    }

    #[qjs(evaluate)]
    pub fn evaluate<'js>(
        ctx: &Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export(
            "cwd",
            Function::new(ctx.clone(), |ctx: Ctx<'_>| {
                let base_class = get_base(&ctx)?;
                let base = base_class.try_borrow()?;

                rquickjs::Result::Ok(base.config.get_cwd().map(|m| m.display().to_string()))
            }),
        )?;

        exports.export(
            "args",
            Function::new(ctx.clone(), |ctx: Ctx<'_>| {
                let base_class = get_base(&ctx)?;
                let base = base_class.try_borrow()?;
                rquickjs::Result::Ok(base.config.get_args())
            }),
        )?;
        Ok(())
    }
}
