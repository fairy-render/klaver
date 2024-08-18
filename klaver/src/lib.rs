mod base;
mod error;
pub mod modules;
#[cfg(feature = "pool")]
pub mod pool;
pub mod vm;

mod macros;

pub use klaver_shared as shared;

pub use self::{
    error::Error,
    vm::{Vm, VmOptions},
};

pub mod core {
    pub use super::base::{get_core, Core, Extensions};
}

pub use rquickjs as quick;
