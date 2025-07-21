use rquickjs::{Ctx, Value, prelude::Opt};
use rquickjs_util::throw;

use crate::structured_clone::registry::SerializationOptions;

use super::Registry;

pub fn structured_clone<'js>(
    ctx: Ctx<'js>,
    value: Value<'js>,
    options: Opt<SerializationOptions<'js>>,
) -> rquickjs::Result<Value<'js>> {
    let Some(registry) = ctx.userdata::<Registry>() else {
        throw!(ctx, "Registry not registered")
    };

    let opts = options.0.unwrap_or_default();

    registry.structured_clone_value(&ctx, &value, &opts)
}
