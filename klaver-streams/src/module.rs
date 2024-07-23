pub type Module = js_module;

klaver::module_info!("@klaver/streams" => Module);

#[rquickjs::module]
pub mod module {
    pub use crate::stream::ReadableStream;
}
