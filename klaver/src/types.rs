#[cfg(not(feature = "parallel"))]
pub trait MaybeSend {}

#[cfg(not(feature = "parallel"))]
impl<T> MaybeSend for T {}

#[cfg(feature = "parallel")]
pub trait MaybeSend: Send {}

#[cfg(feature = "parallel")]
impl<T: Send> MaybeSend for T {}
