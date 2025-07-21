use rquickjs::{Ctx, Value};
use rquickjs_util::throw;

use super::Registry;

pub fn structured_clone<'js>(ctx: Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Value<'js>> {
    let Some(registry) = ctx.userdata::<Registry>() else {
        throw!(ctx, "Registry not registered")
    };
    registry.structured_clone_value(&ctx, &value)
}
