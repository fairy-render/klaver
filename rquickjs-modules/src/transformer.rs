use core::fmt;
use std::path::Path;

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
use rquickjs::loader::Loader;

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

    fn compile(&self, source: &str, path: &str) -> Result<CodegenReturn, CompileError> {
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

        let ret = Transformer::new(&allocator, Path::new(path), self.transform_options.clone())
            .build_with_symbols_and_scopes(symbols, scopes, &mut program);

        if !ret.errors.is_empty() {
            return Err(CompileError::from(path, source, ret.errors));
        }

        let ret = CodeGenerator::new()
            .with_options(self.codegen_options.clone())
            .build(&program);

        Ok(ret)
    }
}

#[derive(Default)]
pub struct FileLoader {
    compiler: Compiler,
}

impl FileLoader {
    pub fn new(compiler: Compiler) -> FileLoader {
        FileLoader { compiler }
    }
}

impl Loader for FileLoader {
    fn load<'js>(
        &mut self,
        ctx: &rquickjs::Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        let content = std::fs::read_to_string(name).unwrap();
        let codegen = self
            .compiler
            .compile(&content, name)
            .map_err(|err| rquickjs::Error::new_loading_message(name, err.to_string()))?;

        rquickjs::Module::declare(ctx.clone(), name, codegen.code)
    }
}