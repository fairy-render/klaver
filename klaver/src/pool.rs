use std::{future::Future, option, pin::Pin, sync::Arc};

use deadpool::managed::{Metrics, RecycleResult};
use rquickjs::{runtime::MemoryUsage, AsyncContext, AsyncRuntime, Ctx};
use rquickjs_modules::Environ;
use rquickjs_util::RuntimeError;

use crate::{worker::Worker, Options, Vm};

pub type CustomizeFn = Arc<
    dyn for<'a> Fn(
            &'a PooledVm,
        ) -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type Pool = deadpool::managed::Pool<Manager>;

pub type PoolError = deadpool::managed::PoolError<RuntimeError>;

pub struct Manager {
    init: Option<CustomizeFn>,
    options: VmPoolOptions,
}

pub struct VmPoolOptions {
    pub max_stack_size: Option<usize>,
    pub memory_limit: Option<usize>,
    pub modules: Environ,
    pub worker_thread: bool,
}

impl VmPoolOptions {
    pub fn from(options: Options) -> Result<VmPoolOptions, RuntimeError> {
        Ok(VmPoolOptions {
            max_stack_size: options.max_stack_size,
            memory_limit: options.memory_limit,
            modules: options.build_environ(),
            worker_thread: false,
        })
    }
}

impl Manager {
    pub fn new(options: VmPoolOptions) -> Result<Manager, RuntimeError> {
        Ok(Manager {
            init: None,
            options,
        })
    }

    pub fn use_worker_thread(mut self) -> Self {
        self.options.worker_thread = true;
        self
    }

    pub fn init<T>(mut self, init: T) -> Self
    where
        T: Send + Sync + 'static,
        for<'a> T:
            Fn(&'a PooledVm) -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + Send + 'a>>,
    {
        self.init = Some(Arc::new(init));
        self
    }
}

impl deadpool::managed::Manager for Manager {
    type Type = PooledVm;

    type Error = RuntimeError;

    fn create(&self) -> impl std::future::Future<Output = Result<Self::Type, Self::Error>> + Send {
        async move {
            let vm = if self.options.worker_thread {
                let vm = Worker::new(
                    self.options.modules.clone(),
                    self.options.max_stack_size,
                    self.options.memory_limit,
                )
                .await?;
                PooledVm::Worker(vm)
            } else {
                let vm = Vm::new_with(
                    &self.options.modules,
                    self.options.max_stack_size,
                    self.options.memory_limit,
                )
                .await?;

                PooledVm::Vm(vm)
            };

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

pub enum PooledVm {
    Vm(Vm),
    Worker(Worker),
}

impl PooledVm {
    pub async fn with<T, R>(&self, func: T) -> Result<R, RuntimeError>
    where
        T: Send + 'static,
        for<'js> T: FnOnce(Ctx<'js>) -> Result<R, RuntimeError>,
        R: Send + 'static,
    {
        match self {
            PooledVm::Vm(vm) => vm.with(func).await,
            PooledVm::Worker(worker) => worker.with(func).await,
        }
    }

    pub async fn async_with<T, R>(&self, func: T) -> Result<R, RuntimeError>
    where
        T: Send,
        for<'js> T:
            FnOnce(Ctx<'js>) -> Pin<Box<dyn Future<Output = Result<R, RuntimeError>> + 'js + Send>>,
        R: Send + 'static,
    {
        match self {
            PooledVm::Vm(vm) => vm.async_with(func).await,
            PooledVm::Worker(worker) => worker.async_with(func).await,
        }
    }

    pub async fn run_gc(&self) {
        match self {
            PooledVm::Vm(vm) => vm.run_gc().await,
            PooledVm::Worker(worker) => worker.run_gc().await,
        }
    }

    pub async fn memory_usage(&self) -> Result<MemoryUsage, RuntimeError> {
        match self {
            PooledVm::Vm(vm) => Ok(vm.memory_usage().await),
            PooledVm::Worker(vm) => vm.memory_usage().await,
        }
    }

    pub async fn idle(&self) -> Result<(), RuntimeError> {
        match self {
            PooledVm::Vm(vm) => vm.idle().await,
            PooledVm::Worker(worker) => worker.idle().await,
        }
    }

    pub fn env(&self) -> &Environ {
        match self {
            PooledVm::Vm(vm) => vm.env(),
            PooledVm::Worker(worker) => worker.env(),
        }
    }
}
