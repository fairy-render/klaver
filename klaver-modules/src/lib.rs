mod file_resolver;
mod global_info;
pub mod loader;
mod module_info;
mod modules;
mod modules_builder;
pub mod transformer;
mod types;

mod builtin_loader;
mod builtin_resolver;

mod builder;
mod environ;
mod globals;

pub use self::{
    builder::Builder,
    environ::{Environ, WeakEnviron},
    global_info::{Global, GlobalBuilder, GlobalInfo},
    module_info::{ModuleBuilder, ModuleInfo},
    modules::Modules,
    types::Typings,
};

pub use oxc_resolver as resolver;
pub use oxc_resolver::ResolveOptions;
