use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use rquickjs::{runtime::MemoryUsage, Ctx};
use rquickjs_modules::Environ;
use rquickjs_util::RuntimeError;
use tokio::sync::{mpsc, oneshot};

use crate::Vm;

pub type WithAsyncFn = Box<
    dyn for<'a> Fn(
            Ctx<'a>,
        ) -> Pin<
            Box<dyn Future<Output = Result<Box<dyn Any + Send>, RuntimeError>> + 'a + Send>,
        > + Send,
>;

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

pub struct Worker {
    sx: mpsc::Sender<Request>,
}

impl Worker {
    pub async fn new_with(
        modules: Arc<Environ>,
        max_stack_size: Option<usize>,
        memory_limit: Option<usize>,
    ) -> Result<Worker, RuntimeError> {
        let sx = create_worker(modules, max_stack_size, memory_limit, false).await?;
        Ok(Worker { sx })
    }

    pub async fn idle(&self) -> Result<(), RuntimeError> {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send(Request::Idle { returns: sx })
            .await
            .map_err(|err| RuntimeError::Message(Some(err.to_string())))?;

        rx.await
            .map_err(|err| RuntimeError::Custom(Box::new(err)))?
    }

    // pub async fn new(options: VmOptions) -> Result<Worker, RuntimeError> {
    //     Self::new_with(
    //         options.modules.build()?,
    //         options.max_stack_size,
    //         options.memory_limit,
    //     )
    //     .await
    // }

    pub async fn run_gc(&self) {
        self.sx
            .send(Request::RunGc)
            .await
            .map_err(|err| RuntimeError::Message(Some(err.to_string())))
            .ok();
    }

    pub async fn memory_usage(&self) -> Result<MemoryUsage, RuntimeError> {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send(Request::MemoryUsage { returns: sx })
            .await
            .map_err(|err| RuntimeError::Message(Some(err.to_string())))?;

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
            .send(Request::With {
                func: Box::new(move |ctx| {
                    let ret = func(ctx)?;
                    Ok(Box::new(ret))
                }),
                returns: sx,
            })
            .await
            .map_err(|err| RuntimeError::Message(Some(err.to_string())))?;

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
            std::mem::transmute(func)
        }

        let func = unsafe { lift(func) };

        self.sx
            .send(Request::WithAsync {
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
            .map_err(|err| RuntimeError::Message(Some(err.to_string())))?;

        let ret = rx
            .await
            .map_err(|err| RuntimeError::Custom(Box::new(err)))??;

        Ok(*ret
            .downcast()
            .map_err(|_| RuntimeError::Custom("type RuntimeError".into()))?)
    }
}

async fn create_worker(
    modules: Arc<Environ>,
    max_stack_size: Option<usize>,
    memory_limit: Option<usize>,
    drive: bool,
) -> Result<mpsc::Sender<Request>, RuntimeError> {
    let (set_ready, ready) = oneshot::channel();

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(ret) => ret,
            Err(err) => {
                set_ready.send(Err(RuntimeError::Custom(err.into()))).ok();
                return;
            }
        };

        let (sx, mut rx) = mpsc::channel(10);

        runtime.block_on(async move {
            let vm = match Vm::new_with(&*modules, max_stack_size, memory_limit).await {
                Ok(vm) => vm,
                Err(err) => {
                    set_ready.send(Err(err)).expect("send");
                    return;
                }
            };

            set_ready.send(Ok(sx)).expect("send");

            if drive {
                let mut idle = vm.idle();
                loop {
                    tokio::select! {
                        biased;
                        next = rx.recv() => {
                           let Some(next) = next else {
                            break;
                           };
                           process(&vm, next).await;
                        }
                        _ = &mut idle => {
                            break
                        }
                    };
                }
            } else {
                while let Some(next) = rx.recv().await {
                    process(&vm, next).await;
                }
            }
        });
    });

    ready.await.map_err(|_| RuntimeError::Message(None))?
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
            vm.run_gc().await;
        }
        Request::MemoryUsage { returns } => {
            let ret = vm.memory_usage().await;
            returns.send(ret).ok();
        }
        Request::Idle { returns } => {
            returns.send(vm.idle().await).ok();
        }
    }
}
