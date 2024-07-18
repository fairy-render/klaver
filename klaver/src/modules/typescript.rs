use std::{io, sync::Arc};

use rquickjs::Module;
use swc::{config::IsModule, Compiler as SwcCompiler, PrintArgs};
use swc_common::{errors::Handler, source_map::SourceMap, sync::Lrc, Mark, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::TsSyntax;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_react::Runtime;
use swc_ecma_transforms_react::{jsx, parse_expr_for_jsx, Options};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_transforms_typescript::tsx;
use swc_ecma_transforms_typescript::{Config, TsxConfig};
use swc_ecma_visit::FoldWith;

use crate::Error;

use super::util::check_extensions;

pub struct Compiler {
    cm: Arc<SourceMap>,
    compiler: SwcCompiler,
    handler: Handler,
}

pub struct Source {
    pub code: String,
    pub sourcemap: Option<String>,
}

impl Compiler {
    pub fn new() -> Compiler {
        let cm = Lrc::new(SourceMap::new(swc_common::FilePathMapping::empty()));

        let compiler = SwcCompiler::new(cm.clone());
        let handler =
            Handler::with_emitter_writer(Box::new(io::stderr()), Some(compiler.cm.clone()));

        Compiler {
            cm,
            compiler,
            handler,
        }
    }
    pub fn compile(&self, filename: &str, code: &str) -> Result<Source, Error> {
        let source = self.cm.new_source_file(
            swc_common::FileName::Custom(filename.into()).into(),
            code.to_string(),
        );

        let (code, map) = GLOBALS
            .set(&Default::default(), || {
                let program = self.compiler.parse_js(
                    source,
                    &self.handler,
                    EsVersion::Es2022,
                    Syntax::Typescript(TsSyntax {
                        tsx: true,
                        decorators: true,
                        ..Default::default()
                    }),
                    IsModule::Bool(true),
                    Some(self.compiler.comments()),
                )?;

                // Add TypeScript type stripping transform
                let unresolved_mark = Mark::new();
                let top_level_mark = Mark::new();
                let program = program
                    .fold_with(&mut strip(unresolved_mark, top_level_mark))
                    .fold_with(&mut jsx(
                        self.cm.clone(),
                        Some(self.compiler.comments()),
                        Options {
                            runtime: Some(Runtime::Automatic),
                            ..Default::default()
                        },
                        top_level_mark,
                        unresolved_mark,
                    ))
                    .fold_with(&mut resolver(unresolved_mark, top_level_mark, true));

                // https://rustdoc.swc.rs/swc/struct.Compiler.html#method.print
                self.compiler
                    .print(
                        &program, // ast to print
                        PrintArgs::default(),
                    )
                    .map(|ret| (ret.code, ret.map))
            })
            .map_err(|err| Error::Unknown(Some(err.to_string())))?;

        Ok(Source {
            code,
            sourcemap: map,
        })
    }
}

pub struct TsLoader {
    extensions: Vec<String>,
    compiler: Compiler,
}

impl Default for TsLoader {
    fn default() -> Self {
        TsLoader {
            extensions: vec![
                "js".to_string(),
                "jsx".to_string(),
                "ts".to_string(),
                "jsx".to_string(),
            ],
            compiler: Compiler::new(),
        }
    }
}

impl rquickjs::loader::Loader for TsLoader {
    fn load<'js>(
        &mut self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        if !check_extensions(path, &self.extensions) {
            return Err(rquickjs::Error::new_loading(path));
        }

        let content = std::fs::read_to_string(path)?;

        let source = self
            .compiler
            .compile(path, &content)
            .map_err(|err| rquickjs::Error::new_loading_message(path, err.to_string()))?;

        Module::declare(ctx.clone(), path, source.code)
    }
}
