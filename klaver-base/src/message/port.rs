use crate::{Emitter, EventList, EventTarget};
use rquickjs::{Ctx, JsLifetime, class::Trace};
use rquickjs_util::{Subclass, throw};

#[derive(Trace, Default)]
#[rquickjs::class]
pub struct MessagePort<'js> {
    listener: EventList<'js>,
}

unsafe impl<'js> JsLifetime<'js> for MessagePort<'js> {
    type Changed<'to> = MessagePort<'to>;
}

#[rquickjs::methods]
impl<'js> MessagePort<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<MessagePort<'js>> {
        throw!(ctx, "MessagePort cannot be constructed directly")
    }
}

impl<'js> Emitter<'js> for MessagePort<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listener
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listener
    }
}

impl<'js> Subclass<'js, EventTarget<'js>> for MessagePort<'js> {}
