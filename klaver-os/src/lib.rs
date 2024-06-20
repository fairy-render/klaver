use rquickjs::{Class, Ctx, Module, Result};
use stream::AsyncByteIter;

pub mod env;
pub mod shell;

pub mod stream;

pub fn register(ctx: &Ctx<'_>) -> Result<()> {
    Module::declare_def::<env::js_env_module, _>(ctx.clone(), "@klaver/env")?;
    Module::declare_def::<shell::js_shell_mod, _>(ctx.clone(), "@klaver/shell")?;

    Class::<AsyncByteIter>::register(ctx)?;
    Class::<AsyncByteIter>::define(&ctx.globals())?;

    Ok(())
}
