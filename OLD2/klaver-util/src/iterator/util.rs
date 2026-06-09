use rquickjs::{Function, Value, atom::PredefinedAtom};

pub fn is_iteratable(value: &Value<'_>) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };

    obj.get::<_, Function>(PredefinedAtom::SymbolIterator)
        .is_ok()
}
