use anyhow::anyhow;
use std::{path::Path, sync::Arc};
use swc_common::{Globals, Mark, SourceMap, GLOBALS};
use swc_ecma_ast::{EsVersion, Pass, Program};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms::{
    compat::es2017,
    fixer,
    helpers::{inject_helpers, Helpers, HELPERS},
    hygiene,
    proposals::{
        decorators::{decorators, Config as DecoratorsConfig},
        explicit_resource_management::explicit_resource_management,
    },
    resolver, typescript as ts,
};
use swc_ecma_visit::VisitMutWith;
use swc_node_comments::SwcComments;

pub struct CodegenResult {
    pub code: Vec<u8>,
    pub sourcemap: sourcemap::SourceMap,
}

#[derive(Debug, Clone, Copy)]
pub enum Decorators {
    Stage2022,
    Legacy,
}

impl Decorators {
    fn apply(&self, program: &mut Program) {
        match self {
            Self::Stage2022 => {
                // decorator_2022_03().process(program);
                decorators(DecoratorsConfig {
                    legacy: false,
                    emit_metadata: false,
                    use_define_for_class_fields: true,
                })
                .process(program);
            }
            Self::Legacy => {
                decorators(DecoratorsConfig {
                    legacy: true,
                    emit_metadata: true,
                    use_define_for_class_fields: true,
                })
                .process(program);
            }
        }
    }
}

pub struct CompilerOptions {
    pub decorators: Decorators,
    pub async_context: bool,
    pub explicit_resource_management: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        CompilerOptions {
            decorators: Decorators::Stage2022,
            async_context: false,
            explicit_resource_management: false,
        }
    }
}

#[derive(Default)]
pub struct Compiler {
    cm: Arc<SourceMap>,
    comments: SwcComments,
    globals: Globals,
    opts: CompilerOptions,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            cm: Default::default(),
            comments: Default::default(),
            globals: Default::default(),
            opts: Default::default(),
        }
    }

    pub fn new_with(options: CompilerOptions) -> Compiler {
        Compiler {
            cm: Default::default(),
            comments: Default::default(),
            globals: Default::default(),
            opts: options,
        }
    }

    pub fn compile(&self, path: &Path) -> anyhow::Result<CodegenResult> {
        let fm = self.cm.load_file(path)?;

        let mut errors = Vec::default();

        let mut program = swc_ecma_parser::parse_file_as_program(
            &fm,
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                dts: false,
                no_early_errors: true,
                disallow_ambiguous_jsx_like: true,
            }),
            EsVersion::Es2022,
            Some(&self.comments),
            &mut errors,
        )
        .map_err(|err| anyhow!("Could not parse: {}", err.into_kind().msg()))?;

        self.run(|| {
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            let helpers = Helpers::new(false);

            HELPERS.set(&helpers, || {
                self.opts.decorators.apply(&mut program);

                ts::tsx(
                    self.cm.clone(),
                    ts::Config::default(),
                    ts::TsxConfig::default(),
                    &self.comments,
                    unresolved_mark,
                    top_level_mark,
                )
                .process(&mut program);

                if self.opts.explicit_resource_management {
                    explicit_resource_management().process(&mut program);
                }

                if self.opts.async_context {
                    es2017(es2017::Config::default(), unresolved_mark).process(&mut program);
                }

                program.visit_mut_with(&mut fixer(Some(&self.comments)));

                hygiene().process(&mut program);

                program.visit_mut_with(&mut inject_helpers(top_level_mark));
                program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, true));
            });
        });

        let mut code = Vec::new();
        let mut srcmap = Vec::new();

        let mut emitter = swc_ecma_codegen::Emitter {
            cfg: swc_ecma_codegen::Config::default(),
            cm: self.cm.clone(),
            comments: Some(&self.comments),
            wr: JsWriter::new(self.cm.clone(), "\n", &mut code, Some(&mut srcmap)),
        };

        emitter.emit_program(&program)?;

        let srcmap = self.cm.build_source_map(&srcmap);

        Ok(CodegenResult {
            code,
            sourcemap: srcmap,
        })
    }

    fn run<T: FnOnce() -> U, U>(&self, func: T) -> U {
        GLOBALS.set(&self.globals, || func())
    }
}
