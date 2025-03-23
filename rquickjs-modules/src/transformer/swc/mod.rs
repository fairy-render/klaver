mod compiler;

use std::{collections::HashMap, path::PathBuf};

use parking_lot::Mutex;

use crate::transformer::{Transpiler, TranspilerError};

pub use self::compiler::*;

pub struct SwcTranspiler {
    compiler: Compiler,
    cache: Mutex<HashMap<PathBuf, CodegenResult>>,
}

impl SwcTranspiler {
    pub fn new() -> SwcTranspiler {
        SwcTranspiler {
            compiler: Compiler::new(),
            cache: Default::default(),
        }
    }

    pub fn new_with(decorator: Decorators) -> SwcTranspiler {
        SwcTranspiler {
            compiler: Compiler::new_with(decorator),
            cache: Default::default(),
        }
    }
}

impl Transpiler for SwcTranspiler {
    fn compile(&self, path: &std::path::Path) -> Result<String, TranspilerError> {
        let result = self.compiler.compile(path).map_err(TranspilerError::new)?;

        let source = String::from_utf8(result.code.clone()).map_err(TranspilerError::new)?;

        self.cache.lock().insert(path.to_path_buf(), result);

        Ok(source)
    }

    fn map(&self, path: &std::path::Path, line: usize, col: usize) -> Option<(usize, usize)> {
        let lock = self.cache.lock();
        let entry = lock.get(path)?;

        let token = entry.sourcemap.lookup_token(line as u32, col as u32)?;
        let dst = token.get_src();

        Some((dst.0 as usize, dst.1 as usize))
    }
}
