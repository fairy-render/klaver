use rquickjs::{Array, Ctx, FromIteratorJs, Object, String, Value};

pub fn from_json<'js>(
    ctx: Ctx<'js>,
    value: serde_json::Value,
) -> rquickjs::Result<rquickjs::Value<'js>> {
    match value {
        serde_json::Value::Null => Ok(Value::new_null(ctx)),
        serde_json::Value::Bool(b) => Ok(Value::new_bool(ctx, b)),
        serde_json::Value::Number(n) => Ok(if n.is_f64() {
            Value::new_float(ctx, n.as_f64().unwrap_or_default())
        } else {
            Value::new_int(ctx, n.as_i64().unwrap_or_default() as i32)
        }),
        serde_json::Value::String(s) => Ok(Value::from_string(String::from_str(ctx, s.as_str())?)),
        serde_json::Value::Array(list) => from_json_list(&ctx, list),
        serde_json::Value::Object(map) => from_object_list(ctx, map),
    }
}

fn from_json_list<'js>(
    ctx: &Ctx<'js>,
    list: Vec<serde_json::Value>,
) -> rquickjs::Result<Value<'js>> {
    let values = list
        .into_iter()
        .map(|m| from_json(ctx.clone(), m))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Value::from_array(Array::from_iter_js(ctx, values)?))
}

fn from_object_list<'js>(
    ctx: Ctx<'js>,
    map: serde_json::Map<std::string::String, serde_json::Value>,
) -> rquickjs::Result<Value<'js>> {
    let object = Object::new(ctx.clone())?;

    for (k, v) in map {
        object.set(k, from_json(ctx.clone(), v))?;
    }

    Ok(Value::from_object(object))
}
