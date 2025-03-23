use core::fmt;
use std::{path::Path, sync::Arc};

use rquickjs::Module;

use crate::loader::Loader;

#[cfg(feature = "swc-transform")]
pub mod swc;
#[cfg(feature = "swc-transform")]
pub use self::swc::SwcTranspiler;

#[derive(Debug)]
pub struct TranspilerError {
    inner: Box<dyn std::error::Error + Send + Sync>,
}

impl TranspilerError {
    pub fn new<T: Into<Box<dyn std::error::Error + Send + Sync>>>(error: T) -> TranspilerError {
        TranspilerError {
            inner: error.into(),
        }
    }
}

impl fmt::Display for TranspilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for TranspilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.inner)
    }
}

pub trait Transpiler: Send + Sync {
    fn compile(&self, path: &Path) -> Result<String, TranspilerError>;

    fn map(&self, path: &Path, line: usize, col: usize) -> Option<(usize, usize)>;
}

#[derive(Clone)]
pub struct Transformer {
    compiler: Arc<dyn Transpiler>,
}

impl Transformer {
    pub fn new<T: Transpiler + 'static>(transpiler: T) -> Transformer {
        Transformer {
            compiler: Arc::from(transpiler),
        }
    }

    pub fn map(&self, path: &Path, line: usize, col: usize) -> Option<(usize, usize)> {
        self.compiler.map(path, line, col)
    }
}

impl Loader for Transformer {
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        let source = self
            .compiler
            .compile(Path::new(path))
            .map_err(|err| rquickjs::Error::new_loading_message(path, err.to_string()))?;

        Module::declare(ctx.clone(), path, source)
    }
}
