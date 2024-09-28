use klaver::module_info;

pub type Module = js_module;

module_info!("@klaver/image" => Module);

#[rquickjs::module]
pub mod module {
    pub use crate::image::JsImage;
}
