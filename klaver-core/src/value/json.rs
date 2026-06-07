use crate::ObjectExt;
use crate::exception;
use crate::throw;
use rquickjs::{FromJs, IntoJs, JsLifetime, class::Trace};
use serde_json::Number;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Json(pub serde_json::Value);

impl<'js> Trace<'js> for Json {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {
        // No need to trace anything since Json doesn't hold any JS values
    }
}

unsafe impl<'js> JsLifetime<'js> for Json {
    type Changed<'to> = Json;
}

#[cfg(feature = "serde")]
impl serde::Serialize for Json {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Json {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Json(value))
    }
}

impl<'js> IntoJs<'js> for Json {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self.0 {
            serde_json::Value::Null => Ok(rquickjs::Value::new_null(ctx.clone())),
            serde_json::Value::Bool(b) => Ok(rquickjs::Value::new_bool(ctx.clone(), b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(rquickjs::Value::new_int(ctx.clone(), i as i32))
                } else if let Some(u) = n.as_u64() {
                    Ok(rquickjs::Value::new_int(ctx.clone(), u as i32))
                } else if let Some(f) = n.as_f64() {
                    Ok(rquickjs::Value::new_float(ctx.clone(), f))
                } else {
                    throw!(@type ctx, "Invalid number")
                }
            }
            serde_json::Value::String(s) => {
                Ok(rquickjs::String::from_str(ctx.clone(), &s)?.into_value())
            }
            serde_json::Value::Array(arr) => {
                let js_arr = rquickjs::Array::new(ctx.clone())?;
                for (i, item) in arr.into_iter().enumerate() {
                    js_arr.set(i, Json(item).into_js(ctx)?)?;
                }
                Ok(js_arr.into_value())
            }
            serde_json::Value::Object(obj) => {
                let js_obj = rquickjs::Object::new(ctx.clone())?;
                for (k, v) in obj.into_iter() {
                    js_obj.set(k, Json(v).into_js(ctx)?)?;
                }
                Ok(js_obj.into_value())
            }
        }
    }
}

impl<'js> FromJs<'js> for Json {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if value.is_null() {
            Ok(Json(serde_json::Value::Null))
        } else if let Some(b) = value.as_bool() {
            Ok(Json(serde_json::Value::Bool(b)))
        } else if let Some(i) = value.as_int() {
            Ok(Json(serde_json::Value::Number(i.into())))
        } else if let Some(f) = value.as_float() {
            let number =
                Number::from_f64(f).ok_or_else(|| exception!(@type ctx, "Invalid number"))?;

            Ok(Json(serde_json::Value::Number(number)))
        } else if let Some(s) = value.as_string() {
            Ok(Json(serde_json::Value::String(s.to_string()?)))
        } else if let Some(arr) = value.as_array() {
            let mut vec = Vec::new();
            for i in 0..arr.len() {
                vec.push(Json::from_js(ctx, arr.get(i)?).map(|j| j.0)?);
            }
            Ok(Json(serde_json::Value::Array(vec)))
        } else if let Some(obj) = value.as_object() {
            let mut map = serde_json::Map::new();
            for ret in obj.props::<String, Json>() {
                let (k, v) = ret?;
                map.insert(k, v.0);
            }
            Ok(Json(serde_json::Value::Object(map)))
        } else {
            throw!(@type ctx, "Value cannot be converted to JSON")
        }
    }
}
