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

pub fn evaluate<'js>(
    ctx: &rquickjs::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    define!(exports, ctx, Event, EventTarget);

    EventTarget::add_event_target_prototype(ctx)?;
    Event::add_event_prototype(ctx)?;

    Ok(())
}
