mod compiler;

use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use std::sync::Mutex;

use klaver_core::throw_if;
use rquickjs::{Ctx, Module};

use crate::loaders::Transformer;
use crate::source_map::{SourceMap, SourceMaps};

pub use self::compiler::*;

pub struct OxcTransformer {
    compiler: Compiler,
    cache: Mutex<HashMap<PathBuf, CodegenResult>>,
}

impl OxcTransformer {
    pub fn new() -> OxcTransformer {
        OxcTransformer {
            compiler: Compiler::new(),
            cache: Default::default(),
        }
    }

    pub fn new_with(opts: CompilerOptions) -> OxcTransformer {
        OxcTransformer {
            compiler: Compiler::new_with(opts),
            cache: Default::default(),
        }
    }
}

impl Default for OxcTransformer {
    fn default() -> Self {
        OxcTransformer::new()
    }
}

impl Transformer for OxcTransformer {
    fn transform<'js>(
        &self,
        sourcemaps: &SourceMaps,
        ctx: &Ctx<'js>,
        path: &Path,
        _attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let result = throw_if!(ctx, self.compiler.compile(path));

        let source = throw_if!(ctx, String::from_utf8(result.code.clone()));

        let sourcemap = SourceMap::from_iter(result.sourcemap.get_tokens().map(|token| {
            (
                (token.get_src_line(), token.get_src_col()),
                (token.get_dst_line(), token.get_dst_col()),
            )
        }));

        sourcemaps.insert(path.display().to_string(), sourcemap);

        self.cache
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), result);

        Module::declare(ctx.clone(), path.to_string_lossy().as_ref(), source)
    }

    fn map(&self, path: &std::path::Path, line: usize, col: usize) -> Option<(usize, usize)> {
        let lock = self.cache.lock().expect("Lock");
        let entry = lock.get(path)?;

        let lookup_table = entry.sourcemap.generate_lookup_table();
        let token = entry
            .sourcemap
            .lookup_token(&lookup_table, line as u32, col as u32)?;

        Some((token.get_src_line() as usize, token.get_src_col() as usize))
    }

    fn can_transform(
        &self,
        path: &Path,
        attributes: Option<&rquickjs::loader::ImportAttributes<'_>>,
    ) -> bool {
        if let Some(attrs) = attributes {
            if attrs.get("oxc").is_ok() {
                return true;
            }
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "ts" | "tsx" | "js" | "jsx" | "mjs"))
            .unwrap_or(false)
    }
}
