use std::{any::Any, future::Future, pin::Pin};

use futures::channel::oneshot;
use klaver_modules::Environ;
use klaver_util::RuntimeError;
use rquickjs::{Ctx, runtime::MemoryUsage};

use crate::{Vm, VmOptions};

pub trait WorkerRuntime {
    fn block_on<T>(self, future: T) -> T::Output
    where
        T: Future;
}

enum Request {
    With {
        func: Box<dyn for<'a> FnOnce(Ctx<'a>) -> Result<Box<dyn Any + Send>, RuntimeError> + Send>,
        returns: oneshot::Sender<Result<Box<dyn Any + Send>, RuntimeError>>,
    },
    WithAsync {
        func: Box<
            dyn for<'a> FnOnce(
                    Ctx<'a>,
                ) -> Pin<
                    Box<dyn Future<Output = Result<Box<dyn Any + Send>, RuntimeError>> + 'a + Send>,
                > + Send,
        >,
        returns: oneshot::Sender<Result<Box<dyn Any + Send>, RuntimeError>>,
    },
    RunGc,
    MemoryUsage {
        returns: oneshot::Sender<MemoryUsage>,
    },
    Idle {
        returns: oneshot::Sender<Result<(), RuntimeError>>,
    },
}

#[derive(Clone)]
pub struct Worker {
    sx: flume::Sender<Request>,
    env: Environ,
}

impl Worker {
    pub async fn new<T: WorkerRuntime + Send + 'static>(
        modules: Environ,
        options: VmOptions,
        runtime: T,
    ) -> Result<Worker, RuntimeError> {
        let sx = create_worker(modules.clone(), options, runtime, false).await?;
        Ok(Worker { sx, env: modules })
    }

    pub fn env(&self) -> &Environ {
        &self.env
    }

    pub async fn idle(&self) -> Result<(), RuntimeError> {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send_async(Request::Idle { returns: sx })
            .await
            .map_err(|err| RuntimeError::new(err.to_string()))?;

        rx.await
            .map_err(|err| RuntimeError::Custom(Box::new(err)))?
    }

    pub async fn run_gc(&self) {
        self.sx
            .send_async(Request::RunGc)
            .await
            .map_err(|err| RuntimeError::new(err.to_string()))
            .ok();
    }

    pub async fn memory_usage(&self) -> Result<MemoryUsage, RuntimeError> {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send_async(Request::MemoryUsage { returns: sx })
            .await
            .map_err(|err| RuntimeError::new(err.to_string()))?;

        let ret = rx
            .await
            .map_err(|err| RuntimeError::Custom(Box::new(err)))?;

        Ok(ret)
    }

    pub async fn with<T, R>(&self, func: T) -> Result<R, RuntimeError>
    where
        T: Send + 'static,
        for<'js> T: FnOnce(Ctx<'js>) -> Result<R, RuntimeError>,
        R: Send + 'static,
    {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send_async(Request::With {
                func: Box::new(move |ctx| {
                    let ret = func(ctx)?;
                    Ok(Box::new(ret))
                }),
                returns: sx,
            })
            .await
            .map_err(|err| RuntimeError::new(err.to_string()))?;

        let ret = rx
            .await
            .map_err(|err| RuntimeError::Custom(Box::new(err)))??;

        Ok(*ret
            .downcast()
            .map_err(|_| RuntimeError::Custom("type RuntimeError".into()))?)
    }

    pub async fn async_with<T, R>(&self, func: T) -> Result<R, RuntimeError>
    where
        T: Send,
        for<'js> T:
            FnOnce(Ctx<'js>) -> Pin<Box<dyn Future<Output = Result<R, RuntimeError>> + 'js + Send>>,
        R: Send + 'static,
    {
        let (sx, rx) = oneshot::channel();

        let func = Box::new(func)
            as Box<
                dyn for<'a> FnOnce(
                        Ctx<'a>,
                    ) -> Pin<
                        Box<dyn Future<Output = Result<R, RuntimeError>> + 'a + Send>,
                    > + Send,
            >;

        unsafe fn lift<'a, 'b, R>(
            func: Box<
                dyn for<'js> FnOnce(
                        Ctx<'js>,
                    ) -> Pin<
                        Box<dyn Future<Output = Result<R, RuntimeError>> + 'js + Send>,
                    > + Send
                    + 'a,
            >,
        ) -> Box<
            dyn for<'js> FnOnce(
                    Ctx<'js>,
                ) -> Pin<
                    Box<dyn Future<Output = Result<R, RuntimeError>> + 'js + Send>,
                > + Send
                + 'b,
        > {
            unsafe { std::mem::transmute(func) }
        }

        let func = unsafe { lift(func) };

        self.sx
            .send_async(Request::WithAsync {
                func: Box::new(move |ctx| {
                    Box::pin(async {
                        let future = func(ctx);
                        let ret = future.await?;
                        Ok(Box::new(ret) as Box<dyn Any + Send>)
                    })
                }),
                returns: sx,
            })
            .await
            .map_err(|err| RuntimeError::new(err.to_string()))?;

        let ret = rx
            .await
            .map_err(|err| RuntimeError::Custom(Box::new(err)))??;

        Ok(*ret
            .downcast()
            .map_err(|_| RuntimeError::Custom("type RuntimeError".into()))?)
    }
}

async fn create_worker<T: WorkerRuntime + Send + 'static>(
    modules: Environ,
    options: VmOptions,
    runtime: T,
    _drive: bool,
) -> Result<flume::Sender<Request>, RuntimeError> {
    let (set_ready, ready) = oneshot::channel();

    std::thread::spawn(move || {
        let (sx, rx) = flume::bounded(10);

        runtime.block_on(async move {
            let vm = match Vm::new(&modules, options).await {
                Ok(vm) => vm,
                Err(err) => {
                    set_ready.send(Err(err)).expect("send");
                    return;
                }
            };

            set_ready.send(Ok(sx)).expect("send");

            while let Ok(next) = rx.recv_async().await {
                process(&vm, next).await;
            }
            // if drive {
            //     let idle = vm.idle();
            //     pin_mut!(idle);
            //     loop {
            //         tokio::select! {
            //             biased;
            //             next = rx.recv() => {
            //                let Some(next) = next else {
            //                 break;
            //                };
            //                process(&vm, next).await;
            //             }
            //             _ = &mut idle => {
            //                 break
            //             }
            //         };
            //     }
            // } else {
            //     while let Ok(next) = rx.recv_async().await {
            //         process(&vm, next).await;
            //     }
            // }
        });
    });

    ready
        .await
        .map_err(|_| RuntimeError::new("Channel closed"))?
}

async fn process(vm: &Vm, next: Request) {
    match next {
        Request::With { func, returns } => {
            let ret = vm.with(move |ctx| func(ctx.clone())).await;
            returns.send(ret.map_err(Into::into)).ok();
        }
        Request::WithAsync { func, returns } => {
            let ret = vm.async_with(move |ctx| func(ctx.clone())).await;
            returns.send(ret.map_err(Into::into)).ok();
        }
        Request::RunGc => {
            vm.runtime().run_gc().await;
        }
        Request::MemoryUsage { returns } => {
            let ret = vm.runtime().memory_usage().await;
            returns.send(ret).ok();
        }
        Request::Idle { returns } => {
            returns.send(Ok(vm.idle().await)).ok();
        }
    }
}
