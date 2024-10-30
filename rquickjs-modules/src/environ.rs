use rquickjs::{AsyncContext, Context};

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

    pub async fn init(&self, context: &AsyncContext) -> rquickjs::Result<()> {
        self.modules.attach(context.runtime()).await?;

        rquickjs::async_with!(context => |ctx| {
          self.globals.attach(ctx).await
        })
        .await?;

        Ok(())
    }
}
