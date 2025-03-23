use std::path::Path;

use rquickjs_modules::transformer::{swc::Decorators, SwcTranspiler, Transpiler};

fn main() {
    let compiler = SwcTranspiler::new_with(Decorators::Legacy);

    // compiler.transform_options.jsx.import_source = Some("@klaver/template".to_string());

    let ret = compiler
        .compile(Path::new("./rquickjs-modules/examples/component.tsx"))
        .unwrap();

    println!("{}", ret);
}
