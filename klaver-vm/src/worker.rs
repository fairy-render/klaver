use std::{any::Any, pin::Pin};

use futures::channel::oneshot;
use klaver_core::RuntimeError;
use klaver_modules::Environ;
use klaver_runtime::Runner;
use rquickjs::{Ctx, FromJs};

use crate::{Vm, VmOptions};

enum Request {
    WithAsync {
        function: Box<
            dyn for<'js> FnOnce(
                    Ctx<'js>,
                ) -> Pin<
                    Box<dyn Future<Output = Result<Box<dyn Any + Send>, RuntimeError>> + 'js>,
                > + Send,
        >,
        returns: oneshot::Sender<Result<Box<dyn Any + Send>, RuntimeError>>,
    },
    WithVm {
        function: Box<
            dyn for<'js> FnOnce(
                    &'js Vm,
                ) -> Pin<
                    Box<dyn Future<Output = Result<Box<dyn Any + Send>, RuntimeError>> + 'js>,
                > + Send,
        >,
        returns: oneshot::Sender<Result<Box<dyn Any + Send>, RuntimeError>>,
    },
}

pub struct Worker {
    sx: flume::Sender<Request>,
}

impl Worker {
    pub async fn new(environ: Environ, options: VmOptions) -> Result<Self, RuntimeError> {
        let (sx, rx) = oneshot::channel();

        create_worker_thread(environ, options, sx);

        let worker_sx = rx
            .await
            .map_err(|_| RuntimeError::new("Worker thread panicked"))??;

        Ok(Self { sx: worker_sx })
    }

    pub async fn async_with<'a, F, R>(&self, f: F) -> Result<R, RuntimeError>
    where
        F: for<'js> AsyncFnOnce(Ctx<'js>) -> Result<R, RuntimeError> + Send + 'static,
        R: Send + 'static,
    {
        let (sx, rx) = oneshot::channel();

        let req = Request::WithAsync {
            function: Box::new(move |ctx| {
                Box::pin(async move {
                    let ret = f(ctx).await?;
                    Ok(Box::new(ret) as Box<dyn Any + Send>)
                })
            }),
            returns: sx,
        };

        self.sx
            .send_async(req)
            .await
            .map_err(|err| RuntimeError::new(err.to_string()))?;

        let ret = rx
            .await
            .map_err(|_| RuntimeError::new("Worker dropped"))??;

        let downcasted = ret
            .downcast::<R>()
            .map_err(|_| RuntimeError::new("Worker returned wrong type"))?;

        Ok(*downcasted)
    }

    pub async fn run<T, R>(&self, task: T) -> Result<R, RuntimeError>
    where
        T: Send + 'static,
        T: for<'js> Runner<'js, Output = R>,
        R: for<'js> FromJs<'js>,
        R: 'static + Send,
    {
        self.exec(async |vm| vm.run(task).await).await
    }

    pub async fn run_module(&self, module: &str) -> Result<(), RuntimeError> {
        let module = module.to_string();
        self.exec(async move |vm| vm.run_module(&module).await)
            .await
    }

    async fn exec<F, R>(&self, f: F) -> Result<R, RuntimeError>
    where
        F: for<'js> AsyncFnOnce(&'js Vm) -> Result<R, RuntimeError> + Send + 'static,
        R: Send + 'static,
    {
        let (sx, rx) = oneshot::channel();

        let req = Request::WithVm {
            function: Box::new(move |ctx| {
                Box::pin(async move {
                    let ret = f(ctx).await?;
                    Ok(Box::new(ret) as Box<dyn Any + Send>)
                })
            }),
            returns: sx,
        };

        self.sx
            .send_async(req)
            .await
            .map_err(|err| RuntimeError::new(err.to_string()))?;

        let ret = rx
            .await
            .map_err(|_| RuntimeError::new("Worker dropped"))??;

        let downcasted = ret
            .downcast::<R>()
            .map_err(|_| RuntimeError::new("Worker returned wrong type"))?;

        Ok(*downcasted)
    }
}

fn create_worker_thread(
    environ: Environ,
    options: VmOptions,
    sx: oneshot::Sender<Result<flume::Sender<Request>, RuntimeError>>,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let (worker_sx, worker_rx) = flume::bounded(10);

            let vm = match Vm::new(&environ, options).await {
                Ok(ret) => {
                    let _ = sx.send(Ok(worker_sx));
                    ret
                }
                Err(err) => {
                    let _ = sx.send(Err(err));
                    return;
                }
            };

            worker(worker_rx, vm).await.unwrap();
        });
    });
}

async fn worker(rx: flume::Receiver<Request>, vm: Vm) -> Result<(), RuntimeError> {
    while let Ok(req) = rx.recv_async().await {
        match req {
            Request::WithAsync { function, returns } => {
                let ret = vm.async_with(async |ctx| (function)(ctx).await).await;
                let _ = returns.send(ret);
            }
            Request::WithVm { function, returns } => {
                let ret = function(&vm).await;
                let _ = returns.send(ret);
            }
        }
    }

    Ok(())
}
