use std::{path::Path, sync::Arc};
use swc::{
    config::{IsModule, Options},
    try_with_handler,
};
use swc_common::{Globals, Mark, SourceMap, GLOBALS};
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
use swc_ecma_visit::{FoldWith, VisitWith};
pub struct Compiler {
    cm: Arc<SourceMap>,
    compiler: swc::Compiler,
    globals: Globals,
}

impl Compiler {
    pub fn compile(&self, path: &str) {
        GLOBALS.set(&self.globals, || {
            try_with_handler(self.cm.clone(), Default::default(), |handler| {
                //
                let fm = self.cm.load_file(Path::new(path))?;

                let program = self.compiler.parse_js(
                    fm,
                    handler,
                    EsVersion::latest(),
                    Syntax::Typescript(TsSyntax {
                        tsx: true,
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
                    program.visit_with(&mut ts::tsx(
                        self.cm.clone(),
                        ts::Config::default(),
                        ts::TsxConfig::default(),
                        self.compiler.comments(),
                        unresolved_mark,
                        top_level_mark,
                    ));

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
                    ));

                    program = program.fold_with(&mut decorator_2022_03());

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

                self.compiler
                    .print(
                        &program, // ast to print
                        PrintArgs {
                            // codegen_config: CodegenConfig::default().with_target(EsVersion::Es2022),
                            ..Default::default()
                        },
                    )
                    .map(|ret| (ret.code, ret.map));

                Ok(())
            })
        });
    }
}
