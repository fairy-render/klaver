#[cfg(feature = "async")]
pub mod async_iterator;
mod buffer;
mod context;
mod date;
mod equal;
mod extensions;
mod finalization_registry;
mod format;
pub mod iterable;
mod map;
mod primordials;
mod regexp;
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

use crate::Core;

pub use self::{
    buffer::*, context::AsContext, date::Date, equal::equal, extensions::*,
    finalization_registry::FinalizationRegistry, format::*, map::*, regexp::RegExp, set::*,
    string_ref::StringRef, tuples::*, typed_array::*, typed_map::*, typed_multi_map::*, util::*,
    weak_map::WeakMap,
};

pub mod prelude {
    pub use super::context::AsContext;
    pub use super::extensions::*;
    pub use super::iterable::NativeIteratorExt;
}

pub(crate) fn register(ctx: &rquickjs::Ctx) -> rquickjs::Result<()> {
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
