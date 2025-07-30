use rquickjs::{Ctx, Value, prelude::Opt};

use crate::{TransObject, structured_clone::registry::SerializationOptions};

use super::Registry;

pub fn structured_clone<'js>(
    ctx: Ctx<'js>,
    value: Value<'js>,
    options: Opt<SerializationOptions<'js>>,
) -> rquickjs::Result<Value<'js>> {
    let opts = options.0.unwrap_or_default();

    Registry::instance(&ctx)?.structured_clone_value(&ctx, &value, &opts)
}

pub fn serialize<'js>(
    ctx: Ctx<'js>,
    value: Value<'js>,
    options: Opt<SerializationOptions<'js>>,
) -> rquickjs::Result<TransObject> {
    let opts = options.0.unwrap_or_default();

    Registry::instance(&ctx)?.serialize(&ctx, &value, &opts)
}
