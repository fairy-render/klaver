#[cfg(feature = "send")]
pub type Krc<T> = std::sync::Arc<T>;

#[cfg(not(feature = "send"))]
pub type Krc<T> = std::sync::Arc<T>;

#[cfg(not(feature = "send"))]
pub trait MaybeSend {}

#[cfg(not(feature = "send"))]
impl<T> MaybeSend for T {}

#[cfg(feature = "send")]
pub trait MaybeSend: Send {}

#[cfg(feature = "send")]
impl<T: Send> MaybeSend for T {}

#[cfg(not(feature = "send"))]
pub type Locket<T> = Krc<std::cell::RefCell<T>>;

#[cfg(feature = "send")]
pub type Locket<T> = Krc<std::sync::Mutex<T>>;

pub use locket;
