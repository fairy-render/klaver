use rquickjs::Ctx;

use crate::global_info::DynamicGlobal;

/// Globals is a struct that contains the globals for the modules.
/// It is used to attach the globals to the context.
pub struct Globals {
    globals: Vec<Box<dyn DynamicGlobal + Send + Sync>>,
}

impl Globals {
    pub(crate) fn new(globals: Vec<Box<dyn DynamicGlobal + Send + Sync>>) -> Globals {
        Globals { globals }
    }
}

impl Globals {
    pub async fn attach<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        for global in &self.globals {
            global.define(ctx.clone()).await?;
        }

        Ok(())
    }
}
