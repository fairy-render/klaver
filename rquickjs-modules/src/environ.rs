use std::sync::Arc;

use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt};
use rquickjs_util::RuntimeError;

use crate::{globals::Globals, Modules, Typings};

struct Inner {
    pub(crate) modules: Modules,
    pub(crate) globals: Globals,
    pub(crate) typings: Typings,
}

#[derive(Clone)]
pub struct Environ(Arc<Inner>);

impl Environ {
    pub fn new(modules: Modules, globals: Globals, typings: Typings) -> Environ {
        Environ(Arc::new(Inner {
            modules,
            globals,
            typings,
        }))
    }

    pub fn typings(&self) -> &Typings {
        &self.0.typings
    }

    pub async fn create_runtime(&self) -> Result<AsyncRuntime, RuntimeError> {
        let runtime = AsyncRuntime::new()?;

        self.0.modules.attach(&runtime).await?;

        Ok(runtime)
    }

    pub async fn init(&self, context: &AsyncContext) -> Result<(), RuntimeError> {
        rquickjs::async_with!(context => |ctx| {
          self.0.globals.attach(ctx.clone()).await.catch(&ctx).map_err(|err| RuntimeError::from(err))
        })
        .await?;

        Ok(())
    }
}
