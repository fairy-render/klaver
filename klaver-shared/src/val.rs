use rquickjs::{
    class::Trace, Array, FromJs, IntoJs, IteratorJs, String as JsString, Type, Value as JsValue,
};
pub use vaerdi::{Map, Value};

use crate::date::Date;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Val(pub vaerdi::Value);

impl core::ops::Deref for Val {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Val {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'js> Trace<'js> for Val {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

macro_rules! un {
    ($expr: expr) => {
        $expr.map_err(|v| rquickjs::Error::new_from_js(v.type_name(), "value"))
    };
}

fn from_js<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    value: rquickjs::Value<'js>,
) -> rquickjs::Result<Val> {
    match value.type_of() {
        Type::Bool => Ok(Val(Value::Bool(value.as_bool().unwrap()))),
        Type::String => Ok(Val(Value::String(
            un!(value.try_into_string())?.to_string()?.into(),
        ))),
        Type::Int => Ok(Val(Value::Number(value.as_int().unwrap().into()))),
        Type::Float => Ok(Val(Value::Number(value.as_float().unwrap().into()))),
        Type::Null | Type::Undefined => Ok(Val(Value::Null)),
        Type::Array => {
            let array = un!(value.try_into_array())?;
            Ok(Val(Value::List(
                array
                    .iter::<Val>()
                    .map(|m| m.map(|m| m.0))
                    .collect::<Result<_, _>>()?,
            )))
        }
        Type::Object => {
            if Date::is(ctx, &value)? {
                let date = Date::from_js(ctx, value)?;
                return Ok(Val(date.to_datetime()?.into()));
            }

            let object = un!(value.try_into_object())?;

            let mut map = Map::default();
            for k in object.keys::<String>() {
                let k = k?;
                let v = object.get::<_, Val>(&k)?;
                map.insert(k, v.0);
            }

            Ok(Val(Value::Map(map)))
        }
        Type::Exception => {
            let exption = un!(value.try_into_exception())?;
            Ok(Val(Value::String(exption.to_string().into())))
        }
        _ => Err(rquickjs::Error::new_from_js("value", "value")),
    }
}

impl<'js> FromJs<'js> for Val {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        from_js(ctx, value)
    }
}

impl<'js> IntoJs<'js> for Val {
    fn into_js(self, ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<JsValue<'js>> {
        let val = match self.0 {
            Value::Bool(b) => JsValue::new_bool(ctx.clone(), b),
            Value::String(t) => JsString::from_str(ctx.clone(), t.as_str())?.into(),
            Value::Map(map) => {
                let obj = rquickjs::Object::new(ctx.clone())?;
                for (k, v) in map {
                    obj.set(k.as_str(), Val(v).into_js(ctx)?)?;
                }
                obj.into_value()
            }
            Value::List(list) => {
                let items = list
                    .into_iter()
                    .map(|value| Val(value).into_js(ctx))
                    .collect_js::<Array>(ctx)?;

                items.into_value()
            }
            Value::Bytes(bs) => rquickjs::ArrayBuffer::new(ctx.clone(), bs)?.into_value(),
            // Value::Date(_) => todo!(),
            Value::DateTime(date) => Date::from_chrono(ctx, date.and_utc()).into_js(&ctx)?,
            // Value::Time(_) => todo!(),
            Value::Uuid(b) => {
                JsString::from_str(ctx.clone(), &b.as_hyphenated().to_string())?.into()
            }
            Value::Number(n) => {
                if n.is_float() {
                    JsValue::new_float(ctx.clone(), n.as_f64())
                } else {
                    JsValue::new_int(ctx.clone(), n.as_i32())
                }
            }
            Value::Char(c) => JsValue::new_int(ctx.clone(), c as u32 as i32),
            Value::Null => JsValue::new_null(ctx.clone()),
            _ => return Err(rquickjs::Error::new_into_js("value", "value")),
        };

        Ok(val)
    }
}
