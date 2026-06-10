use core::fmt;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc::{
    allocator::Allocator,
    ast::ast::Program,
    codegen::{CodeGenerator, CodegenOptions, CodegenReturn},
    diagnostics::{Error, NamedSource, OxcDiagnostic},
    parser::{ParseOptions, Parser},
    semantic::SemanticBuilder,
    span::SourceType,
    transformer::{TransformOptions, Transformer},
};
use parking_lot::RwLock;

use crate::loader::Loader;

#[derive(Debug)]
pub struct CompileError {
    reports: Vec<Error>,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.reports {
            writeln!(f, "{error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for CompileError {}

impl CompileError {
    fn from(source_name: &str, source_text: &str, errors: Vec<OxcDiagnostic>) -> CompileError {
        CompileError {
            reports: errors
                .into_iter()
                .map(|m| m.with_source_code(NamedSource::new(source_name, source_text.to_string())))
                .collect(),
        }
    }
}

pub struct SourceMap<'a> {
    map: &'a oxc::sourcemap::SourceMap,
    lookup_table: Vec<(u32, u32, u32)>,
}

impl<'a> SourceMap<'a> {
    pub fn view(&self, line: u32, col: u32) -> Option<oxc::sourcemap::SourceViewToken<'_>> {
        self.map
            .lookup_source_view_token(&self.lookup_table, line, col)
    }
}

pub struct CacheEntry {
    pub source: String,
    pub transformed: CodegenReturn,
}

impl CacheEntry {
    pub fn source_map(&self) -> Option<SourceMap<'_>> {
        let Some(source) = &self.transformed.map else {
            return None;
        };

        let ids = source.generate_lookup_table();

        Some(SourceMap {
            map: source,
            lookup_table: ids,
        })
    }
}

#[derive(Default, Clone)]
pub struct Cache {
    entries: Arc<RwLock<HashMap<String, Arc<CacheEntry>>>>,
}

impl Cache {
    pub fn get(&self, path: &str) -> Option<Arc<CacheEntry>> {
        let lock = self.entries.read();

        lock.get(path).cloned()
    }

    pub fn clear(&self) {
        self.entries.write().clear();
    }

    pub fn set(&self, path: &str, source: String, transformed: CodegenReturn) {
        let mut lock = self.entries.write();

        lock.insert(
            path.to_string(),
            CacheEntry {
                source,
                transformed,
            }
            .into(),
        );
    }
}

#[derive(Default)]
pub struct Compiler {
    pub parse_options: ParseOptions,
    pub transform_options: TransformOptions,
    pub codegen_options: CodegenOptions,
}

impl Compiler {
    fn parse<'a>(
        &self,
        allocator: &'a Allocator,
        source_name: &'a str,
        source_text: &'a str,
        source_type: SourceType,
    ) -> Result<Program<'a>, CompileError> {
        let ret = Parser::new(allocator, source_text, source_type)
            .with_options(self.parse_options.clone())
            .parse();

        if ret.panicked {
            return Err(CompileError {
                reports: ret
                    .errors
                    .into_iter()
                    .map(|m| {
                        m.with_source_code(NamedSource::new(source_name, source_text.to_string()))
                    })
                    .collect(),
            });
        }

        Ok(ret.program)
    }

    pub fn compile(&self, source: &str, path: &str) -> Result<CodegenReturn, CompileError> {
        let allocator = Allocator::default();

        let source_type = SourceType::from_path(path).unwrap();

        let mut program = self.parse(&allocator, path, source, source_type)?;

        let ret = SemanticBuilder::new()
            // Estimate transformer will triple scopes, symbols, references
            .with_excess_capacity(2.0)
            .build(&program);

        if !ret.errors.is_empty() {
            return Err(CompileError::from(path, source, ret.errors));
        }

        let (symbols, scopes) = ret.semantic.into_symbol_table_and_scope_tree();

        let ret = Transformer::new(&allocator, Path::new(path), &self.transform_options)
            .build_with_symbols_and_scopes(symbols, scopes, &mut program);

        if !ret.errors.is_empty() {
            return Err(CompileError::from(path, source, ret.errors));
        }

        let ret = CodeGenerator::new()
            .with_options(CodegenOptions {
                source_map_path: Some(PathBuf::from(path)),
                ..Default::default()
            })
            .build(&program);

        Ok(ret)
    }
}

#[derive(Default)]
pub struct FileLoader {
    compiler: Compiler,
    cache: Cache,
    use_cache: bool,
}

impl FileLoader {
    pub fn new(compiler: Compiler, cache: Cache, use_cache: bool) -> FileLoader {
        FileLoader {
            compiler,
            cache,
            use_cache,
        }
    }
}

impl Loader for FileLoader {
    fn load<'js>(
        &self,
        ctx: &rquickjs::Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        if !Path::new(name).exists() {
            return Err(rquickjs::Error::new_loading_message(
                name,
                "File does not exists",
            ));
        }

        if self.use_cache {
            if let Some(entry) = self.cache.get(name) {
                return rquickjs::Module::declare(ctx.clone(), name, &*entry.transformed.code);
            }
        }

        let content = std::fs::read_to_string(name).unwrap();
        let codegen = self
            .compiler
            .compile(&content, name)
            .map_err(|err| rquickjs::Error::new_loading_message(name, err.to_string()))?;

        self.cache.set(name, content, codegen);

        let Some(entry) = self.cache.get(name) else {
            return Err(rquickjs::Error::new_loading_message(name, "Cache error"));
        };

        rquickjs::Module::declare(ctx.clone(), name, &*entry.transformed.code)
    }
}
