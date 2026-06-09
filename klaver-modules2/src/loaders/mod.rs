mod builtin;
mod file;
#[cfg(feature = "swc")]
mod swc;
pub use self::{
    builtin::BuiltinLoader,
    file::{FileLoader, Transformer},
};

#[cfg(feature = "swc")]
pub use self::swc::SwcTransformer;
