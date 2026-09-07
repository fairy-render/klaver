mod builtin;
mod file;
#[cfg(feature = "oxc")]
mod oxc;
#[cfg(feature = "swc")]
mod swc;
pub use self::{
    builtin::BuiltinLoader,
    file::{FileLoader, Transformer},
};

#[cfg(feature = "oxc")]
pub use self::oxc::{
    Compiler as OxcCompiler, CompilerOptions as OxcCompilerOptions, OxcTransformer,
};

#[cfg(feature = "swc")]
pub use self::swc::{
    Compiler as SwcCompiler, CompilerOptions as SwcCompilerOptions, Decorators as SwcDecocators,
    SwcTransformer,
};
