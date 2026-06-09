use rquickjs::{Ctx, Module, loader::ImportAttributes};
use std::sync::{Arc, Mutex};

/// Loader is a trait that defines the interface for loading modules.
/// Contrary to rquickjs's Loader, self is not mutable, and it is expected to be thread safe.
/// This is because the loader will be shared across multiple runtimes, and it should be able to handle concurrent requests.
pub trait Loader {
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>>;
}

/// Resolver is a trait that defines the interface for resolving module paths.
/// It is used to resolve the module paths before they are loaded by the loader.
/// Contrary to rquickjs's Resolver, self is not mutable, and it is expected to be thread safe.
/// This is because the resolver will be shared across multiple runtimes, and it should be able to handle concurrent requests.
pub trait Resolver {
    fn resolve<'js>(
        &self,
        ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> rquickjs::Result<String>;
}

/// QuickWrap is a wrapper around a Loader or Resolver that allows it to be shared across multiple runtimes.
#[derive(Debug)]
pub struct QuickWrap<T>(Mutex<T>);

impl<T> QuickWrap<T> {
    pub fn new(item: T) -> QuickWrap<T> {
        QuickWrap(Mutex::new(item))
    }
}

impl<T> Loader for QuickWrap<T>
where
    T: rquickjs::loader::Loader,
{
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        self.0.lock().unwrap().load(ctx, path, attributes)
    }
}

impl<T> Resolver for QuickWrap<T>
where
    T: rquickjs::loader::Resolver,
{
    fn resolve<'js>(
        &self,
        ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> rquickjs::Result<String> {
        self.0.lock().unwrap().resolve(ctx, base, name, attributes)
    }
}

/**
 * Modules is a struct that contains the resolvers and loaders for the module system.
 * It also contains an optional transformer that can be used to transform the code before it is executed.
 * The Modules struct is used to attach the resolvers and loaders to the runtime,
 * and it is also used to store the transformer for later use.
 */
#[derive(Clone)]
pub struct ModuleLoader(Arc<ModulesInner>);

struct ModulesInner {
    resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
    loaders: Vec<Box<dyn Loader + Send + Sync>>,
}

impl ModuleLoader {
    pub fn new(
        resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
        loaders: Vec<Box<dyn Loader + Send + Sync>>,
    ) -> ModuleLoader {
        ModuleLoader(Arc::new(ModulesInner { resolvers, loaders }))
    }
}

impl ModuleLoader {
    pub async fn attach<T: internal::Runtime>(&self, runtime: &T) -> rquickjs::Result<()> {
        runtime.set_loader(self.clone(), self.clone()).await;
        Ok(())
    }
}

impl rquickjs::loader::Loader for ModuleLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let mut error = None;
        for loader in self.0.loaders.iter() {
            match loader.load(ctx, name, attributes.clone()) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    error = Some(err);
                }
            }
        }

        Err(error.unwrap_or_else(|| rquickjs::Error::new_loading(name)))
    }
}

impl rquickjs::loader::Resolver for ModuleLoader {
    fn resolve<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> rquickjs::Result<String> {
        for resolver in self.0.resolvers.iter() {
            if let Ok(ret) = resolver.resolve(ctx, base, name, attributes.clone()) {
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
