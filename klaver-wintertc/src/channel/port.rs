use std::sync::Arc;

use crate::{
    channel::{MessageEvent, event::MessageEventOptions},
    events::{Emitter, EventCallback, EventKey, EventList, EventTarget},
};
use flume::{Receiver, Sender};
use futures::channel::oneshot;
use klaver_core::{
    Exportable, Subclass, throw,
    value::structured_clone::{
        Clonable, NativeData, NativeObject, SerializationContext, SerializationOptions,
        StructuredClone, Tag, TransObject, TransferData, register,
    },
};
use klaver_core::{Registry, throw_if};
use klaver_runtime::{AsyncState, Resource, ResourceId};
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

impl NativeObject for Channel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any + Send + Sync> {
        self
    }
}

#[rquickjs::class]
pub struct MessagePort<'js> {
    listener: EventList<'js>,
    channel: Option<Channel>,
    kill: Option<oneshot::Sender<()>>,
}

impl<'js> MessagePort<'js> {
    pub fn create(remote: Sender<Message>, rx: Receiver<Message>) -> MessagePort<'js> {
        MessagePort {
            listener: Default::default(),
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
            kill: None,
        }
    }
}

impl<'js> Trace<'js> for MessagePort<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.listener.trace(tracer);
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
        self.set_handler(
            EventKey::from_str(ctx, "message")?,
            func.map(EventCallback::Function),
        );
        Ok(())
    }

    #[qjs(get, rename = "onmessage")]
    pub fn get_onmessage(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<Function<'js>>> {
        Ok(self.get_handler_function(&EventKey::from_str(ctx, "message")?))
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
}

impl<'js> Subclass<'js, EventTarget<'js>> for MessagePort<'js> {}

impl<'js> Exportable<'js> for MessagePort<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        registry: &klaver_core::value::structured_clone::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            MessagePort::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        MessagePort::inherit(ctx)?;

        register::<Self>(ctx, registry)?;

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

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::NativeObject(data) => {
                let any = data.instance.into_any();
                let channel = throw_if!(
                    ctx,
                    any.downcast::<Channel>()
                        .map_err(|_| "Invalid object: Expected Channel")
                );

                let port = MessagePort::from_channel(*channel);
                Class::instance(ctx.ctx().clone(), port)
            }
            _ => throw!(ctx, "Invalid transfer data for MessagePort"),
        }
    }

    fn to_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        if !ctx.should_move(value.as_value()) {
            throw!(ctx, "MessagePort cannot be cloned: It is not transferable")
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

                    MessagePort::dispatch_native(&self.message_port, &ctx, event)?;
                }
                _ = &mut self.kill => {
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, NativeEvent};
    use rquickjs::{CatchResultExt, Context, Runtime};

    /// Builds a detached `MessagePort` (a real one, just not connected to a live peer - fine
    /// since these tests only exercise `onmessage`/`addEventListener`/`dispatchEvent`, not
    /// actual message delivery) and makes it available as the `port` global, alongside
    /// `EventTarget`, `Event` and `MessageEvent`. `body` is expected to throw on failure.
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                ctx.globals().set(
                    "EventTarget",
                    Class::<EventTarget>::create_constructor(&ctx)?,
                )?;
                EventTarget::add_event_target_prototype(&ctx)?;

                ctx.globals()
                    .set("Event", Class::<Event>::create_constructor(&ctx)?)?;
                Event::add_event_prototype(&ctx)?;

                ctx.globals().set(
                    "MessageEvent",
                    Class::<MessageEvent>::create_constructor(&ctx)?,
                )?;
                MessageEvent::inherit(&ctx)?;

                MessagePort::inherit(&ctx)?;

                let (tx, rx) = flume::unbounded();
                let port = Class::instance(ctx.clone(), MessagePort::create(tx, rx))?;
                ctx.globals().set("port", port)?;

                let test_fn: rquickjs::Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn message_event_inherits_base_event_behavior() {
        run(r#"
            const e = new MessageEvent("message", { data: 1 });
            if (!(e instanceof Event)) throw new Error("MessageEvent instance is not instanceof Event");
            if (e.type !== "message") throw new Error(`type was ${e.type}`);
            if (e.defaultPrevented) throw new Error("defaultPrevented was true");

            // Must not throw: dispatching it exercises `MessageEvent.prototype`'s own copies of
            // the base `Event` accessors (`type`, `defaultPrevented`, ...), not
            // `Event.prototype`'s (which would fail to unwrap `this` for a `MessageEvent`).
            port.dispatchEvent(e);
        "#);
    }

    #[test]
    fn onmessage_fires_synchronously_with_correct_this() {
        run(r#"
            let seenData;
            let seenThis;
            port.onmessage = function (e) {
                seenData = e.data;
                seenThis = this;
            };
            port.dispatchEvent(new MessageEvent("message", { data: 42 }));
            if (seenData !== 42) throw new Error(`data was ${seenData}`);
            if (seenThis !== port) throw new Error("onmessage's `this` was not the port");
        "#);
    }

    #[test]
    fn onmessage_and_add_event_listener_both_fire() {
        run(r#"
            const order = [];
            port.onmessage = () => order.push("onmessage");
            port.addEventListener("message", () => order.push("listener"));
            port.dispatchEvent(new MessageEvent("message"));
            if (order.join(",") !== "onmessage,listener") throw new Error(`order was ${order}`);
        "#);
    }

    #[test]
    fn reassigning_onmessage_replaces_the_previous_handler() {
        run(r#"
            let first = 0;
            let second = 0;
            port.onmessage = () => { first += 1; };
            port.onmessage = () => { second += 1; };
            port.dispatchEvent(new MessageEvent("message"));
            if (first !== 0) throw new Error(`first ran ${first} times`);
            if (second !== 1) throw new Error(`second ran ${second} times`);
        "#);
    }

    #[test]
    fn onmessage_getter_reflects_currently_assigned_function_and_clears() {
        run(r#"
            if (port.onmessage !== undefined) throw new Error("expected undefined before assignment");

            const fn = () => {};
            port.onmessage = fn;
            if (port.onmessage !== fn) throw new Error("getter did not return the assigned function");

            port.onmessage = null;
            if (port.onmessage !== undefined) throw new Error("getter did not return undefined after clearing");

            let called = false;
            port.dispatchEvent(new MessageEvent("message"));
            // No listener left at all - just checking dispatch doesn't throw with nothing
            // registered.
            if (called) throw new Error("unreachable");
        "#);
    }
}
