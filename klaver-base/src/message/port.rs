use crate::{Emitter, EventList, EventTarget, Exportable};
use klaver_util::{Subclass, throw};
use rquickjs::{
    Class, Ctx, JsLifetime, Value,
    class::{JsClass, Trace},
};

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

    #[qjs(rename = "postMessage")]
    pub fn post_message(&self, msg: Value<'js>) -> rquickjs::Result<()> {
        todo!("Postmessage!")
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

impl<'js> Exportable<'js> for MessagePort<'js> {
    fn export<T>(ctx: &Ctx<'js>, _registry: &crate::Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::ExportTarget<'js>,
    {
        target.set(
            ctx,
            MessagePort::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        MessagePort::inherit(ctx)?;

        Ok(())
    }
}
