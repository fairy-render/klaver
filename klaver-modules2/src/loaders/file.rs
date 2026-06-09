use std::path::Path;

use rquickjs::{Ctx, Module};

use crate::Loader;

pub trait Transformer {
    fn transform<'js>(
        &self,
        ctx: &Ctx<'js>,
        path: &Path,
        attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>>;

    fn map(&self, path: &Path, line: usize, col: usize) -> Option<(usize, usize)>;

    fn can_transform(
        &self,
        path: &Path,
        attributes: Option<&rquickjs::loader::ImportAttributes<'_>>,
    ) -> bool {
        true
    }
}

impl Transformer for () {
    fn transform<'js>(
        &self,
        ctx: &Ctx<'js>,
        path: &Path,
        _attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let code = std::fs::read_to_string(path).map_err(|err| {
            rquickjs::Error::new_loading_message(path.to_string_lossy().as_ref(), err.to_string())
        })?;
        Module::declare(ctx.clone(), path.to_string_lossy().as_ref(), code)
    }

    fn map(&self, _path: &Path, line: usize, col: usize) -> Option<(usize, usize)> {
        Some((line, col))
    }

    fn can_transform(
        &self,
        path: &Path,
        _attributes: Option<&rquickjs::loader::ImportAttributes<'_>>,
    ) -> bool {
        path.ends_with(".js") || path.ends_with(".mjs")
    }
}

pub struct FileLoader {
    transformer: Vec<Box<dyn Transformer + Send + Sync>>,
}

impl Default for FileLoader {
    fn default() -> Self {
        FileLoader {
            transformer: Default::default(),
        }
    }
}

impl FileLoader {
    pub fn new(transformer: Vec<Box<dyn Transformer + Send + Sync>>) -> FileLoader {
        FileLoader { transformer }
    }

    pub fn with_transformer<T: Transformer + Send + Sync + 'static>(
        mut self,
        transformer: T,
    ) -> Self {
        self.transformer.push(Box::new(transformer));
        self
    }

    pub fn add_transformer<T: Transformer + Send + Sync + 'static>(&mut self, transformer: T) {
        self.transformer.push(Box::new(transformer));
    }
}

impl Loader for FileLoader {
    fn load<'js>(
        &self,
        ctx: &Ctx<'js>,
        path: &str,
        attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let module = self
            .transformer
            .iter()
            .find(|t| t.can_transform(Path::new(path), attributes.as_ref()))
            .ok_or_else(|| {
                rquickjs::Error::new_loading_message(path, "No suitable transformer found")
            })?
            .transform(ctx, Path::new(path), attributes)?;
        module.meta()?.set("url", format!("file://{}", path))?;
        Ok(module)
    }
}
