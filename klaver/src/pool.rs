use std::{future::Future, pin::Pin, sync::Arc};

use deadpool::managed::{Metrics, RecycleResult};
use rquickjs::{AsyncContext, AsyncRuntime, Ctx};

use crate::{
    vm::{Vm, VmOptions},
    Error,
};

pub type CustomizeFn = Arc<
    dyn for<'a> Fn(&'a Vm) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type Pool = deadpool::managed::Pool<Manager>;

pub struct Manager {
    init: Option<CustomizeFn>,
    options: VmOptions,
}

impl Manager {
    pub fn new(options: VmOptions) -> Manager {
        Manager {
            init: None,
            options,
        }
    }

    pub fn init<T>(mut self, init: T) -> Self
    where
        T: Send + Sync + 'static,
        for<'a> T: Fn(&'a Vm) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>,
    {
        self.init = Some(Arc::new(init));
        self
    }
}

impl deadpool::managed::Manager for Manager {
    type Type = Vm;

    type Error = Error;

    fn create(&self) -> impl std::future::Future<Output = Result<Self::Type, Self::Error>> + Send {
        async move {
            let vm = self.options.clone().build().await?;

            if let Some(init) = &self.init {
                init(&vm).await?;
            }

            Ok(vm)
        }
    }

    fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &Metrics,
    ) -> impl std::future::Future<Output = RecycleResult<Self::Error>> + Send {
        async move { Ok(()) }
    }
}
