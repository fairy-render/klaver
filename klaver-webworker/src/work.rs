use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt, channel::mpsc, future::LocalBoxFuture};
use klaver_base::{Emitter, Event, EventList, NativeEvent};
use klaver_runner::{Func, Runner, Shutdown, Workers};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, FromJs, Function, JsLifetime, Module,
    String, Value, class::Trace, prelude::Opt,
};
use rquickjs_util::{RuntimeError, StringRef, Subclass, Val};

pub enum Message {
    Kill,
    Event(Val),
}

pub fn work(
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
