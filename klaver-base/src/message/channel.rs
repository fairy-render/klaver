use crate::message::port::MessagePort;
use rquickjs::{Class, Ctx, JsLifetime, class::Trace};

#[derive(Trace)]
#[rquickjs::class]
pub struct MessageChannel<'js> {
    #[qjs(get)]
    port1: Class<'js, MessagePort<'js>>,
    #[qjs(get)]
    port2: Class<'js, MessagePort<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for MessageChannel<'js> {
    type Changed<'to> = MessageChannel<'to>;
}

#[rquickjs::methods]
impl<'js> MessageChannel<'js> {
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<MessageChannel<'js>> {
        let (sx1, rx1) = flume::unbounded();
        let (sx2, rx2) = flume::unbounded();

        Ok(MessageChannel {
            port1: Class::instance(ctx.clone(), MessagePort::create(sx2, rx1))?,
            port2: Class::instance(ctx, MessagePort::create(sx1, rx2))?,
        })
    }
}

create_export!(MessageChannel<'js>);
