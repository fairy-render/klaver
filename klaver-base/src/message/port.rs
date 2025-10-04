use std::sync::Arc;

use crate::{
    Clonable, Emitter, EventList, EventTarget, Exportable, NativeData, Registry,
    SerializationOptions, StructuredClone, Tag, TransObject, TransferData,
    message::{MessageEvent, event::MessageEventOptions},
};
use flume::{Receiver, Sender};
use futures::channel::oneshot;
use klaver_runtime::{AsyncState, Resource, ResourceId};
use klaver_util::{Subclass, throw};
use rquickjs::{
    Class, Ctx, Function, JsLifetime, String, Value,
    class::{JsClass, Trace},
    prelude::{Opt, This},
};

pub struct Message {
    pub message: TransObject,
}

#[derive(Clone)]
pub struct Channel {
    remote: Sender<Message>,
    rx: Arc<Receiver<Message>>,
}

#[rquickjs::class]
pub struct MessagePort<'js> {
    listener: EventList<'js>,
    channel: Option<Channel>,
    onmessage: Option<Function<'js>>,
    kill: Option<oneshot::Sender<()>>,
}

impl<'js> MessagePort<'js> {
    pub fn create(remote: Sender<Message>, rx: Receiver<Message>) -> MessagePort<'js> {
        MessagePort {
            listener: Default::default(),
            onmessage: None,
            channel: Some(Channel {
                remote,
                rx: rx.into(),
            }),
            kill: None,
        }
    }

    pub fn from_channel(channel: Channel) -> MessagePort<'js> {
        MessagePort {
            listener: Default::default(),
            channel: Some(channel),
            onmessage: None,
            kill: None,
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

impl<'js> MessagePort<'js> {
    pub fn start_native(
        ctx: &Ctx<'js>,
        this: Class<'js, MessagePort<'js>>,
    ) -> rquickjs::Result<()> {
        if this.borrow().kill.is_some() {
            return Ok(());
        }

        let Some(channel) = this.borrow().channel.as_ref().cloned() else {
            throw!(ctx, "Port is detached")
        };

        let registry = Registry::instance(&ctx)?;
        let (sx, rx) = oneshot::channel();

        this.borrow_mut().kill = Some(sx);

        AsyncState::push(
            &ctx,
            MessagePortResource {
                registry,
                channel,
                message_port: this,
                kill: rx,
            },
        )?;

        Ok(())
    }
}

#[rquickjs::methods]
impl<'js> MessagePort<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<MessagePort<'js>> {
        throw!(ctx, "MessagePort cannot be constructed directly")
    }

    #[qjs(rename = "postMessage")]
    pub fn post_message(
        &self,
        ctx: Ctx<'js>,
        msg: Value<'js>,
        opts: Opt<SerializationOptions<'js>>,
    ) -> rquickjs::Result<()> {
        let Some(channel) = &self.channel else {
            throw!(ctx, "MessagePort is detached")
        };

        let opts = opts.0.unwrap_or_default();

        let message = Registry::instance(&ctx)?.serialize(&ctx, &msg, &opts)?;

        channel.remote.send(Message { message }).ok();

        Ok(())
    }

    #[qjs(set, rename = "onmessage")]
    pub fn set_onmessage(
        &mut self,
        ctx: Ctx<'js>,
        func: Option<Function<'js>>,
    ) -> rquickjs::Result<()> {
        self.onmessage = func;

        if self.onmessage.is_some() {}

        Ok(())
    }

    #[qjs(get, rename = "onmessage")]
    pub fn get_onmessage(&mut self) -> rquickjs::Result<Value<'js>> {
        todo!()
    }

    pub fn start(This(this): This<Class<'js, Self>>, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        Self::start_native(&ctx, this)
    }

    pub fn close(&mut self) {
        if let Some(sx) = self.kill.take() {
            sx.send(()).ok();
        }
    }
}

impl<'js> MessagePort<'js> {
    pub fn detach(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<Channel> {
        let Some(channel) = self.channel.take() else {
            throw!(ctx, "MessagePort already detached")
        };

        Ok(channel)
    }
}

impl<'js> Emitter<'js> for MessagePort<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listener
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listener
    }

    fn dispatch(&self, ctx: &Ctx<'js>, event: crate::DynEvent<'js>) -> rquickjs::Result<()> {
        if event.ty(ctx)?.as_str() == "message" {
            let Some(cb) = &self.onmessage else {
                return Ok(());
            };

            cb.defer((event,))?;
        }
        Ok(())
    }
}

impl<'js> Subclass<'js, EventTarget<'js>> for MessagePort<'js> {}

impl<'js> Exportable<'js> for MessagePort<'js> {
    fn export<T>(ctx: &Ctx<'js>, registry: &crate::Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::ExportTarget<'js>,
    {
        target.set(
            ctx,
            MessagePort::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        MessagePort::inherit(ctx)?;

        registry.register::<Self>().unwrap();

        Ok(())
    }
}

impl<'js> Clonable for MessagePort<'js> {
    type Cloner = MessagePortCloner;
}

pub struct MessagePortCloner;

impl StructuredClone for MessagePortCloner {
    type Item<'js> = Class<'js, MessagePort<'js>>;

    const TRANSFERBLE: bool = true;

    fn tag() -> &'static crate::Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &mut crate::SerializationContext<'js, '_>,
        obj: crate::TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        ctx: &mut crate::SerializationContext<'js, '_>,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<crate::TransferData> {
        if !ctx.should_move(value.as_value()) {
            throw!(ctx, "Move")
        }

        let channel = value.borrow_mut().detach(ctx.ctx())?;
        let data = NativeData {
            instance: Box::new(channel),
            id: 1,
        };
        Ok(TransferData::NativeObject(data))
    }
}

struct MessagePortResourceKey;

impl ResourceId for MessagePortResourceKey {
    fn name() -> &'static str {
        "MessagePort"
    }
}

struct MessagePortResource<'js> {
    registry: Registry,
    channel: Channel,
    message_port: Class<'js, MessagePort<'js>>,
    kill: oneshot::Receiver<()>,
}

impl<'js> Resource<'js> for MessagePortResource<'js> {
    type Id = MessagePortResourceKey;

    const INTERNAL: bool = false;
    const SCOPED: bool = false;

    async fn run(mut self, ctx: klaver_runtime::Context<'js>) -> rquickjs::Result<()> {
        loop {
            futures::select! {
                next = self.channel.rx.recv_async() => {
                    let Ok(next) = next else {
                        break;
                    };

                    let data = self.registry.deserialize(&ctx, next.message)?;

                    let msg = String::from_str(ctx.ctx().clone(), "message")?;

                    let event =
                        MessageEvent::new(msg, Opt(Some(MessageEventOptions { data: Some(data) })))?;

                    self.message_port.borrow_mut().dispatch_native(&ctx, event)?;
                }
                _ = &mut self.kill => {
                    break;
                }
            }
        }

        Ok(())
    }
}
