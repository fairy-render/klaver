mod builtin;
mod file;
#[cfg(feature = "swc")]
mod swc;
pub use self::{
    builtin::BuiltinLoader,
    file::{FileLoader, Transformer},
};

#[cfg(feature = "swc")]
pub use self::swc::{
    CompilerOptions as SwcCompilerOptions, Decorators as SwcDecocators, SwcTransformer,
};
