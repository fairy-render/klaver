use klaver_base::{Emitter, EventKey, MessageChannel, MessagePort};
use klaver_core::{
    Exportable, Registry,
    value::structured_clone::SerializationOptions,
};
use klaver_modules::WeakEnviron;
use klaver_runtime::{AsyncState, TaskHandle};
use rquickjs::{
    Class, Ctx, FromJs, Function, JsLifetime, Value,
    class::{JsClass, Trace},
    prelude::Opt,
};

use crate::resource::WorkerResource;

#[rquickjs::class(rename = "Worker")]
pub struct WebWorker<'js> {
    port: Class<'js, MessagePort<'js>>,
    onmessage: Option<Function<'js>>,
    handle: Option<TaskHandle>,
}

impl<'js> Trace<'js> for WebWorker<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.port.trace(tracer);
        self.onmessage.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for WebWorker<'js> {
    type Changed<'to> = WebWorker<'to>;
}

#[rquickjs::methods]
impl<'js> WebWorker<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        path: std::string::String,
    ) -> rquickjs::Result<Class<'js, WebWorker<'js>>> {
        let registry = Registry::instance(&ctx)?;

        let channel = MessageChannel::new(ctx.clone())?;

        let port = channel.port1;
        let channel = channel.port2.borrow_mut().detach(&ctx)?;

        let env = ctx
            .userdata::<WeakEnviron>()
            .unwrap()
            .clone()
            .upgrade(&ctx)?;

        let resource = WorkerResource::new(path, env, channel, registry);

        let handle = AsyncState::push(&ctx, resource)?;

        MessagePort::start_native(&ctx, port.clone())?;

        let this = Class::instance(
            ctx.clone(),
            WebWorker {
                port,
                onmessage: None,
                handle: Some(handle),
            },
        )?;

        Ok(this)
    }

    #[qjs(rename = "postMessage")]
    pub fn post_message(
        &self,
        ctx: Ctx<'js>,
        value: Value<'js>,
        opts: Opt<SerializationOptions<'js>>,
    ) -> rquickjs::Result<()> {
        self.port.borrow().post_message(ctx, value, opts)?;

        Ok(())
    }

    #[qjs(rename = "addEventListener")]
    pub fn add_event_listener(
        &self,
        event: EventKey<'js>,
        cb: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.port
            .borrow_mut()
            .add_event_listener_native(event, cb)?;
        Ok(())
    }

    #[qjs(rename = "removeEventListener")]
    pub fn remove_event_listener(
        &self,
        event: EventKey<'js>,
        cb: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.port
            .borrow_mut()
            .remove_event_listener_native(event, cb)?;
        Ok(())
    }

    #[qjs(set, rename = "onmessage")]
    pub fn set_onmessage(&mut self, ctx: Ctx<'js>, cb: Opt<Function<'js>>) -> rquickjs::Result<()> {
        self.port.borrow_mut().set_onmessage(ctx, cb.0)
    }

    #[qjs(get, rename = "onmessage")]
    pub fn get_onmessage(&self) -> rquickjs::Result<Value<'js>> {
        self.port.borrow_mut().get_onmessage()
    }
    pub fn terminate(&mut self) -> rquickjs::Result<()> {
        let Some(handle) = self.handle.take() else {
            return Ok(());
        };
        handle.kill();
        self.port.borrow_mut().close();

        Ok(())
    }
}

// impl<'js> Emitter<'js> for WebWorker<'js> {
//     fn get_listeners(&self) -> &EventList<'js> {
//         &self.listeners
//     }

//     fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
//         &mut self.listeners
//     }
// }

// impl<'js> Subclass<'js, EventTarget<'js>> for WebWorker<'js> {}

impl<'js> Exportable<'js> for WebWorker<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            WebWorker::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        // Self::inherit(ctx)?;

        Ok(())
    }
}

// async fn listen<'js>(
//     ctx: Ctx<'js>,
//     mut kill: Shutdown,
//     rx: Receiver<TransObject>,
//     worker: Class<'js, WebWorker<'js>>,
// ) -> rquickjs::Result<()> {
//     loop {
//         if kill.is_killed() {
//             return Ok(());
//         }

//         futures::select! {
//             ret = rx.recv_async() => {

//                 let Ok(ret) = ret else {
//                     // Channel closed, which means the worker thread is terminated
//                     return Ok(())
//                 };

//             }
//             _ = kill => {
//                 return Ok(())
//             }
//         }
//     }

//     Ok(())
// }
