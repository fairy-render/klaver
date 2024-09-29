use klaver::module_info;

pub type Module = js_image_module;

module_info!("@klaver/image" @types: include_str!("../module.d.ts") => Module);

#[rquickjs::module]
pub mod image_module {
    pub use crate::image::JsImage as Image;
}
