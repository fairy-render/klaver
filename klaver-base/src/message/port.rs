use std::sync::Arc;

use crate::{Emitter, EventList, EventTarget, Exportable, NativeObject, TransObject};
use flume::{Receiver, Sender};
use klaver_util::{Subclass, throw};
use rquickjs::{
    Class, Ctx, Function, JsLifetime, Value,
    class::{JsClass, Trace},
    qjs,
};

pub struct Message {
    message: TransObject,
}

#[derive(Clone)]
pub struct Channel {
    remote: Sender<Message>,
    rx: Arc<Receiver<Message>>,
}

#[rquickjs::class]
pub struct MessagePort<'js> {
    listener: EventList<'js>,
    channel: Channel,
    onmessage: Option<Function<'js>>,
}

impl<'js> MessagePort<'js> {
    pub fn create(remote: Sender<Message>, rx: Receiver<Message>) -> MessagePort<'js> {
        MessagePort {
            listener: Default::default(),
            onmessage: None,
            channel: Channel {
                remote,
                rx: rx.into(),
            },
        }
    }
}

impl<'js> Trace<'js> for MessagePort<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.listener.trace(tracer);
        self.onmessage.trace(tracer);
    }
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

    #[qjs(set, rename = "onmessage")]
    pub fn set_onmessage(&mut self, func: Function<'js>) -> rquickjs::Result<()> {
        Ok(())
    }

    #[qjs(get, rename = "onmessage")]
    pub fn get_onmessage(&mut self) -> rquickjs::Result<Value<'js>> {
        Ok(())
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
