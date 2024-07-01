use std::io;

use rquickjs::{Error, Module};
use swc::{config::IsModule, Compiler, PrintArgs};
use swc_common::{errors::Handler, source_map::SourceMap, sync::Lrc, Mark, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::Syntax;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

use super::util::check_extensions;

/// Transforms typescript to javascript. Returns tuple (js string, source map)
pub fn compile(filename: &str, ts_code: &str) -> (String, String) {
    let cm = Lrc::new(SourceMap::new(swc_common::FilePathMapping::empty()));

    let compiler = Compiler::new(cm.clone());

    let source = cm.new_source_file(
        swc_common::FileName::Custom(filename.into()),
        ts_code.to_string(),
    );

    let handler = Handler::with_emitter_writer(Box::new(io::stderr()), Some(compiler.cm.clone()));

    return GLOBALS.set(&Default::default(), || {
        let program = compiler
            .parse_js(
                source,
                &handler,
                EsVersion::Es2022,
                Syntax::Typescript(Default::default()),
                IsModule::Bool(true),
                Some(compiler.comments()),
            )
            .expect("parse_js failed");

        // Add TypeScript type stripping transform
        let top_level_mark = Mark::new();
        let program = program.fold_with(&mut strip(top_level_mark));

        // https://rustdoc.swc.rs/swc/struct.Compiler.html#method.print
        let ret = compiler
            .print(
                &program, // ast to print
                PrintArgs::default(),
            )
            .expect("print failed");

        return (ret.code, ret.map.expect("no source map"));
    });
}

pub struct TsLoader {
    extensions: Vec<String>,
}

impl Default for TsLoader {
    fn default() -> Self {
        TsLoader {
            extensions: vec!["js".to_string(), "ts".to_string()],
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
            return Err(Error::new_loading(path));
        }

        let content = std::fs::read(path)?;

        Module::declare(ctx.clone(), path, content)
    }
}
