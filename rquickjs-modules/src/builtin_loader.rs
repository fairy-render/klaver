use std::collections::HashMap;

use rquickjs::{Ctx, Error, Module};

use crate::modules_builder::LoadFn;

use super::loader::Loader;

pub struct BuiltinLoader {
    pub modules: HashMap<String, LoadFn>,
    pub modules_src: HashMap<String, Vec<u8>>,
}

impl Loader for BuiltinLoader {
    fn load<'js>(
        &self,
        ctx: &Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        if let Some(load) = self.modules.get(path) {
            (load)(ctx.clone(), Vec::from(path))
        } else if let Some(source) = self.modules_src.get(path) {
            Module::declare(ctx.clone(), path, source.clone())
        } else {
            Err(Error::new_loading(path))
        }
    }
}
