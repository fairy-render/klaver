use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use ordered_float::OrderedFloat;
use rquickjs::{Class, Ctx, FromJs, IntoJs, Object, String, Value, class::JsClass};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    Registry,
    value::{TransObject, TransferData},
};

pub trait StructuredClone: Sized {
    const TAG: &'static str;
    const TRANSFERBLE: bool = false;

    type Item<'js>: IntoJs<'js> + FromJs<'js>;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>>;

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData>;
}

pub trait Clonable {
    type Cloner: StructuredClone + 'static;
}

#[derive(Debug, Clone, Copy)]
pub struct StringCloner;

impl StructuredClone for StringCloner {
    const TAG: &'static str = "String";
    type Item<'js> = rquickjs::String<'js>;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        _ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        Ok(TransferData::String(value.to_string()?))
    }
}

impl<'js> Clonable for rquickjs::String<'js> {
    type Cloner = StringCloner;
}

#[derive(Debug, Clone, Copy)]
pub struct IntCloner;

impl StructuredClone for IntCloner {
    const TAG: &'static str = "Integer";
    type Item<'js> = i64;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        _ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        Ok(TransferData::Integer(*value))
    }
}

impl Clonable for i64 {
    type Cloner = IntCloner;
}

#[derive(Debug, Clone, Copy)]
pub struct FloatCloner;

impl StructuredClone for FloatCloner {
    const TAG: &'static str = "Float";
    type Item<'js> = f64;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        _ctx: &Ctx<'js>,
        _registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        Ok(TransferData::Float((*value).into()))
    }
}

impl Clonable for f64 {
    type Cloner = FloatCloner;
}

#[derive(Debug, Clone, Copy)]
pub struct BoolCloner;

impl StructuredClone for BoolCloner {
    const TAG: &'static str = "Boolean";
    type Item<'js> = bool;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        _ctx: &Ctx<'js>,
        _registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        Ok(TransferData::Bool(*value))
    }
}

impl Clonable for bool {
    type Cloner = BoolCloner;
}

pub struct ObjectCloner;

impl StructuredClone for ObjectCloner {
    const TAG: &'static str = "Object";

    type Item<'js> = Object<'js>;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        let mut output = BTreeMap::new();

        for ret in value.clone().into_iter() {
            let (k, v) = ret?;
            let key = registry.to_transfer_object::<String>(ctx, k.to_js_string()?)?;
            let value = registry.to_transfer_object_value(ctx, &v)?;
            output.insert(key, value);
        }

        Ok(TransferData::Object(output))
    }
}

impl<'js> Clonable for Object<'js> {
    type Cloner = ObjectCloner;
}

pub struct ValueCloner;

impl StructuredClone for ValueCloner {
    const TAG: &'static str = "JsValue";

    type Item<'js> = Value<'js>;

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        let data = match value.type_of() {
            rquickjs::Type::Uninitialized => todo!(),
            rquickjs::Type::Undefined => todo!(),
            rquickjs::Type::Null => todo!(),
            rquickjs::Type::Bool => registry.to_transfer_object_value(ctx, value)?,
            rquickjs::Type::Int => todo!(),
            rquickjs::Type::Float => todo!(),
            rquickjs::Type::String => todo!(),
            rquickjs::Type::Symbol => todo!(),
            rquickjs::Type::Array => todo!(),
            rquickjs::Type::Constructor => todo!(),
            rquickjs::Type::Function => todo!(),
            rquickjs::Type::Promise => todo!(),
            rquickjs::Type::Exception => todo!(),
            rquickjs::Type::Object => todo!(),
            rquickjs::Type::Module => todo!(),
            rquickjs::Type::BigInt => todo!(),
            rquickjs::Type::Unknown => todo!(),
        };

        Ok(TransferData::Item(Box::new(data)))
    }
}

impl<'js> Clonable for Value<'js> {
    type Cloner = ValueCloner;
}
