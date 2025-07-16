use futures::{SinkExt, StreamExt, channel::mpsc, future::LocalBoxFuture};
use klaver_runner::{Func, FuncFn, Runner, Shutdown, Workers};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Function, JsLifetime, Module, class::Trace,
};
use rquickjs_util::{RuntimeError, Val};

#[rquickjs::class]
pub struct WebWorker {
    sx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Val>,
}

impl<'js> Trace<'js> for WebWorker {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for WebWorker {
    type Changed<'to> = WebWorker;
}

#[rquickjs::methods]
impl WebWorker {
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, path: String) -> rquickjs::Result<WebWorker> {
        let (work_sx, work_rx) = mpsc::channel(1);
        let (parent_sx, parent_rx) = mpsc::channel(1);

        let workers = Workers::from_ctx(&ctx)?;

        std::thread::spawn(move || {
            work(&path, work_rx, parent_sx);
        });

        workers.push(ctx.clone(), |ctx, kill| async move {
            listen(ctx.clone(), kill).await.catch(&ctx)?;
            Ok(())
        });

        Ok(WebWorker {
            sx: work_sx,
            rx: parent_rx,
        })
    }

    #[qjs(rename = "postMessage")]
    pub fn post_message<'js>(&self, ctx: Ctx<'js>, msg: Val) -> rquickjs::Result<()> {
        let mut sx = self.sx.clone();
        ctx.spawn(async move {
            sx.send(Message::Event(msg)).await;
        });
        Ok(())
    }

    pub fn terminate(&self) -> rquickjs::Result<()> {
        Ok(())
    }
}

async fn listen<'js>(ctx: Ctx<'js>, kill: Shutdown) -> rquickjs::Result<()> {
    Ok(())
}

fn work(
    path: &str,
    rx: mpsc::Receiver<Message>,
    sx: mpsc::Sender<Val>,
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
                            let mut sx = sx.clone();
                            ctx.spawn(async move {
                                sx.send(msg).await.ok();
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
                // sx,
                rx,
            },
        );

        Ok(())
    })
}

struct Work {
    path: String,
    // sx: mpsc::Sender<Val>,
    rx: mpsc::Receiver<Message>,
}

impl Func for Work {
    type Future<'js> = LocalBoxFuture<'js, Result<(), RuntimeError>>;

    fn call<'js>(mut self, ctx: Ctx<'js>, worker: klaver_runner::Workers) -> Self::Future<'js> {
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
                        val = self.rx.next() => {
                            let Some(val) = val else {
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
