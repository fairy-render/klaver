use rquickjs::{module::ModuleDef, Ctx, Module};
use std::collections::HashMap;

use crate::global_info::{DynamicGlobal, Global, GlobalBox};

pub type LoadFn = for<'js> fn(Ctx<'js>, Vec<u8>) -> rquickjs::Result<Module<'js>>;

#[derive(Default)]
pub(crate) struct ModulesBuilder {
    pub modules: HashMap<String, LoadFn>,
    pub modules_src: HashMap<String, Vec<u8>>,
    pub globals: Vec<Box<dyn DynamicGlobal + Send + Sync>>,
}

impl ModulesBuilder {
    pub fn load_func<'js, D: ModuleDef>(
        ctx: Ctx<'js>,
        name: Vec<u8>,
    ) -> rquickjs::Result<Module<'js>> {
        Module::declare_def::<D, _>(ctx, name)
    }

    pub fn register_global<T: Global + Send + Sync + 'static>(&mut self, global: T) -> &mut Self {
        self.globals.push(Box::new(GlobalBox(global)));
        self
    }

    pub fn register_source(&mut self, name: impl ToString, source: Vec<u8>) {
        self.modules_src.insert(name.to_string(), source);
    }
}
