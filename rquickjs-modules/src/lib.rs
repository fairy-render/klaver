mod file_resolver;
mod global_info;
mod loader;
mod module_info;
mod modules;
mod modules_builder;
#[cfg(feature = "transform")]
pub mod transformer;
mod types;

mod builtin_loader;
mod builtin_resolver;

mod builder;
mod environ;
mod globals;

pub use self::{
    builder::Builder,
    global_info::{Global, GlobalBuilder, GlobalInfo},
    module_info::{ModuleBuilder, ModuleInfo},
    modules::Modules,
    types::Typings,
};
