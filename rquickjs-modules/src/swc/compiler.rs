use anyhow::anyhow;
use std::{path::Path, sync::Arc};
use swc_common::{BytePos, Globals, LineCol, Mark, SourceMap, GLOBALS};
use swc_ecma_ast::{EsVersion, Pass};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms::{
    fixer,
    helpers::{inject_helpers, Helpers, HELPERS},
    hygiene,
    proposals::{
        decorator_2022_03::decorator_2022_03,
        decorators::{decorators, Config as DecoratorsConfig},
    },
    react, resolver, typescript as ts, Assumptions,
};
use swc_ecma_visit::{FoldWith, VisitMutWith, VisitWith};
use swc_node_comments::SwcComments;

pub struct CodegenResult {
    pub code: Vec<u8>,
    pub sourcemap: sourcemap::SourceMap,
}

pub struct Compiler {
    cm: Arc<SourceMap>,
    comments: SwcComments,
    globals: Globals,
}

impl Compiler {
    pub fn compile(&self, path: &str) -> anyhow::Result<CodegenResult> {
        let fm = self.cm.load_file(Path::new(path))?;

        let mut errors = Vec::default();

        let mut program = swc_ecma_parser::parse_file_as_program(
            &fm,
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                dts: true,
                no_early_errors: true,
                disallow_ambiguous_jsx_like: true,
            }),
            EsVersion::Es2022,
            Some(&self.comments),
            &mut errors,
        )
        .map_err(|err| anyhow!("Could not parse"))?;

        self.run(|| {
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            let helpers = Helpers::new(false);

            HELPERS.set(&helpers, || {
                ts::tsx(
                    self.cm.clone(),
                    ts::Config::default(),
                    ts::TsxConfig::default(),
                    &self.comments,
                    unresolved_mark,
                    top_level_mark,
                )
                .process(&mut program);

                program.visit_mut_with(&mut fixer(Some(&self.comments)));

                hygiene().process(&mut program);

                program.visit_mut_with(&mut inject_helpers(top_level_mark));
                program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, true));
            });
        });

        let mut code = Vec::new();
        let mut srcmap = Vec::new();

        let mut emitter = swc_ecma_codegen::Emitter {
            cfg: Default::default(),
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
