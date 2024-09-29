use klaver::module_info;

pub type Module = js_image_module;

module_info!("@klaver/image" => Module);

#[rquickjs::module]
pub mod image_module {
    pub use crate::image::JsImage;
}
