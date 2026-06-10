mod dyn_event;
mod emitter;
mod event;
mod event_target;
mod listener;

use klaver_core::ExportTarget;

pub use self::{dyn_event::*, emitter::*, event::*, event_target::*, listener::*};

pub(crate) fn exports<'js, T: ExportTarget<'js>>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &klaver_core::value::structured_clone::Registry,
    exports: &T,
) -> rquickjs::Result<()> {
    export!(ctx, registry, exports, Event, EventTarget);

    Ok(())
}
