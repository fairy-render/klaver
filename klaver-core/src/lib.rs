mod context;
mod core;
pub mod error;
mod extensions;

pub use self::{context::AsContext, core::Core, error::RuntimeError, extensions::Extensions};

pub fn register(ctx: &rquickjs::Ctx) -> rquickjs::Result<()> {
    let core = Core::new(ctx)?;
    ctx.globals().set("$_runtime", core)?;
    Ok(())
}
