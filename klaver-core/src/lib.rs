mod context;
mod core;
pub mod error;
mod extensions;
mod inherit;
pub mod sync;
pub mod value;

#[cfg(feature = "structured-clone")]
mod export;

#[cfg(feature = "structured-clone")]
pub use self::{export::*, value::structured_clone::Registry};

pub use self::{
    context::AsContext,
    core::Core,
    error::{CaugthException, RuntimeError},
    extensions::Extensions,
    inherit::*,
    value::{ArrayExt, FunctionExt, ObjectExt, StringExt},
};

pub fn register(ctx: &rquickjs::Ctx) -> rquickjs::Result<()> {
    let core = Core::new(ctx)?;
    ctx.globals().set("$_runtime", core)?;
    crate::value::register(ctx)?;

    Ok(())
}

pub mod prelude {
    pub use crate::value::prelude::*;
}

pub use rquickjs;
