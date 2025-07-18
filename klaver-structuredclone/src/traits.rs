use rquickjs::{Array, ArrayBuffer, Ctx, FromJs, IntoJs, Object, String, Value};
use rquickjs_util::{Date, throw, util::ArrayExt};
use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
    Registry, get_tag_value,
    tag::Tag,
    value::{TransObject, TransferData},
};

pub trait StructuredClone: Sized {
    const TRANSFERBLE: bool = false;

    type Item<'js>: IntoJs<'js> + FromJs<'js>;

    fn tag() -> &'static Tag;

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

macro_rules! tag {
    () => {
        fn tag() -> &'static Tag {
            static TAG: Tag = Tag::new();
            &TAG
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct StringCloner;

impl StructuredClone for StringCloner {
    type Item<'js> = rquickjs::String<'js>;

    tag!();

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        _registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::String(str) => rquickjs::String::from_str(ctx.clone(), &str),
            _ => throw!(@type ctx, "Expected String"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &Ctx<'js>,
        _: &Registry,
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
    type Item<'js> = i64;

    tag!();

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        _registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Integer(i) => Ok(i),
            _ => throw!(@type ctx, "Expected integer"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &Ctx<'js>,
        _registry: &Registry,
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
    type Item<'js> = f64;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        _registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Float(i) => Ok(*i),
            _ => throw!(@type ctx, "Expected Float"),
        }
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
    type Item<'js> = bool;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        _registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Bool(i) => Ok(i),
            _ => throw!(@type ctx, "Expected Boolean"),
        }
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

// Null

#[derive(Debug, Clone, Copy)]
pub struct NullClone<T>(PhantomData<T>);

impl<T> NullClone<T> {
    pub fn new() -> NullClone<T> {
        NullClone(PhantomData)
    }
}

impl<T> StructuredClone for NullClone<T>
where
    T: StructuredClone,
{
    type Item<'js> = Option<T::Item<'js>>;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Option(ret) => match ret {
                Some(ret) => Ok(Some(T::from_transfer_object(ctx, registry, *ret)?)),
                None => Ok(None),
            },
            _ => throw!(@type ctx, "Expected integer"),
        }
    }

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        match value {
            Some(ret) => Ok(TransferData::Option(Some(Box::new(T::to_transfer_object(
                ctx, registry, ret,
            )?)))),
            None => Ok(TransferData::Option(None)),
        }
    }
}

impl<T> Clonable for Option<T>
where
    T: StructuredClone + 'static,
{
    type Cloner = NullClone<T>;
}

// DAte

#[derive(Debug, Clone, Copy)]
pub struct DateCloner;

impl StructuredClone for DateCloner {
    type Item<'js> = Date<'js>;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        _registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::String(i) => Ok(Date::from_str(ctx, &i)?),
            _ => throw!(@type ctx, "Expected Boolean"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &Ctx<'js>,
        _registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        Ok(TransferData::String(
            value.to_string()?.as_str().to_string(),
        ))
    }
}

impl<'js> Clonable for Date<'js> {
    type Cloner = DateCloner;
}

pub struct ObjectCloner;

impl StructuredClone for ObjectCloner {
    type Item<'js> = Object<'js>;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        let TransferData::Object(btree) = obj else {
            throw!(@type ctx, "Expected Object")
        };

        let obj = Object::new(ctx.clone())?;

        for (k, v) in btree {
            let key = registry.from_transfer_object_value(ctx, k)?;
            let val = registry.from_transfer_object_value(ctx, v)?;
            obj.set(key, val)?;
        }

        Ok(obj)
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
            let value = value_transfer(ctx, registry, &v)?;
            output.insert(key, value);
        }

        Ok(TransferData::Object(output))
    }
}

impl<'js> Clonable for Object<'js> {
    type Cloner = ObjectCloner;
}

// Array

pub struct ArrayCloner;

impl StructuredClone for ArrayCloner {
    type Item<'js> = Array<'js>;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        let TransferData::List(list) = obj else {
            throw!(@type ctx, "Expected List")
        };

        let obj = Array::new(ctx.clone())?;

        for v in list {
            let val = registry.from_transfer_object_value(ctx, v)?;
            obj.push(val)?;
        }

        Ok(obj)
    }

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        let mut output = Vec::new();

        for ret in value.clone().into_iter() {
            let value = ret?;
            let value = value_transfer(ctx, registry, &value)?;
            output.push(value);
        }

        Ok(TransferData::List(output))
    }
}

impl<'js> Clonable for Array<'js> {
    type Cloner = ArrayCloner;
}

pub struct ValueCloner;

impl StructuredClone for ValueCloner {
    type Item<'js> = Value<'js>;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        let value = transer_value(ctx, registry, obj)?;
        Ok(value)
    }

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        let data = value_transfer(ctx, registry, value)?;

        Ok(TransferData::Item(Box::new(data)))
    }
}

impl<'js> Clonable for Value<'js> {
    type Cloner = ValueCloner;
}

fn value_transfer<'js>(
    ctx: &Ctx<'js>,
    registry: &Registry,
    value: &Value<'js>,
) -> rquickjs::Result<TransObject> {
    let tag = get_tag_value(ctx, value)?;
    registry
        .get_by_tag(ctx, &tag)?
        .to_transfer_object(ctx, registry, value)
}

fn transer_value<'js>(
    ctx: &Ctx<'js>,
    registry: &Registry,
    data: TransferData,
) -> rquickjs::Result<Value<'js>> {
    match data {
        TransferData::String(str) => {
            Ok(rquickjs::String::from_str(ctx.clone(), &*str)?.into_value())
        }
        TransferData::Integer(i) => Ok(Value::new_int(ctx.clone(), i as i32)),
        TransferData::Float(ordered_float) => Ok(Value::new_float(ctx.clone(), *ordered_float)),
        TransferData::Bool(b) => Ok(Value::new_bool(ctx.clone(), b)),
        TransferData::Bytes(b) => Ok(ArrayBuffer::new(ctx.clone(), b)?.into_value()),
        TransferData::List(trans_objects) => {
            //
            Ok(
                ArrayCloner::from_transfer_object(
                    ctx,
                    registry,
                    TransferData::List(trans_objects),
                )?
                .into_value(),
            )
        }
        TransferData::Object(btree_map) => {
            Ok(
                ObjectCloner::from_transfer_object(ctx, registry, TransferData::Object(btree_map))?
                    .into_value(),
            )
        }
        TransferData::Item(trans_object) => registry.from_transfer_object_value(ctx, *trans_object),
        TransferData::NativeObject(native_data) => {
            todo!()
        }
        TransferData::Option(opt) => {
            //
            match opt {
                Some(v) => transer_value(ctx, registry, *v),
                None => Ok(Value::new_null(ctx.clone())),
            }
        }
    }
}
