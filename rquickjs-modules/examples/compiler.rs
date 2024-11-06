use rquickjs_modules::transformer::Compiler;

fn main() {
    let compiler = Compiler::default();

    // compiler.transform_options.jsx.import_source = Some("@klaver/template".to_string());

    let ret = compiler
        .compile(include_str!("./component.tsx"), "component.tsx")
        .unwrap();

    println!("{}", ret.code);
}
