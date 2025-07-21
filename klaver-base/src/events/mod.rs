mod dyn_event;
mod emitter;
mod event;
mod event_target;
mod listener;

pub use self::{dyn_event::*, emitter::*, event::*, event_target::*, listener::*};
use rquickjs::class::JsClass;

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(decl, Event, EventTarget);
    Ok(())
}

pub fn exports<'js>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &crate::Registry,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    export!(ctx, registry, exports, Event, EventTarget);

    Ok(())
}
