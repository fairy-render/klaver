use std::{collections::HashMap, path::PathBuf};

use relative_path::RelativePathBuf;
use rquickjs::{
    loader::{BuiltinResolver, FileResolver, Loader, Resolver},
    module::ModuleDef,
    Ctx, Error, Module, Result,
};

mod file;
#[cfg(feature = "typescript")]
pub mod typescript;
mod util;

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

#[derive(Debug, Default)]
pub struct Modules {
    modules: HashMap<String, LoadFn>,
    search_paths: Vec<PathBuf>,
    patterns: Vec<String>,
}

impl Modules {
    fn load_func<'js, D: ModuleDef>(ctx: Ctx<'js>, name: Vec<u8>) -> Result<Module<'js>> {
        Module::declare_def::<D, _>(ctx, name)
    }

    pub fn register<T: ModuleDef>(&mut self, name: impl ToString) {
        self.modules
            .insert(name.to_string(), Modules::load_func::<T>);
    }

    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.search_paths.push(path.into());
        self
    }

    pub async fn attach<T: Runtime>(self, runtime: &T) {
        let mut builtin_resolver = BuiltinResolver::default();
        let mut file_resolver = file::FileResolver::default();
        #[cfg(feature = "typescript")]
        let script_loader = crate::typescript::TsLoader::default();
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

        file_resolver.add_paths(self.search_paths);
        for pattern in self.patterns {
            file_resolver.add_pattern(pattern);
        }

        let module_loader = ModuleLoader {
            modules: self.modules,
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
}

impl Loader for ModuleLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        path: &str,
    ) -> Result<Module<'js, rquickjs::module::Declared>> {
        let load = self
            .modules
            .remove(path)
            .ok_or_else(|| Error::new_loading(path))?;

        (load)(ctx.clone(), Vec::from(path))
    }
}
