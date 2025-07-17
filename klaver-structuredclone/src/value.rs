use std::collections::BTreeMap;

use ordered_float::OrderedFloat;
use rquickjs::IntoJs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum TransferData {
    String(String),
    Integer(i64),
    Float(OrderedFloat<f64>),
    Bool(bool),
    Bytes(bool),
    List(Vec<TransObject>),
    Object(BTreeMap<TransObject, TransObject>),
    Item(Box<TransObject>),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TransObject {
    pub tag: String,
    pub data: TransferData,
}

impl<'js> IntoJs<'js> for TransferData {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        todo!()
    }
}
