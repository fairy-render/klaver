use rquickjs::{FromJs, IntoJs, Trace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Json(pub serde_json::Value);

impl<'js> Trace<'js> for Json {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {
        // No need to trace anything since Json doesn't hold any JS values
    }
}

impl<'js> IntoJs<'js> for Json {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self.0 {
            serde_json::Value::Null => Ok(rquickjs::Value::Null),
            serde_json::Value::Bool(b) => Ok(rquickjs::Value::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(rquickjs::Value::Int(i as i32))
                } else if let Some(u) = n.as_u64() {
                    Ok(rquickjs::Value::Int(u as i32))
                } else if let Some(f) = n.as_f64() {
                    Ok(rquickjs::Value::Float(f))
                } else {
                    Err(rquickjs::Error::new_from_str(
                        rquickjs::ErrorKind::Type,
                        "Invalid number",
                    ))
                }
            }
            serde_json::Value::String(s) => Ok(rquickjs::Value::String(s.into())),
            serde_json::Value::Array(arr) => {
                let js_arr = rquickjs::Array::new(ctx.clone())?;
                for (i, item) in arr.into_iter().enumerate() {
                    js_arr.set(i as u32, Json(item).into_js(ctx)?)?;
                }
                Ok(js_arr.into_value())
            }
            serde_json::Value::Object(obj) => {
                let js_obj = rquickjs::Object::new(ctx.clone())?;
                for (k, v) in obj.into_iter() {
                    js_obj.set(k.into(), Json(v).into_js(ctx)?)?;
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
            Ok(Json(serde_json::Value::Number(f.into())))
        } else if let Some(s) = value.as_str() {
            Ok(Json(serde_json::Value::String(s.to_string())))
        } else if let Some(arr) = value.as_array() {
            let mut vec = Vec::new();
            for i in 0..arr.len()? {
                vec.push(Json::from_js(ctx, arr.get(i)?).map(|j| j.0)?);
            }
            Ok(Json(serde_json::Value::Array(vec)))
        } else if let Some(obj) = value.as_object() {
            let mut map = serde_json::Map::new();
            for (k, v) in obj.iter() {
                map.insert(k.to_string(), Json::from_js(ctx, v)?.0);
            }
            Ok(Json(serde_json::Value::Object(map)))
        } else {
            Err(rquickjs::Error::new_from_str(
                rquickjs::ErrorKind::Type,
                "Expected JSON value",
            ))
        }
    }
}
