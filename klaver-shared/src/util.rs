use rquickjs::{atom::PredefinedAtom, object, Ctx, Function, Symbol, Value};

pub fn is_iterator(value: &Value<'_>) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };

    obj.get::<_, Function>(PredefinedAtom::SymbolIterator)
        .is_ok()
}

pub fn is_async_iterator<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };

    let symbol = Symbol::async_iterator(ctx.clone());

    obj.get::<_, Function>(symbol).is_ok()
}
