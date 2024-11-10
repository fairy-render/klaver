use crate::{
    abort_controller::{AbortController, AbortSignal},
    blob::Blob,
    event_target::{Emitter, Event, EventTarget},
    DOMException,
};

pub fn register<'js>(ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<()> {
    // let config = Class::instance(ctx.clone(), WinterCG::new(ctx.clone())?)?;
    // exports.export(stringify!(config), config.clone())?;

    define!(
        ctx,
        DOMException,
        EventTarget,
        Event,
        AbortController,
        AbortSignal,
        Blob
    );

    DOMException::init(ctx)?;
    EventTarget::add_event_target_prototype(ctx)?;
    AbortSignal::add_event_target_prototype(ctx)?;

    Ok(())
}
