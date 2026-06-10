use std::collections::HashMap;

use rquickjs::{Ctx, Error, Module, loader::ImportAttributes};

use crate::{Loader, environ_builder::LoadFn, source_map::SourceMaps};

/// BuiltinLoader is a loader that loads builtin modules.
/// It is used to load modules that are built into the runtime, such as "node:fs" or "node:path".
#[derive(Debug, Default)]
pub struct BuiltinLoader {
    pub modules: HashMap<String, LoadFn>,
    pub modules_src: HashMap<String, Vec<u8>>,
}

impl Loader for BuiltinLoader {
    fn load<'js>(
        &self,
        _sourcemaps: &SourceMaps,
        ctx: &Ctx<'js>,
        path: &str,
        _attributes: Option<ImportAttributes<'js>>,
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
