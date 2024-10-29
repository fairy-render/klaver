use std::{collections::HashMap, path::PathBuf};

use rquickjs::{
    loader::{Loader, Resolver},
    module::ModuleDef,
    Ctx, Module,
};
use samling::{File, FileStore};

type LoadFn = for<'js> fn(Ctx<'js>, Vec<u8>) -> rquickjs::Result<Module<'js>>;

pub(crate) struct ModulesBuilder {
    pub modules: HashMap<String, LoadFn>,
    pub modules_src: HashMap<String, Vec<u8>>,
    pub search_paths: Vec<PathBuf>,
    pub resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
    pub loaders: Vec<Box<dyn Loader + Send + Sync>>,
    pub routes: samling::SyncComposite,
}

impl ModulesBuilder {
    pub fn load_func<'js, D: ModuleDef>(
        ctx: Ctx<'js>,
        name: Vec<u8>,
    ) -> rquickjs::Result<Module<'js>> {
        Module::declare_def::<D, _>(ctx, name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.modules.contains_key(name) || self.modules_src.contains_key(name)
    }

    pub fn register<T: ModuleDef>(&mut self, name: impl ToString) -> &mut Self {
        self.modules
            .insert(name.to_string(), ModulesBuilder::load_func::<T>);
        self
    }

    pub fn register_source(&mut self, name: impl ToString, source: Vec<u8>) {
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
