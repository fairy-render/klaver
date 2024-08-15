use std::{collections::HashMap, path::PathBuf, sync::Arc};

use parking_lot::Mutex;
use rquickjs::{
    loader::{BuiltinResolver, Loader, Resolver},
    module::ModuleDef,
    AsyncContext, Ctx, Error, Module,
};

mod file;
mod init;
mod module;
#[cfg(feature = "typescript")]
pub mod typescript;
mod util;

pub use self::{init::*, module::*};

type LoadFn = for<'js> fn(Ctx<'js>, Vec<u8>) -> rquickjs::Result<Module<'js>>;

pub(crate) trait Runtime {
    async fn set_loader<R, L>(&self, resolver: R, loader: L)
    where
        R: Resolver + 'static,
        L: Loader + 'static;
}

impl Runtime for rquickjs::Runtime {
    async fn set_loader<R, L>(&self, resolver: R, loader: L)
    where
        R: Resolver + 'static,
        L: Loader + 'static,
    {
        self.set_loader(resolver, loader)
    }
}

impl Runtime for rquickjs::AsyncRuntime {
    async fn set_loader<R, L>(&self, resolver: R, loader: L)
    where
        R: Resolver + 'static,
        L: Loader + 'static,
    {
        self.set_loader(resolver, loader).await
    }
}

#[derive(Default, Clone)]
pub struct Modules {
    pub(crate) modules: HashMap<String, LoadFn>,
    modules_src: HashMap<String, Vec<u8>>,
    search_paths: Vec<PathBuf>,
    patterns: Vec<String>,
    inits: Vec<Arc<dyn Init + Send + Sync>>,
    jsx_import_source: Option<String>,
    ts_decorators: bool,
    resolvers: ResolverList,
    loaders: LoaderList,
}

impl Modules {
    pub(crate) fn load_func<'js, D: ModuleDef>(
        ctx: Ctx<'js>,
        name: Vec<u8>,
    ) -> rquickjs::Result<Module<'js>> {
        Module::declare_def::<D, _>(ctx, name)
    }

    pub fn set_jsx_import_source(&mut self, path: &str) -> &mut Self {
        self.jsx_import_source = Some(path.to_string());
        self
    }

    pub fn use_legacy_decorators(&mut self, on: bool) -> &mut Self {
        self.ts_decorators = on;
        self
    }

    pub fn register_module<T: ModuleInfo>(&mut self) -> &mut Self {
        T::register(&mut Builder::new(self));
        self
    }

    pub fn register<T: ModuleDef>(&mut self, name: impl ToString) {
        self.modules
            .insert(name.to_string(), Modules::load_func::<T>);
    }

    pub fn register_src(&mut self, name: impl ToString, source: Vec<u8>) {
        self.modules_src.insert(name.to_string(), source);
    }

    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.search_paths.push(path.into());
        self
    }

    pub fn add_init<T>(&mut self, init: T) -> &mut Self
    where
        T: Init + Send + Sync + 'static,
    {
        self.inits.push(Arc::new(init));
        self
    }

    pub fn add_loader<T: rquickjs::loader::Loader + Send + Sync + 'static>(
        &mut self,
        loader: T,
    ) -> &mut Self {
        self.loaders.loaders.lock().push(Box::new(loader));
        self
    }

    pub fn add_resolver<T: rquickjs::loader::Resolver + Send + Sync + 'static>(
        &mut self,
        resolver: T,
    ) -> &mut Self {
        self.resolvers.resolvers.lock().push(Box::new(resolver));
        self
    }

    pub(crate) async fn attach<T: Runtime>(
        self,
        runtime: &T,
        context: &AsyncContext,
    ) -> Result<(), Error> {
        let mut builtin_resolver = BuiltinResolver::default();
        let mut file_resolver = file::FileResolver::default();
        #[cfg(feature = "typescript")]
        let script_loader = typescript::TsLoader::new(self.jsx_import_source, self.ts_decorators);
        #[cfg(feature = "typescript")]
        {
            file_resolver.add_pattern("{}.ts");
            file_resolver.add_pattern("{}.tsx");
        }
        #[cfg(not(feature = "typescript"))]
        let script_loader = rquickjs::loader::ScriptLoader::default();

        for k in self.modules.keys() {
            builtin_resolver.add_module(k);
        }

        for k in self.modules_src.keys() {
            builtin_resolver.add_module(k);
        }

        file_resolver.add_paths(self.search_paths);
        for pattern in self.patterns {
            file_resolver.add_pattern(pattern);
        }

        let module_loader = ModuleLoader {
            modules: self.modules,
            modules_src: self.modules_src,
        };

        runtime
            .set_loader(
                (builtin_resolver, file_resolver, self.resolvers.clone()),
                (script_loader, module_loader, self.loaders.clone()),
            )
            .await;

        context
            .with(|ctx| {
                for init in self.inits {
                    init.init(ctx.clone())?
                }

                rquickjs::Result::Ok(())
            })
            .await?;

        Ok(())
    }
}

struct ModuleLoader {
    modules: HashMap<String, LoadFn>,
    modules_src: HashMap<String, Vec<u8>>,
}

impl Loader for ModuleLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        if let Some(load) = self.modules.remove(path) {
            (load)(ctx.clone(), Vec::from(path))
        } else if let Some(source) = self.modules_src.remove(path) {
            Module::declare(ctx.clone(), path, source)
        } else {
            Err(Error::new_loading(path))
        }
    }
}

#[derive(Clone, Default)]
struct LoaderList {
    loaders: Arc<Mutex<Vec<Box<dyn rquickjs::loader::Loader + Send + Sync>>>>,
}

impl rquickjs::loader::Loader for LoaderList {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let mut loaders = self.loaders.lock();
        for loader in &mut *loaders {
            if let Ok(module) = loader.load(ctx, name) {
                return Ok(module);
            }
        }

        Err(rquickjs::Error::new_loading(name))
    }
}

#[derive(Clone, Default)]
struct ResolverList {
    resolvers: Arc<Mutex<Vec<Box<dyn rquickjs::loader::Resolver + Send + Sync>>>>,
}

impl rquickjs::loader::Resolver for ResolverList {
    fn resolve<'js>(&mut self, ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String> {
        let mut resolvers = self.resolvers.lock();

        for resolver in &mut *resolvers {
            if let Ok(url) = resolver.resolve(ctx, base, name) {
                return Ok(url);
            }
        }

        Err(rquickjs::Error::new_resolving(base, name))
    }
}
