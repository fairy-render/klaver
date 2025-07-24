use rquickjs::{Ctx, Function, Symbol, Value};

pub fn is_async_iteratable<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };

    let symbol = Symbol::async_iterator(ctx.clone());

    obj.get::<_, Function>(symbol).is_ok()
}
