use std::{any::Any, pin::Pin, sync::Arc};

use futures::{lock::Mutex, Future};
use rquickjs::{AsyncContext, AsyncRuntime, Ctx};

use crate::{Error, Persistence, WithAsyncFn, Worker};

pub type Pool = deadpool::managed::Pool<Manager>;

pub type PoolError = deadpool::managed::PoolError<Error>;

pub type CustomizeFn = Box<
    dyn for<'a> Fn(
            &'a AsyncRuntime,
            &'a AsyncContext,
        ) -> Pin<Box<dyn Future<Output = Result<(), rquickjs::Error>> + 'a>>
        + Send,
>;

pub struct Manager {
    init: Option<Arc<Mutex<WithAsyncFn>>>,
    customize: Option<Arc<Mutex<CustomizeFn>>>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            init: None,
            customize: None,
        }
    }

    pub fn new_with<T>(init: T) -> Manager
    where
        T: Send + Sync + Clone,
        for<'js> T: Fn(
                Ctx<'js>,
                Persistence,
            )
                -> Pin<Box<dyn Future<Output = Result<(), rquickjs::Error>> + 'js + Send>>
            + 'js,
    {
        Manager {
            init: Some(Arc::new(Mutex::new(Box::new(move |ctx, p| {
                let init = init.clone();
                Box::pin(async move {
                    let future = init(ctx, p);
                    let ret = future.await?;
                    Ok(Box::new(ret) as Box<dyn Any + Send>)
                })
            })))),
            customize: None,
        }
    }

    pub fn new_with_customize<C, T>(customize: C, init: T) -> Manager
    where
        C: Clone + Send + 'static,
        for<'js> C: Fn(
            &'js AsyncRuntime,
            &'js AsyncContext,
        ) -> Pin<Box<dyn Future<Output = Result<(), rquickjs::Error>> + 'js>>,
        T: Send + Sync + Clone,
        for<'js> T: Fn(
                Ctx<'js>,
                Persistence,
            )
                -> Pin<Box<dyn Future<Output = Result<(), rquickjs::Error>> + 'js + Send>>
            + 'js,
    {
        Manager {
            init: Some(Arc::new(Mutex::new(Box::new(move |ctx, p| {
                let init = init.clone();
                Box::pin(async move {
                    let future = init(ctx, p);
                    let ret = future.await?;
                    Ok(Box::new(ret) as Box<dyn Any + Send>)
                })
            })))),
            customize: Some(Arc::new(Mutex::new(Box::new(move |ctx, p| {
                let init = customize.clone();
                Box::pin(async move {
                    let future = init(ctx, p);
                    future.await?;
                    Ok(())
                })
            })))),
        }
    }
}

impl deadpool::managed::Manager for Manager {
    type Type = Worker;

    type Error = Error;

    fn create(
        &self,
    ) -> impl futures::prelude::Future<Output = Result<Self::Type, Self::Error>> + Send {
        async move {
            let worker = Worker::new().await?;

            if let Some(custom) = self.customize.as_ref() {
                let lock = custom.clone();

                worker
                    .customize(|runtime, ctx| {
                        Box::pin(async move {
                            let lock = lock.lock().await;
                            (lock)(runtime, ctx).await
                        })
                    })
                    .await?;
            }

            if let Some(init) = self.init.as_ref() {
                let lock = init.clone();

                worker
                    .with_async(|ctx, p| {
                        Box::pin(async move {
                            let lock = lock.lock().await;
                            (lock)(ctx, p).await
                        })
                    })
                    .await?;
            }

            Ok(worker)
        }
    }

    fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &deadpool::managed::Metrics,
    ) -> impl futures::prelude::Future<Output = deadpool::managed::RecycleResult<Self::Error>> + Send
    {
        async move { Ok(()) }
    }
}

#[cfg(feature = "send")]
pub struct SendManager {
    init: Option<
        Box<
            dyn for<'a> Fn(
                    Ctx<'a>,
                )
                    -> Pin<Box<dyn Future<Output = Result<(), rquickjs::Error>> + 'a + Send>>
                + Send
                + Sync,
        >,
    >,
    runtime: rquickjs::AsyncRuntime,
}

#[cfg(feature = "send")]
impl SendManager {
    pub fn new() -> Result<SendManager, rquickjs::Error> {
        let runtime = rquickjs::AsyncRuntime::new()?;

        Ok(SendManager {
            init: None,
            runtime,
        })
    }

    pub fn new_with(runtime: rquickjs::AsyncRuntime) -> Result<SendManager, rquickjs::Error> {
        Ok(SendManager {
            init: None,
            runtime,
        })
    }

    pub fn with<T>(mut self, fun: T) -> Self
    where
        T: Send + Sync,
        for<'js> T: Fn(Ctx<'js>) -> Pin<Box<dyn Future<Output = Result<(), rquickjs::Error>> + 'js + Send>>
            + Send
            + 'js,
    {
        self.init = Some(Box::new(fun));
        self
    }
}

#[cfg(feature = "send")]
impl deadpool::managed::Manager for SendManager {
    type Type = rquickjs::AsyncContext;

    type Error = Error;

    fn create(
        &self,
    ) -> impl futures::prelude::Future<Output = Result<Self::Type, Self::Error>> + Send {
        async move {
            let ctx = rquickjs::AsyncContext::full(&self.runtime).await?;
            if let Some(init) = &self.init {
                ctx.async_with(init).await?;
            }
            Ok(ctx)
        }
    }

    fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &deadpool::managed::Metrics,
    ) -> impl futures::prelude::Future<Output = deadpool::managed::RecycleResult<Self::Error>> + Send
    {
        async move { Ok(()) }
    }
}

#[cfg(feature = "send")]
pub type SendPool = deadpool::managed::Pool<SendManager>;
