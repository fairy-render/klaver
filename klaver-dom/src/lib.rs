#[macro_use]
mod macros;

mod class_list;
mod document;
mod element;
mod lock;
mod node_list;

use klaver::module_info;

pub use self::{document::JsDocument, element::JsElement, node_list::Children};

pub type Module = js_domjohnson;

#[rquickjs::module(rename_vars = "camelCase")]
pub mod domjohnson {

    pub use super::JsDocument as Document;

    #[rquickjs::function]
    pub fn parse(input: String) -> rquickjs::Result<Document> {
        let doc = domjohnson::Document::parse(&input);
        Ok(Document::from_doc(doc))
    }
}

module_info!("@klaver/dom" @types: include_str!("../module.d.ts") => Module);
