use klaver_base::{Channel, MessagePort};
use klaver_core::{Registry, throw_if};
use klaver_modules::Environ;
use klaver_runtime::{Resource, ResourceId, Runner};
use klaver_vm::{VmOptions, Worker};
use rquickjs::{Class, Function, Module, Value};

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
}

impl<'a> WorkerResource {
    pub fn new(path: String, env: Environ, channel: Channel, registry: Registry) -> Self {
        Self {
            path,
            env,
            channel,
            registry,
        }
    }
}

impl<'js> Resource<'js> for WorkerResource {
    type Id = WorkerResourceId;

    const INTERNAL: bool = true;
    const SCOPED: bool = false;

    fn run(self, ctx: klaver_runtime::Context<'js>) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            let worker = throw_if!(ctx.ctx(), Worker::new(self.env, VmOptions::default()).await);
            let registry = self.registry;
            let channel = self.channel;
            let ret = worker
                .async_with(async move |ctx| {
                    registry.attach(&ctx)?;

                    Ok(())
                })
                .await;
            throw_if!(ctx.ctx(), ret);
            throw_if!(
                ctx.ctx(),
                worker
                    .run(WorkResourceRunner {
                        path: self.path,
                        channel,
                    })
                    .await
            );
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
