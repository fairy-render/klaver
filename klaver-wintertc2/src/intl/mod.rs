mod datetime;
mod locale;
mod module;
mod numberformat;
pub mod provider;

#[cfg(feature = "intl-baked")]
pub mod baked;

pub use self::module::IntlModule;
