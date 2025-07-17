use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt, channel::mpsc, future::LocalBoxFuture};
use klaver_base::{Emitter, Event, EventList, NativeEvent};
use klaver_runner::{Func, Runner, Shutdown, Workers};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, FromJs, Function, JsLifetime, Module,
    String, Value, class::Trace, prelude::Opt,
};
use rquickjs_util::{RuntimeError, StringRef, Subclass, Val};

#[rquickjs::class]
pub struct WebWorker<'js> {
    sx: Sender<Message>,
    listeners: EventList<'js>,
}

impl<'js> Trace<'js> for WebWorker<'js> {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.listeners.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for WebWorker<'js> {
    type Changed<'to> = WebWorker;
}

#[rquickjs::methods]
impl<'js> WebWorker<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, path: String) -> rquickjs::Result<Class<'js, WebWorker<'js>>> {
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

fn work(
    path: &str,
    rx: flume::Receiver<Message>,
    sx: flume::Sender<Val>,
) -> Result<(), RuntimeError> {
    futures::executor::block_on(async move {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        context
            .with(move |ctx| {
                ctx.globals().set(
                    "postMessage",
                    rquickjs::prelude::Func::from(rquickjs::function::MutFn::new(
                        move |ctx: Ctx<'_>, msg: Val| {
                            let sx = sx.clone();
                            ctx.spawn(async move {
                                sx.send_async(msg).await.ok();
                            });
                            rquickjs::Result::Ok(())
                        },
                    )),
                )
            })
            .await?;

        Runner::new(
            &context,
            Work {
                path: path.to_string(),
                rx,
            },
        );

        Ok(())
    })
}

struct Work {
    path: std::string::String,
    rx: flume::Receiver<Message>,
}

impl Func for Work {
    type Future<'js> = LocalBoxFuture<'js, Result<(), RuntimeError>>;

    fn call<'js>(self, ctx: Ctx<'js>, worker: klaver_runner::Workers) -> Self::Future<'js> {
        Box::pin(async move {
            worker.push(ctx.clone(), |ctx, mut shutdown| async move {
                //

                let trigger = ctx
                    .globals()
                    .get::<_, Function>("__triggerMessage")
                    .catch(&ctx)?;

                loop {
                    futures::select! {
                        _ = shutdown => {
                            break
                        }
                        val = self.rx.recv_async() => {
                            let Ok(val) = val else {
                                break;
                            };

                            match val {
                                Message::Event(val) => {
                                    trigger.call::<_, ()>((val,)).catch(&ctx)?;
                                }
                                Message::Kill => {
                                    break
                                }
                            }
                        }
                    }
                }

                Ok(())
            });

            Module::import(&ctx, self.path)
                .catch(&ctx)?
                .into_future::<()>()
                .await?;

            Ok(())
        })
    }
}

enum Message {
    Kill,
    Event(Val),
}

#[derive(Debug, Trace, JsLifetime)]
#[rquickjs::class]
pub struct MessageEvent<'js> {
    ty: String<'js>,
    #[qjs(get)]
    data: Option<Value<'js>>,
}

pub struct MessageEventOptions<'js> {
    data: Option<Value<'js>>,
}

impl<'js> FromJs<'js> for MessageEventOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        todo!()
    }
}

#[rquickjs::methods]
impl<'js> MessageEvent<'js> {
    pub fn new(
        ty: String<'js>,
        ops: Opt<MessageEventOptions<'js>>,
    ) -> rquickjs::Result<MessageEvent<'js>> {
        Ok(MessageEvent { data: None, ty })
    }
}

impl<'js> NativeEvent<'js> for MessageEvent<'js> {
    fn ty(
        this: rquickjs::prelude::This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<String<'js>> {
        Ok(this.borrow().ty.clone())
    }
}

impl<'js> Subclass<'js, Event<'js>> for MessageEvent<'js> {}
