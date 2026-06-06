#[cfg(feature = "async")]
pub mod async_iterator;
mod context;
mod date;
mod equal;
mod extensions;
mod inherit;
pub mod iterable;
mod map;
mod primordials;
mod set;
mod string_ref;
#[cfg(feature = "structured-clone")]
pub mod structured_clone;
mod tuples;
mod typed_array;
mod typed_map;
mod typed_multi_map;
mod util;
mod weak_map;

use klaver_core::Core;

pub use self::{
    context::AsContext, date::Date, equal::equal, extensions::*, inherit::*, map::Map, set::Set,
    string_ref::StringRef, typed_array::*, typed_map::TypedMap, typed_multi_map::TypedMultiMap,
    util::is_plain_object, weak_map::WeakMap,
};

pub mod prelude {
    pub use super::context::AsContext;
    pub use super::extensions::*;
}

pub fn register(ctx: &rquickjs::Ctx) -> rquickjs::Result<()> {
    let primordials = primordials::BasePrimordials::new(ctx)?;
    let core = Core::from_ctx(ctx)?;
    core.borrow_mut().register("primordials", primordials)?;
    #[cfg(feature = "structured-clone")]
    {
        let registry = structured_clone::Registry::new(ctx)?;
        ctx.store_userdata(registry)?;
    }
    Ok(())
}
