use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt, channel::mpsc, future::LocalBoxFuture};
use klaver_base::{Emitter, Event, EventList, NativeEvent};
use klaver_runner::{Func, Runner, Shutdown, Workers};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, FromJs, Function, JsLifetime, Module,
    String, Value, class::Trace, prelude::Opt,
};
use rquickjs_util::{RuntimeError, StringRef, Subclass, Val};

use crate::work::{Message, work};

#[rquickjs::class]
pub struct WebWorker<'js> {
    sx: Sender<Message>,
    listeners: EventList<'js>,
}

impl<'js> Trace<'js> for WebWorker<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.listeners.trace(tracer);
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
        let (work_sx, work_rx) = flume::bounded(1);
        let (parent_sx, parent_rx) = flume::bounded(1);

        let workers = Workers::from_ctx(&ctx)?;

        std::thread::spawn(move || {
            work(&path, work_rx, parent_sx).ok();
        });

        let this = Class::instance(
            ctx.clone(),
            WebWorker {
                sx: work_sx,
                listeners: EventList::default(),
            },
        )?;

        let cloned_this = this.clone();
        workers.push(ctx.clone(), |ctx, kill| async move {
            listen(ctx.clone(), kill, parent_rx, cloned_this)
                .await
                .catch(&ctx)?;
            Ok(())
        });

        Ok(this)
    }

    #[qjs(rename = "postMessage")]
    pub fn post_message(&self, ctx: Ctx<'js>, msg: Val) -> rquickjs::Result<()> {
        let sx = self.sx.clone();
        ctx.spawn(async move {
            sx.send_async(Message::Event(msg)).await.ok();
        });

        Ok(())
    }

    pub fn terminate(&self) -> rquickjs::Result<()> {
        Ok(())
    }
}

impl<'js> Emitter<'js> for WebWorker<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listeners
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listeners
    }
}

async fn listen<'js>(
    ctx: Ctx<'js>,
    mut kill: Shutdown,
    rx: Receiver<Val>,
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
