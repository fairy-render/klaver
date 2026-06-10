mod compiler;

use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use std::sync::Mutex;

use klaver_core::{throw, throw_if};
use rquickjs::{Ctx, Module};

use crate::loaders::Transformer;
use crate::source_map::{SourceMap, SourceMaps};

pub use self::compiler::*;

pub struct SwcTransformer {
    compiler: Compiler,
    cache: Mutex<HashMap<PathBuf, CodegenResult>>,
}

impl SwcTransformer {
    pub fn new() -> SwcTransformer {
        SwcTransformer {
            compiler: Compiler::new(),
            cache: Default::default(),
        }
    }

    pub fn new_with(opts: CompilerOptions) -> SwcTransformer {
        SwcTransformer {
            compiler: Compiler::new_with(opts),
            cache: Default::default(),
        }
    }
}

impl Transformer for SwcTransformer {
    fn transform<'js>(
        &self,
        sourcemaps: &SourceMaps,
        ctx: &Ctx<'js>,
        path: &Path,
        _attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let result = throw_if!(ctx, self.compiler.compile(path));

        let source = throw_if!(ctx, String::from_utf8(result.code.clone()));

        let sourcmap = SourceMap::from_iter(
            result
                .sourcemap
                .tokens()
                .map(|token| (token.get_src(), token.get_dst())),
        );

        sourcemaps.insert(path.display().to_string(), sourcmap);

        self.cache
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), result);

        Module::declare(ctx.clone(), path.to_string_lossy().as_ref(), source)
    }

    fn map(&self, path: &std::path::Path, line: usize, col: usize) -> Option<(usize, usize)> {
        let lock = self.cache.lock().expect("Lock");
        let entry = lock.get(path)?;

        let token = entry.sourcemap.lookup_token(line as u32, col as u32)?;
        let dst = token.get_src();

        Some((dst.0 as usize, dst.1 as usize))
    }

    fn can_transform(
        &self,
        path: &Path,
        attributes: Option<&rquickjs::loader::ImportAttributes<'_>>,
    ) -> bool {
        if let Some(attrs) = attributes {
            if attrs.get("swc").is_ok() {
                return true;
            }
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "ts" | "tsx" | "js" | "jsx"))
            .unwrap_or(false)
    }
}
