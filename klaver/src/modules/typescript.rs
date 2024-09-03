use std::{io, sync::Arc};

use relative_path::RelativePath;
use rquickjs::Module;
use samling::FileStore;
use swc::{config::IsModule, Compiler as SwcCompiler, PrintArgs};
use swc_common::{errors::Handler, source_map::SourceMap, sync::Lrc, Mark, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms::{
    fixer,
    helpers::{inject_helpers, Helpers, HELPERS},
    hygiene,
    proposals::{
        decorator_2022_03::decorator_2022_03,
        decorators::{decorators, Config as DecoratorsConfig},
    },
    react, resolver, typescript as ts,
};
use swc_ecma_visit::FoldWith;

use crate::{modules::file_loader::load, Error};

use super::{loader::Loader, util::check_extensions};

pub struct Compiler {
    cm: Arc<SourceMap>,
    compiler: SwcCompiler,
    handler: Handler,
}

#[derive(Debug, Default)]
pub struct CompileOptions<'a> {
    pub jsx: bool,
    pub typescript: bool,
    pub jsx_import_source: Option<&'a str>,
    pub ts_decorators: bool,
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
                let mut program = self.compiler.parse_js(
                    source,
                    &self.handler,
                    EsVersion::Es2022,
                    Syntax::Typescript(TsSyntax {
                        tsx: config.jsx,
                        decorators: true,
                        ..Default::default()
                    }),
                    IsModule::Bool(true),
                    Some(self.compiler.comments()),
                )?;

                let unresolved_mark = Mark::new();
                let top_level_mark = Mark::new();

                let helpers = Helpers::new(false);

                let program = HELPERS.set(&helpers, || {
                    if config.ts_decorators {
                        program = program.fold_with(&mut decorators(DecoratorsConfig {
                            emit_metadata: true,
                            legacy: true,
                            use_define_for_class_fields: false,
                        }));
                    }

                    program = if config.typescript {
                        if config.jsx {
                            program.fold_with(&mut ts::tsx(
                                self.cm.clone(),
                                ts::Config::default(),
                                ts::TsxConfig::default(),
                                self.compiler.comments(),
                                unresolved_mark,
                                top_level_mark,
                            ))
                        } else {
                            program.fold_with(&mut ts::typescript(
                                ts::Config::default(),
                                unresolved_mark,
                                top_level_mark,
                            ))
                        }
                    } else {
                        program
                    };

                    if config.jsx {
                        program = program.fold_with(&mut react::jsx(
                            self.cm.clone(),
                            Some(self.compiler.comments()),
                            react::Options {
                                runtime: Some(react::Runtime::Automatic),
                                import_source: config.jsx_import_source.map(|m| m.to_string()),
                                ..Default::default()
                            },
                            top_level_mark,
                            unresolved_mark,
                        ))
                    }

                    if !config.ts_decorators {
                        program = program.fold_with(&mut decorator_2022_03());
                    }

                    program
                        .fold_with(&mut fixer(Some(self.compiler.comments())))
                        .fold_with(&mut hygiene())
                        .fold_with(&mut inject_helpers(top_level_mark))
                        .fold_with(&mut resolver(
                            unresolved_mark,
                            top_level_mark,
                            config.typescript,
                        ))
                });

                // https://rustdoc.swc.rs/swc/struct.Compiler.html#method.print
                self.compiler
                    .print(
                        &program, // ast to print
                        PrintArgs {
                            // codegen_config: CodegenConfig::default().with_target(EsVersion::Es2022),
                            ..Default::default()
                        },
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

pub struct TsLoader<T> {
    extensions: Vec<String>,
    compiler: Compiler,
    jsx_import_source: Option<String>,
    ts_decorators: bool,
    fs: T,
}

impl<T> TsLoader<T> {
    pub(crate) fn new(
        fs: T,
        jsx_import_source: Option<String>,
        legacy_decorators: bool,
    ) -> TsLoader<T> {
        TsLoader {
            jsx_import_source,
            ts_decorators: legacy_decorators,
            fs,
            extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
            ],
            compiler: Compiler::new(),
        }
    }
}

impl<T> Loader for TsLoader<T>
where
    T: FileStore,
{
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        if !check_extensions(path, &self.extensions) {
            return Err(rquickjs::Error::new_loading_message(
                path,
                "unknown extension",
            ));
        }

        let rel_path = RelativePath::new(path);

        let jsx = rel_path.extension() == Some("tsx") || rel_path.extension() == Some("jsx");
        let typescript = rel_path.extension() == Some("ts") || rel_path.extension() == Some("tsx");

        tracing::trace!(path = %path, jsx = %jsx, "compiling path");

        let content = load(&self.fs, path)?;
        let content = String::from_utf8(content)?;

        let source = self
            .compiler
            .compile(
                path,
                &content,
                CompileOptions {
                    jsx,
                    typescript,
                    jsx_import_source: self.jsx_import_source.as_ref().map(|m| m.as_str()),
                    ts_decorators: self.ts_decorators,
                },
            )
            .map_err(|err| rquickjs::Error::new_loading_message(path, err.to_string()))?;

        Module::declare(ctx.clone(), path, source.code)
    }
}
