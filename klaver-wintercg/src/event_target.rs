use klaver_base::{DynEvent, Emitter, EventKey, EventTarget, ExportTarget};
use rquickjs::{Class, Ctx, Function, IntoJs, String, prelude::Func};
use rquickjs_util::{StringRef, util::FunctionExt};

use crate::event_target;

fn add_event_listener<'js>(
    ctx: Ctx<'js>,
    event_target: Class<'js, EventTarget<'js>>,
    event: StringRef<'js>,
    cb: Function<'js>,
) -> rquickjs::Result<()> {
    event_target
        .borrow_mut()
        .add_event_listener_native(EventKey::new(event), cb)
}

fn remove_event_listener<'js>(
    ctx: Ctx<'js>,
    event_target: Class<'js, EventTarget<'js>>,
    event: StringRef<'js>,
    cb: Function<'js>,
) -> rquickjs::Result<()> {
    event_target
        .borrow_mut()
        .remove_event_listener_native(EventKey::new(event), cb)
}

fn dispatch_event<'js>(
    ctx: Ctx<'js>,
    event_target: Class<'js, EventTarget<'js>>,
    event: DynEvent<'js>,
) -> rquickjs::Result<()> {
    event_target.borrow_mut().dispatch_native(&ctx, event)
}

pub fn export<'js, T: ExportTarget<'js>>(ctx: &Ctx<'js>, target: &T) -> rquickjs::Result<()> {
    let add_event_listener = Func::new(add_event_listener)
        .into_js(ctx)?
        .into_function()
        .expect("add_event_listener");

    let remove_event_listener = Func::new(remove_event_listener)
        .into_js(ctx)?
        .into_function()
        .expect("remove_event_listener");

    let dispatch_event = Func::new(dispatch_event)
        .into_js(ctx)?
        .into_function()
        .expect("dispatch_event");

    let event_target = Class::instance(ctx.clone(), EventTarget::new()?)?;

    target.set(
        ctx,
        "addEventListener",
        add_event_listener.bind(ctx.clone(), (event_target.clone(),)),
    )?;

    target.set(
        ctx,
        "removeEventListener",
        remove_event_listener.bind(ctx.clone(), (event_target.clone(),)),
    )?;

    target.set(
        ctx,
        "dispatchEvent",
        dispatch_event.bind(ctx.clone(), (event_target.clone(),)),
    )?;

    Ok(())
}
