use klaver_modules::Environ;
use klaver_util::RuntimeError;
use rquickjs::AsyncContext;

use crate::context::Context;

#[derive(Debug, Default, Clone, Copy)]
pub struct VmOptions {
    pub max_stack_size: Option<usize>,
    pub memory_limit: Option<usize>,
}

pub struct Vm {
    context: Context,
}

impl Vm {
    pub async fn new(env: &Environ, options: VmOptions) -> Result<Vm, RuntimeError> {
        let runtime = env.create_runtime().await?;

        if let Some(ss) = options.max_stack_size {
            runtime.set_max_stack_size(ss).await;
        }

        if let Some(mm) = options.memory_limit {
            runtime.set_memory_limit(mm).await;
        }

        let context = AsyncContext::full(&runtime).await?;

        env.init(&context).await?;

        klaver_task::set_promise_hook(&runtime).await;

        Ok(Vm {
            context: Context {
                context,
                env: env.clone(),
            },
        })
    }

    pub async fn create_context(&self) -> Result<Context, RuntimeError> {
        let context = AsyncContext::full(&self.context.runtime()).await?;
        self.context.env.init(&context).await?;

        Ok(Context {
            context,
            env: self.context.env.clone(),
        })
    }
}

impl std::ops::Deref for Vm {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
