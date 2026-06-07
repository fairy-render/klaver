use klaver_base::Channel;
use klaver_core::throw;
use klaver_modules::Environ;
use klaver_runtime::{Resource, ResourceId, ResourceKind, Runner};
use klaver_vm::{Vm, VmOptions};

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
}

impl<'js> Resource<'js> for WorkerResource {
    type Id = WorkerResourceId;

    const INTERNAL: bool = true;
    const SCOPED: bool = false;

    fn run(self, ctx: klaver_runtime::Context<'js>) -> impl Future<Output = rquickjs::Result<()>> {
        async move {
            let worker = throw!(ctx.ctx(), Vm::new(&self.env, VmOptions::default()).await);

            // worker.run_module(&self.path).await?;

            Ok(())
        }
    }
}

struct ResourceWorker {}

impl<'js> Runner<'js> for ResourceWorker {
    type Output = ();

    fn run(
        self,
        ctx: klaver_runtime::Context<'js>,
    ) -> impl Future<Output = rquickjs::Result<Self::Output>> {
        async move { Ok(()) }
    }
}
