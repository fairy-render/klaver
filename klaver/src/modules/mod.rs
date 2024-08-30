use builtin_loader::BuiltinLoader;
use builtin_resolver::BuiltinResolver;
use loader::{Loader, Resolver};
use rquickjs::{module::ModuleDef, Ctx, Error, Module};
use samling::{fs::FsFileStore, File, FileStore, FileStoreExt};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
mod builtin_loader;
mod builtin_resolver;
// mod file;
mod file_loader;
mod init;
mod loader;
mod module;
mod resolver;
#[cfg(feature = "typescript")]
pub mod typescript;
mod util;

pub use self::{init::*, module::*};

type LoadFn = for<'js> fn(Ctx<'js>, Vec<u8>) -> rquickjs::Result<Module<'js>>;

pub(crate) trait Runtime {
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

#[derive(Default)]
pub struct ModulesBuilder {
    modules: HashMap<String, LoadFn>,
    modules_src: HashMap<String, Vec<u8>>,
    search_paths: Vec<PathBuf>,
    resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
    loaders: Vec<Box<dyn Loader + Send + Sync>>,
    routes: samling::SyncComposite,

    #[cfg(feature = "typescript")]
    jsx_import_source: Option<String>,
    #[cfg(feature = "typescript")]
    ts_decorators: bool,
}

impl ModulesBuilder {
    pub fn load_func<'js, D: ModuleDef>(
        ctx: Ctx<'js>,
        name: Vec<u8>,
    ) -> rquickjs::Result<Module<'js>> {
        Module::declare_def::<D, _>(ctx, name)
    }

    #[cfg(feature = "typescript")]
    pub fn set_jsx_import_source(&mut self, path: &str) -> &mut Self {
        self.jsx_import_source = Some(path.to_string());
        self
    }

    #[cfg(feature = "typescript")]
    pub fn use_legacy_decorators(&mut self, on: bool) -> &mut Self {
        self.ts_decorators = on;
        self
    }

    pub fn register<T: ModuleDef>(&mut self, name: impl ToString) -> &mut Self {
        self.modules
            .insert(name.to_string(), ModulesBuilder::load_func::<T>);
        self
    }

    pub fn register_src(&mut self, name: impl ToString, source: Vec<u8>) {
        self.modules_src.insert(name.to_string(), source);
    }

    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    pub fn mount_store<T>(&mut self, path: &str, store: T)
    where
        T: FileStore + Send + Sync + 'static,
        T::List: Send,
        T::File: Send + Sync,
        <T::File as File>::Body: Send + 'static,
    {
        self.routes.register(path, store);
    }

    pub fn add_loader<T>(&mut self, loader: T) -> &mut Self
    where
        T: Loader + Send + Sync + 'static,
    {
        self.loaders.push(Box::new(loader));
        self
    }

    pub fn add_resolver<T>(&mut self, resolver: T) -> &mut Self
    where
        T: Resolver + Send + Sync + 'static,
    {
        self.resolvers.push(Box::new(resolver));
        self
    }
}

impl ModulesBuilder {
    pub fn build(mut self) -> Result<Modules, Error> {
        let mut fs = Vec::with_capacity(self.search_paths.len() + 1);

        for path in &self.search_paths {
            fs.push(FsFileStore::new(path.to_path_buf())?.boxed());
        }

        fs.push(self.routes.boxed());

        let fs = Arc::new(fs);

        let file_resolver = resolver::FileResolver::new(fs.clone());
        #[cfg(not(feature = "typescript"))]
        let file_loader = file_loader::FileLoader::new(fs.clone());
        #[cfg(feature = "typescript")]
        let file_loader =
            typescript::TsLoader::new(fs.clone(), self.jsx_import_source, self.ts_decorators);

        let mut builtin_resolver = BuiltinResolver::default();

        for k in self.modules.keys() {
            builtin_resolver.add_module(k);
        }

        for k in self.modules_src.keys() {
            builtin_resolver.add_module(k);
        }

        let module_loader = BuiltinLoader {
            modules: self.modules,
            modules_src: self.modules_src,
        };

        self.resolvers.push(Box::new(builtin_resolver));
        self.resolvers.push(Box::new(file_resolver));

        self.loaders.push(Box::new(module_loader));
        self.loaders.push(Box::new(file_loader));

        Ok(Modules(Arc::new(ModulesInner {
            loaders: self.loaders,
            resolvers: self.resolvers,
        })))
    }
}

struct ModulesInner {
    resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
    loaders: Vec<Box<dyn Loader + Send + Sync>>,
}

#[derive(Clone)]
pub struct Modules(Arc<ModulesInner>);

impl Modules {
    pub(crate) async fn attach<T: Runtime>(&self, runtime: &T) -> rquickjs::Result<()> {
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
