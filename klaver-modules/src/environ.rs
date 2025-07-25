use std::sync::{Arc, Weak};

use klaver_util::{throw, RuntimeError};
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Ctx, JsLifetime};

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

    pub fn modules(&self) -> &Modules {
        &self.0.modules
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
          self.0.globals.attach(ctx.clone()).await.catch(&ctx)?;
          ctx.store_userdata(self.downgrade()).map_err(|err| RuntimeError::Custom(Box::from(err.to_string())))?;
          Result::<_, RuntimeError>::Ok(())
        })
        .await?;

        Ok(())
    }

    pub fn downgrade(&self) -> WeakEnviron {
        WeakEnviron(Arc::downgrade(&self.0))
    }
}

#[derive(Clone)]
pub struct WeakEnviron(Weak<Inner>);

unsafe impl<'js> JsLifetime<'js> for WeakEnviron {
    type Changed<'to> = WeakEnviron;
}

impl WeakEnviron {
    pub fn upgrade<'js>(self, ctx: &Ctx<'js>) -> rquickjs::Result<Environ> {
        match self.0.upgrade() {
            Some(ret) => Ok(Environ(ret)),
            None => throw!(ctx, "Could not upgrade environment"),
        }
    }
}
