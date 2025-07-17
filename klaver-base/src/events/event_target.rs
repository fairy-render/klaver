use rquickjs::{
    Class, Ctx, JsLifetime, Object, Value,
    class::{JsClass, Trace},
    prelude::Func,
};
use rquickjs_util::{Inheritable, SuperClass, throw};

use super::emitter::{Emitter, EventList};

#[rquickjs::class]
#[derive(Trace)]
pub struct EventTarget<'js> {
    listeners: EventList<'js>,
}

unsafe impl<'js> JsLifetime<'js> for EventTarget<'js> {
    type Changed<'to> = EventTarget<'to>;
}

#[rquickjs::methods]
impl<'js> EventTarget<'js> {
    #[qjs(constructor)]
    pub fn new() -> rquickjs::Result<EventTarget<'js>> {
        Ok(EventTarget {
            listeners: Default::default(),
        })
    }
}

impl<'js, T> Inheritable<'js, T> for EventTarget<'js>
where
    T: JsClass<'js> + Emitter<'js>,
{
    fn additional_override(ctx: &Ctx<'js>, proto: &Object<'js>) -> rquickjs::Result<()> {
        proto.set("addEventListener", Func::new(T::add_event_listener))?;
        proto.set("removeEventListener", Func::new(T::remove_event_listener))?;
        proto.set("dispatchEvent", Func::new(T::dispatch_event))?;

        Ok(())
    }
}

impl<'js> SuperClass<'js> for EventTarget<'js> {}

impl<'js> Emitter<'js> for EventTarget<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listeners
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listeners
    }
}
