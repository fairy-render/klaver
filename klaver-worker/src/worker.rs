use flume::{Receiver, Sender};
use klaver_base::{
    Emitter, EventKey, EventList, EventTarget, Exportable, MessageChannel, MessagePort, Registry,
    SerializationOptions, TransObject,
};
use klaver_runner::{Shutdown, Workers};
use klaver_util::{RuntimeError, StringRef, Subclass};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, FromJs, Function, JsLifetime, Module,
    String, Value,
    class::{JsClass, Trace},
    prelude::Opt,
};

use crate::work::{Message, work};

#[rquickjs::class(rename = "Worker")]
pub struct WebWorker<'js> {
    port: Class<'js, MessagePort<'js>>,
    onmessage: Option<Function<'js>>,
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
        let workers = Workers::from_ctx(&ctx)?;

        let registry = Registry::instance(&ctx)?;

        let channel = MessageChannel::new(ctx.clone())?;

        let port = channel.port1;
        let channel = channel.port2.borrow_mut().detach(&ctx)?;

        std::thread::spawn(move || {
            work(&path, registry, channel).ok();
        });

        MessagePort::start_native(&ctx, port.clone())?;

        let this = Class::instance(
            ctx.clone(),
            WebWorker {
                port,
                onmessage: None,
            },
        )?;

        // let cloned_this = this.clone();
        // workers.push(ctx.clone(), |ctx, kill| async move {
        //     listen(ctx.clone(), kill, parent_rx, cloned_this)
        //         .await
        //         .catch(&ctx)?;
        //     Ok(())
        // });

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

    pub fn set_onmessage(&mut self, cb: Opt<Function<'js>>) {}

    pub fn terminate(&self) -> rquickjs::Result<()> {
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
        _registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
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

async fn listen<'js>(
    ctx: Ctx<'js>,
    mut kill: Shutdown,
    rx: Receiver<TransObject>,
    worker: Class<'js, WebWorker<'js>>,
) -> rquickjs::Result<()> {
    loop {
        if kill.is_killed() {
            return Ok(());
        }

        futures::select! {
            ret = rx.recv_async() => {

                let Ok(ret) = ret else {
                    // Channel closed, which means the worker thread is terminated
                    return Ok(())
                };



            }
            _ = kill => {
                return Ok(())
            }
        }
    }

    Ok(())
}
