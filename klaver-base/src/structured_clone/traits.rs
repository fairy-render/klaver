use rquickjs::{Array, ArrayBuffer, FromJs, IntoJs, Object, Value};
use rquickjs_util::{Date, throw, util::ArrayExt};
use std::{collections::BTreeMap, marker::PhantomData};

use crate::structured_clone::context::SerializationContext;

use super::{
    tag::Tag,
    value::{TransObject, TransferData},
};

/// Traits for structured cloning in JavaScript.
/// This trait is used to define how different types can be cloned and transferred
/// between JavaScript and Rust. It allows for custom serialization and deserialization
/// of types, enabling them to be passed as transfer objects in JavaScript.
pub trait StructuredClone: Sized {
    const TRANSFERBLE: bool = false;

    type Item<'js>: IntoJs<'js> + FromJs<'js>;

    fn tag() -> &'static Tag;

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>>;

    fn to_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
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
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::String(str) => rquickjs::String::from_str(ctx.ctx().clone(), &str),
            _ => throw!(@type ctx.ctx(), "Expected String"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &mut SerializationContext<'js, '_>,
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
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Integer(i) => Ok(i),
            _ => throw!(@type ctx.ctx(), "Expected integer"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &mut SerializationContext<'js, '_>,
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

    tag!();

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Float(i) => Ok(*i),
            _ => throw!(@type ctx.ctx(), "Expected Float"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &mut SerializationContext<'js, '_>,
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

    tag!();

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Bool(i) => Ok(i),
            _ => throw!(@type ctx.ctx(), "Expected Boolean"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &mut SerializationContext<'js, '_>,
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

    tag!();

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Option(ret) => match ret {
                Some(ret) => Ok(Some(T::from_transfer_object(ctx, *ret)?)),
                None => Ok(None),
            },
            _ => throw!(@type ctx.ctx(), "Expected integer"),
        }
    }

    fn to_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        match value {
            Some(ret) => Ok(TransferData::Option(Some(Box::new(T::to_transfer_object(
                ctx, ret,
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

    tag!();

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::String(i) => {
                let date = Date::from_str(ctx.ctx(), &i)?;
                Ok(date)
            }
            _ => throw!(@type ctx.ctx(), "Expected Boolean"),
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &mut SerializationContext<'js, '_>,
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

    tag!();

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        let TransferData::Object(btree) = obj else {
            throw!(@type ctx.ctx(), "Expected Object")
        };

        let obj = Object::new(ctx.ctx().clone())?;

        ctx.cache_value(&obj);

        for (k, v) in btree {
            let key = ctx.from_transfer_object(k)?;
            let val = ctx.from_transfer_object(v)?;
            obj.set(key, val)?;
        }

        Ok(obj)
    }

    fn to_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        ctx.cache_value(value);

        let mut output = BTreeMap::new();

        for ret in value.clone().into_iter() {
            let (k, v) = ret?;
            let key = ctx.to_transfer_object(&k.to_value()?)?;
            let value = value_transfer(ctx, &v)?;
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

    tag!();

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        let TransferData::List(list) = obj else {
            throw!(@type ctx.ctx(), "Expected List")
        };

        let obj = Array::new(ctx.ctx().clone())?;

        ctx.cache_value(&obj);

        for v in list {
            let val = ctx.from_transfer_object(v)?;
            obj.push(val)?;
        }

        Ok(obj)
    }

    fn to_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        ctx.cache_value(value);

        let mut output = Vec::new();

        for ret in value.clone().into_iter() {
            let value = ret?;
            let value = value_transfer(ctx, &value)?;
            output.push(value);
        }

        Ok(TransferData::List(output))
    }
}

impl<'js> Clonable for Array<'js> {
    type Cloner = ArrayCloner;
}

// pub struct ValueCloner;

// impl StructuredClone for ValueCloner {
//     type Item<'js> = Value<'js>;

//     tag!();

//     fn from_transfer_object<'js>(
//         ctx: &mut SerializationContext<'js, '_>,
//         obj: TransferData,
//     ) -> rquickjs::Result<Self::Item<'js>> {
//         let value = transer_value(ctx, obj)?;
//         Ok(value)
//     }

//     fn to_transfer_object<'js>(
//         ctx: &mut SerializationContext<'js, '_>,
//         value: &Self::Item<'js>,
//     ) -> rquickjs::Result<TransferData> {
//         let data = value_transfer(ctx, value)?;

//         Ok(TransferData::Item(Box::new(data)))
//     }
// }

// impl<'js> Clonable for Value<'js> {
//     type Cloner = ValueCloner;
// }

fn value_transfer<'js>(
    ctx: &mut SerializationContext<'js, '_>,
    value: &Value<'js>,
) -> rquickjs::Result<TransObject> {
    ctx.to_transfer_object(value)
}

fn transer_value<'js>(
    ctx: &mut SerializationContext<'js, '_>,
    data: TransferData,
) -> rquickjs::Result<Value<'js>> {
    match data {
        TransferData::String(str) => {
            Ok(rquickjs::String::from_str(ctx.ctx().clone(), &*str)?.into_value())
        }
        TransferData::Integer(i) => Ok(Value::new_int(ctx.ctx().clone(), i as i32)),
        TransferData::Float(ordered_float) => {
            Ok(Value::new_float(ctx.ctx().clone(), *ordered_float))
        }
        TransferData::Bool(b) => Ok(Value::new_bool(ctx.ctx().clone(), b)),
        TransferData::Bytes(b) => Ok(ArrayBuffer::new(ctx.ctx().clone(), b)?.into_value()),
        TransferData::List(trans_objects) => {
            //
            Ok(
                ArrayCloner::from_transfer_object(ctx, TransferData::List(trans_objects))?
                    .into_value(),
            )
        }
        TransferData::Object(btree_map) => Ok(ObjectCloner::from_transfer_object(
            ctx,
            TransferData::Object(btree_map),
        )?
        .into_value()),
        TransferData::Item(trans_object) => ctx.from_transfer_object(*trans_object),
        TransferData::NativeObject(native_data) => {
            todo!()
        }
        TransferData::Option(opt) => {
            //
            match opt {
                Some(v) => transer_value(ctx, *v),
                None => Ok(Value::new_null(ctx.ctx().clone())),
            }
        }
    }
}
