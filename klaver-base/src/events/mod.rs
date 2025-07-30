mod dyn_event;
mod emitter;
mod event;
mod event_target;
mod listener;

use crate::ExportTarget;

pub use self::{dyn_event::*, emitter::*, event::*, event_target::*, listener::*};
use rquickjs::class::JsClass;

pub(crate) fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(decl, Event, EventTarget);
    Ok(())
}

pub(crate) fn exports<'js, T: ExportTarget<'js>>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &crate::Registry,
    exports: &T,
) -> rquickjs::Result<()> {
    export!(ctx, registry, exports, Event, EventTarget);

    Ok(())
}
