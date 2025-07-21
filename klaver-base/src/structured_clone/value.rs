use dyn_clone::DynClone;
use ordered_float::OrderedFloat;
use rquickjs::IntoJs;
use std::{any::Any, collections::BTreeMap, fmt::Debug, hash::Hash};

use super::tag::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ObjectId(pub(super) u32);

#[cfg_attr(feature = "serde", feature(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransferData {
    String(String),
    Integer(i64),
    Float(OrderedFloat<f64>),
    Bool(bool),
    Bytes(Vec<u8>),
    List(Vec<TransObject>),
    Object(BTreeMap<TransObject, TransObject>),
    Item(Box<TransObject>),
    NativeObject(NativeData),
    Option(Option<Box<TransferData>>),
}

#[cfg_attr(feature = "serde", feature(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransObject {
    pub tag: Tag,
    pub data: TransferData,
    pub id: ObjectId,
}

impl<'js> IntoJs<'js> for TransferData {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        todo!()
    }
}

#[cfg_attr(feature = "serde", feature(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct NativeData {
    native: Box<dyn NativeObject>,
    id: usize,
}

impl Debug for NativeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeData")
            .field("native", &"NativeObject")
            .field("id", &self.id)
            .finish()
    }
}

impl PartialEq for NativeData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for NativeData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for NativeData {}

impl PartialOrd for NativeData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for NativeData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
#[cfg_attr(feature = "serde", typetag::serde(tag = "nativeType"))]
pub trait NativeObject: DynClone {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl Clone for Box<dyn NativeObject> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}
