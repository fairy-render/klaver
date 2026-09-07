use std::path::Path;

use anyhow::anyhow;
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer as OxcTransformer};

pub struct CodegenResult {
    pub code: Vec<u8>,
    pub sourcemap: oxc_sourcemap::SourceMap<'static>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerOptions {
    /// Enable TypeScript's `experimentalDecorators` (legacy) transform.
    ///
    /// Oxc does not yet implement the TC39 stage-3 decorators transform, unlike the SWC
    /// backend. When this is `false`, decorator syntax is left as-is in the output, which only
    /// runs on engines with native decorator support.
    pub legacy_decorators: bool,
    /// Downlevel `async`/`await` functions into generator functions.
    pub async_context: bool,
    /// Transform `using`/`await using` (explicit resource management) declarations.
    pub explicit_resource_management: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        CompilerOptions {
            legacy_decorators: false,
            async_context: false,
            explicit_resource_management: false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Compiler {
    opts: CompilerOptions,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler::default()
    }

    pub fn new_with(opts: CompilerOptions) -> Compiler {
        Compiler { opts }
    }

    pub fn compile(&self, path: &Path) -> anyhow::Result<CodegenResult> {
        let source_text = std::fs::read_to_string(path)
            .map_err(|err| anyhow!("Could not read {}: {err}", path.display()))?;
        let source_type = SourceType::from_path(path).map_err(|err| {
            anyhow!(
                "Could not determine source type for {}: {err:?}",
                path.display()
            )
        })?;

        let allocator = Allocator::default();

        let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
        if parser_ret.diagnostics.has_errors() {
            return Err(anyhow!(
                "Could not parse {}: {}",
                path.display(),
                render_diagnostics(&parser_ret.diagnostics)
            ));
        }

        let mut program = parser_ret.program;

        let semantic_ret = SemanticBuilder::new()
            // The transformer roughly triples scopes/symbols/references.
            .with_excess_capacity(2.0)
            .build(&program);

        let scoping = semantic_ret.semantic.into_scoping();

        let mut transform_options = TransformOptions::default();
        transform_options.decorator.legacy = self.opts.legacy_decorators;
        transform_options.decorator.emit_decorator_metadata = self.opts.legacy_decorators;
        transform_options.env.es2017.async_to_generator = self.opts.async_context;
        transform_options.env.es2026.explicit_resource_management =
            self.opts.explicit_resource_management;

        let transformer_ret = OxcTransformer::new(&allocator, path, &transform_options)
            .build_with_scoping(scoping, &mut program);
        if transformer_ret.diagnostics.has_errors() {
            return Err(anyhow!(
                "Could not transform {}: {}",
                path.display(),
                render_diagnostics(&transformer_ret.diagnostics)
            ));
        }

        let codegen_ret = Codegen::new()
            .with_options(CodegenOptions {
                source_map_path: Some(path.to_path_buf()),
                ..CodegenOptions::default()
            })
            .build(&program);

        let sourcemap = codegen_ret
            .map
            .ok_or_else(|| {
                anyhow!(
                    "Oxc codegen did not produce a sourcemap for {}",
                    path.display()
                )
            })?
            .into_owned();

        Ok(CodegenResult {
            code: codegen_ret.code.into_bytes(),
            sourcemap,
        })
    }
}

fn render_diagnostics<T: std::fmt::Display>(diagnostics: &[T]) -> String {
    diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}
