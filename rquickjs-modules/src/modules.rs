use rquickjs::{Ctx, Module};
use std::sync::Arc;

use crate::loader::{Loader, Resolver};

struct ModulesInner {
    resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
    loaders: Vec<Box<dyn Loader + Send + Sync>>,
    #[cfg(feature = "transform")]
    cache: crate::transformer::Cache,
}

#[derive(Clone)]
pub struct Modules(Arc<ModulesInner>);

impl Modules {
    #[cfg(not(feature = "transform"))]
    pub fn new(
        resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
        loaders: Vec<Box<dyn Loader + Send + Sync>>,
    ) -> Modules {
        Modules(Arc::new(ModulesInner { resolvers, loaders }))
    }

    #[cfg(feature = "transform")]
    pub fn new(
        cache: crate::transformer::Cache,
        resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
        loaders: Vec<Box<dyn Loader + Send + Sync>>,
    ) -> Modules {
        Modules(Arc::new(ModulesInner {
            resolvers,
            loaders,
            cache,
        }))
    }

    #[cfg(feature = "transform")]
    pub fn cache(&self) -> &crate::transformer::Cache {
        &self.0.cache
    }
}

impl Modules {
    pub async fn attach<T: internal::Runtime>(&self, runtime: &T) -> rquickjs::Result<()> {
        runtime.set_loader(self.clone(), self.clone()).await;
        Ok(())
    }
}

impl rquickjs::loader::Loader for Modules {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        for loader in self.0.loaders.iter() {
            if let Ok(ret) = loader.load(ctx, name) {
                return Ok(ret);
            }
        }

        Err(rquickjs::Error::new_loading(name))
    }
}

impl rquickjs::loader::Resolver for Modules {
    fn resolve<'js>(&mut self, ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String> {
        for resolver in self.0.resolvers.iter() {
            if let Ok(ret) = resolver.resolve(ctx, base, name) {
                return Ok(ret);
            }
        }

        Err(rquickjs::Error::new_resolving(base, name))
    }
}

#[allow(async_fn_in_trait)]
mod internal {
    pub trait Runtime {
        async fn set_loader<R, L>(&self, resolver: R, loader: L)
        where
            R: rquickjs::loader::Resolver + 'static,
            L: rquickjs::loader::Loader + 'static;
    }

    impl Runtime for rquickjs::Runtime {
        async fn set_loader<R, L>(&self, resolver: R, loader: L)
        where
            R: rquickjs::loader::Resolver + 'static,
            L: rquickjs::loader::Loader + 'static,
        {
            self.set_loader(resolver, loader)
        }
    }

    impl Runtime for rquickjs::AsyncRuntime {
        async fn set_loader<R, L>(&self, resolver: R, loader: L)
        where
            R: rquickjs::loader::Resolver + 'static,
            L: rquickjs::loader::Loader + 'static,
        {
            self.set_loader(resolver, loader).await
        }
    }
}
