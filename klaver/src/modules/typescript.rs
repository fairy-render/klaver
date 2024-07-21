use std::{io, sync::Arc};

use relative_path::RelativePath;
use rquickjs::Module;
use swc::{config::IsModule, Compiler as SwcCompiler, PrintArgs};
use swc_common::{errors::Handler, source_map::SourceMap, sync::Lrc, Mark, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::TsSyntax;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_react::Runtime;
use swc_ecma_transforms_react::{jsx, Options};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

use crate::Error;

use super::util::check_extensions;

pub struct Compiler {
    cm: Arc<SourceMap>,
    compiler: SwcCompiler,
    handler: Handler,
}

#[derive(Debug, Default)]
pub struct CompileOptions<'a> {
    pub tsx: bool,
    pub jsx_import_source: Option<&'a str>,
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
    pub fn compile(
        &self,
        filename: &str,
        code: &str,
        config: CompileOptions,
    ) -> Result<Source, Error> {
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
                        tsx: config.tsx,
                        decorators: true,
                        ..Default::default()
                    }),
                    IsModule::Bool(true),
                    Some(self.compiler.comments()),
                )?;

                // Add TypeScript type stripping transform
                let unresolved_mark = Mark::new();
                let top_level_mark = Mark::new();
                let mut program = program.fold_with(&mut strip(unresolved_mark, top_level_mark));
                if config.tsx {
                    program = program
                        .fold_with(&mut jsx(
                            self.cm.clone(),
                            Some(self.compiler.comments()),
                            Options {
                                runtime: Some(Runtime::Automatic),
                                import_source: config.jsx_import_source.map(Into::into),
                                ..Default::default()
                            },
                            top_level_mark,
                            unresolved_mark,
                        ))
                        .fold_with(&mut resolver(unresolved_mark, top_level_mark, true));
                }

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
    jsx_import_source: Option<String>,
}

impl TsLoader {
    pub fn new(jsx_import_source: Option<String>) -> TsLoader {
        TsLoader {
            extensions: Default::default(),
            compiler: Compiler::new(),
            jsx_import_source,
        }
    }
}

impl Default for TsLoader {
    fn default() -> Self {
        TsLoader {
            extensions: vec![
                "js".to_string(),
                "jsx".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
            ],
            compiler: Compiler::new(),
            jsx_import_source: None,
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

        let rel_path = RelativePath::new(path);

        let tsx = rel_path.extension() == Some("tsx") || rel_path.extension() == Some("jsx");

        let source = self
            .compiler
            .compile(
                path,
                &content,
                CompileOptions {
                    tsx,
                    jsx_import_source: self.jsx_import_source.as_ref().map(|m| m.as_str()),
                },
            )
            .map_err(|err| rquickjs::Error::new_loading_message(path, err.to_string()))?;

        Module::declare(ctx.clone(), path, source.code)
    }
}
