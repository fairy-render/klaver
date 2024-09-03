use std::{future::Future, option, pin::Pin, sync::Arc};

use deadpool::managed::{Metrics, RecycleResult};
use rquickjs::{AsyncContext, AsyncRuntime};

use crate::{
    base::init as base_init,
    modules::Modules,
    vm::{Vm, VmOptions},
    Error,
};

pub type CustomizeFn = Arc<
    dyn for<'a> Fn(&'a Vm) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type Pool = deadpool::managed::Pool<Manager>;

pub type PoolError = deadpool::managed::PoolError<Error>;

pub struct Manager {
    init: Option<CustomizeFn>,
    options: VmPoolOptions,
}

pub struct VmPoolOptions {
    pub max_stack_size: Option<usize>,
    pub memory_limit: Option<usize>,
    pub modules: Modules,
}

impl VmPoolOptions {
    pub fn from(options: VmOptions) -> Result<VmPoolOptions, Error> {
        let modules = options.modules.build()?;
        Ok(VmPoolOptions {
            max_stack_size: options.max_stack_size,
            memory_limit: options.memory_limit,
            modules,
        })
    }
}

impl Manager {
    pub fn new(options: VmOptions) -> Result<Manager, Error> {
        let modules = options.modules.build()?;

        Ok(Manager {
            init: None,
            options: VmPoolOptions {
                max_stack_size: options.max_stack_size,
                memory_limit: options.memory_limit,
                modules,
            },
        })
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
            let vm = Vm::with(
                self.options.modules.clone(),
                self.options.max_stack_size,
                self.options.memory_limit,
            )
            .await?;

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
