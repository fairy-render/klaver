use klaver_core::{throw, value::equal as deep_equal_check};
use rquickjs::{Coerced, Ctx, Result, String, Value, prelude::Opt};

fn message(msg: Opt<String<'_>>, default: &str) -> Result<std::string::String> {
    match msg.0 {
        Some(msg) => msg.to_string(),
        None => Ok(default.into()),
    }
}

pub fn ok<'js>(ctx: Ctx<'js>, expr: Coerced<bool>, msg: Opt<String<'js>>) -> Result<()> {
    if !expr.0 {
        throw!(ctx, message(msg, "assertion failed")?)
    }
    Ok(())
}

pub fn equal<'js>(
    ctx: Ctx<'js>,
    actual: Value<'js>,
    expected: Value<'js>,
    msg: Opt<String<'js>>,
) -> Result<()> {
    if actual != expected {
        throw!(ctx, message(msg, "values are not equal")?)
    }
    Ok(())
}

pub fn deep_equal<'js>(
    ctx: Ctx<'js>,
    actual: Value<'js>,
    expected: Value<'js>,
    msg: Opt<String<'js>>,
) -> Result<()> {
    if !deep_equal_check(&ctx, actual, expected)? {
        throw!(ctx, message(msg, "values are not deeply equal")?)
    }
    Ok(())
}
