use dyn_clone::DynClone;
use ordered_float::OrderedFloat;
use rquickjs::{Array, Class, IntoJs, JsLifetime, Object, class::Trace};
use rquickjs_util::util::ArrayExt;
use std::{any::Any, collections::BTreeMap, fmt::Debug, hash::Hash};

use crate::structured_clone::value;

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
pub enum TransObject {
    Data {
        tag: Tag,
        data: TransferData,
        id: ObjectId,
    },
    Ref {
        id: ObjectId,
    },
}

#[cfg_attr(feature = "serde", feature(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
#[rquickjs::class]
pub struct NativeData {
    pub instance: Box<dyn NativeObject>,
    pub id: usize,
}

impl<'js> Trace<'js> for NativeData {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for NativeData {
    type Changed<'to> = NativeData;
}

impl Debug for NativeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeData")
            .field("instance", &self.instance)
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
        {
            self
        };
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

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

impl Clone for Box<dyn NativeObject> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl<T> NativeObject for T
where
    T: 'static + Send + Sync + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}
impl Debug for Box<dyn NativeObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeObject")
            .field("type", &self.as_any().type_id())
            .finish()
    }
}

impl<'js> IntoJs<'js> for TransferData {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            TransferData::String(s) => s.into_js(ctx),
            TransferData::Integer(i) => i.into_js(ctx),
            TransferData::Float(f) => f.into_js(ctx),
            TransferData::Bool(b) => b.into_js(ctx),
            TransferData::Bytes(b) => b.into_js(ctx),
            TransferData::List(l) => l.into_js(ctx),
            TransferData::Object(o) => {
                let array = Array::new(ctx.clone())?;

                for (key, value) in o {
                    let pair = rquickjs_util::Entry { key, value };
                    array.push(pair)?;
                }

                let obj = Object::new(ctx.clone())?;

                obj.set("data", array)?;

                Ok(obj.into_value())
            }
            TransferData::Item(item) => item.into_js(ctx),
            TransferData::NativeObject(native) => {
                Class::instance(ctx.clone(), native)?.into_js(ctx)
            }
            TransferData::Option(opt) => opt.map(|v| v.into_js(ctx)).into_js(ctx),
        }
    }
}

impl<'js> IntoJs<'js> for TransObject {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Self::Data { tag, data, id } => {
                let obj = Object::new(ctx.clone())?;

                obj.set("tag", tag)?;
                obj.set("id", id.0)?;
                obj.set("data", data)?;

                Ok(obj.into_value())
            }
            Self::Ref { id } => {
                let obj = Object::new(ctx.clone())?;

                obj.set("ref", id.0)?;

                Ok(obj.into_value())
            }
        }
    }
}
