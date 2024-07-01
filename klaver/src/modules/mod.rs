use std::{collections::HashMap, hash::Hash, path::PathBuf};

use rquickjs::{
    loader::{BuiltinResolver, Loader, Resolver},
    module::ModuleDef,
    Ctx, Error, Module, Result,
};

mod file;
mod global;
mod module;
#[cfg(feature = "typescript")]
pub mod typescript;
mod util;

pub use self::{global::*, module::*};

type LoadFn = for<'js> fn(Ctx<'js>, Vec<u8>) -> Result<Module<'js>>;

pub trait Runtime {
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

#[derive(Debug, Default, Clone)]
pub struct Modules {
    pub(crate) modules: HashMap<String, LoadFn>,
    modules_src: HashMap<String, Vec<u8>>,
    search_paths: Vec<PathBuf>,
    patterns: Vec<String>,
}

impl Modules {
    pub(crate) fn load_func<'js, D: ModuleDef>(
        ctx: Ctx<'js>,
        name: Vec<u8>,
    ) -> Result<Module<'js>> {
        Module::declare_def::<D, _>(ctx, name)
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

    pub async fn attach<T: Runtime>(self, runtime: &T) {
        let mut builtin_resolver = BuiltinResolver::default();
        let mut file_resolver = file::FileResolver::default();
        #[cfg(feature = "typescript")]
        let script_loader = typescript::TsLoader::default();
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
                (builtin_resolver, file_resolver),
                (script_loader, module_loader),
            )
            .await;
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
    ) -> Result<Module<'js, rquickjs::module::Declared>> {
        if let Some(load) = self.modules.remove(path) {
            (load)(ctx.clone(), Vec::from(path))
        } else if let Some(source) = self.modules_src.remove(path) {
            Module::declare(ctx.clone(), path, source)
        } else {
            Err(Error::new_loading(path))
        }
    }
}
