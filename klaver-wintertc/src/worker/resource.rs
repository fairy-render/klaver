use std::sync::Arc;

use crate::{
    Backend, WinterTcInstance,
    channel::{Channel, MessagePort},
};
use klaver_core::{Registry, throw_if, value::structured_clone::SerializationOptions};
use klaver_modules::Environ;
use klaver_runtime::{Resource, ResourceId, Runner};
use klaver_vm::{VmOptions, Worker};
use rquickjs::{Class, Function, Module, Value};

// TODO: We should get the runtime from the `klaver_vm::Worker` itself, but that requires a change to the `Worker` API to expose it. For now, we just pick one of the two runtimes based on which feature is enabled.
#[cfg(not(any(feature = "tokio", feature = "compio")))]
compile_error!(
    "the `worker` feature requires either the `tokio` or `compio` feature, to supply an async \
     runtime the spawned worker thread can block on"
);

/// The async runtime the worker thread's event loop blocks on. When both `tokio` and `compio`
/// are enabled, `tokio` wins (matching the precedence `klaver-wintertc/src/backend.rs` and
/// `klaver::Builder` use elsewhere for picking a single `Backend`).
#[cfg(feature = "tokio")]
fn worker_runtime() -> klaver_vm::ToktioRuntime {
    klaver_vm::ToktioRuntime
}

#[cfg(all(feature = "compio", not(feature = "tokio")))]
fn worker_runtime() -> klaver_vm::CompioRuntime {
    klaver_vm::CompioRuntime
}

pub struct WorkerResourceId {}

impl ResourceId for WorkerResourceId {
    fn name() -> &'static str {
        "WorkerThread"
    }
}

pub struct WorkerResource {
    path: String,
    env: Environ,
    channel: Channel,
    registry: Registry,
    backend: Arc<dyn Backend + Send + Sync>,
}

impl<'a> WorkerResource {
    pub fn new(
        path: String,
        env: Environ,
        channel: Channel,
        registry: Registry,
        backend: Arc<dyn Backend + Send + Sync>,
    ) -> Self {
        Self {
            path,
            env,
            channel,
            registry,
            backend,
        }
    }
}

impl<'js> Resource<'js> for WorkerResource {
    type Id = WorkerResourceId;

    const INTERNAL: bool = true;
    const SCOPED: bool = false;

    fn run(self, ctx: klaver_runtime::Context<'js>) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            let worker = throw_if!(
                ctx.ctx(),
                Worker::new(self.env, VmOptions::default(), worker_runtime()).await
            );
            let registry = self.registry;
            let channel = self.channel;
            let error_channel = channel.clone();
            let backend = self.backend;
            let ret = worker
                .async_with(async move |ctx| {
                    registry.attach(&ctx)?;
                    WinterTcInstance::from_ctx(&ctx)?
                        .borrow_mut()
                        .set_backend(&ctx, backend)?;
                    Ok(())
                })
                .await;
            throw_if!(ctx.ctx(), ret);

            let ret = worker
                .run(WorkResourceRunner {
                    path: self.path,
                    channel,
                })
                .await;

            // A failure here - a syntax error, or an uncaught exception during the worker
            // script's top-level evaluation - is reported to the creating side as an
            // `"error"`-typed event on the `Worker` (see `WebWorker::onerror`), matching the
            // spec's `AbstractWorker.onerror`, rather than propagating into this resource's own
            // task and poisoning the whole host runtime's shared exception state.
            if let Err(err) = ret {
                let message = rquickjs::String::from_str(ctx.ctx().clone(), &err.to_string())?;
                let payload = Registry::instance(ctx.ctx())?.serialize(
                    ctx.ctx(),
                    &message.into_value(),
                    &SerializationOptions::default(),
                )?;
                error_channel.send_error(payload);
            }

            Ok(())
        }
    }
}

struct WorkResourceRunner {
    path: String,
    channel: Channel,
}

impl<'js> Runner<'js> for WorkResourceRunner {
    type Output = ();

    async fn run(self, ctx: klaver_runtime::Context<'js>) -> rquickjs::Result<Self::Output> {
        let messageport = MessagePort::from_channel(self.channel);
        let messageport = Class::instance(ctx.ctx().clone(), messageport)?;

        MessagePort::start_native(&ctx, messageport.clone())?;

        let init: Function = ctx.eval(include_str!("./init.js"))?;

        init.call::<_, Value>((messageport, ctx.globals()))?;

        Module::import(&ctx, &*self.path)?
            .into_future::<()>()
            .await?;

        Ok(())
    }
}
