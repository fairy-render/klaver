mod builtin;

#[cfg(feature = "file-resolver")]
mod file;

pub use self::builtin::BuiltinResolver;
#[cfg(feature = "file-resolver")]
pub use self::file::{FileResolver, ResolveOptions};
