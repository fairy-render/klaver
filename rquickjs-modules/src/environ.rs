use rquickjs::{AsyncContext, CatchResultExt};
use rquickjs_util::RuntimeError;

use crate::{globals::Globals, Modules, Typings};

pub struct Environ {
    pub(crate) modules: Modules,
    pub(crate) globals: Globals,
    pub(crate) typings: Typings,
}

impl Environ {
    pub fn typings(&self) -> &Typings {
        &self.typings
    }

    pub async fn init(&self, context: &AsyncContext) -> Result<(), RuntimeError> {
        self.modules.attach(context.runtime()).await?;

        rquickjs::async_with!(context => |ctx| {
          self.globals.attach(ctx.clone()).await.catch(&ctx).map_err(|err| RuntimeError::from(err))
        })
        .await?;

        Ok(())
    }
}
