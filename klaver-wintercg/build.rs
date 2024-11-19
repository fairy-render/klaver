use std::env;
use std::fmt::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let mut files = vec!["base"];

    if cfg!(feature = "encoding") {
        files.push("encoding");
    }

    if cfg!(feature = "http") {
        files.push("http");
    }

    if cfg!(feature = "crypto") {
        files.push("crypto");
    }

    if cfg!(feature = "icu") {
        files.push("intl");
    }

    // let mut global = String::from("declare global {\n");
    let mut global = String::from(include_str!("types/global.d.ts"));
    let mut module = String::new();

    for feature in files {
        let content =
            std::fs::read_to_string(format!("types/{feature}.d.ts")).expect("read type file");
        writeln!(module, "{}", content).expect("write module");
        writeln!(
            global,
            "{}",
            content
                .replace("export class", "declare class")
                .replace("export function", "declare function")
                .replace("export const", "declare const")
                .replace("export", "")
        )
        .expect("write global");
    }

    global.push_str(include_str!("types/outro.d.ts"));
    // global.push('}');

    let module_path = Path::new(&out_dir).join("module.d.ts");
    let global_path = Path::new(&out_dir).join("global.d.ts");

    std::fs::write(module_path, module).expect("write file");
    std::fs::write(global_path, global).expect("write global");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=types/base.d.ts");
    println!("cargo::rerun-if-changed=types/http.d.ts");
    println!("cargo::rerun-if-changed=types/crypto.d.ts");
    println!("cargo::rerun-if-changed=types/encoding.d.ts");
    println!("cargo::rerun-if-changed=types/intl.d.ts");

    println!("cargo::rerun-if-changed=types/global.d.ts");
    println!("cargo::rerun-if-changed=types/outro.d.ts");
}
