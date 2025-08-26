use klaver_util::{
    rquickjs::{self, Coerced, Ctx, String, Value, prelude::Opt},
    throw,
};

pub struct Assert {}

impl Assert {
    pub fn assert<'js>(
        &self,
        ctx: Ctx<'js>,
        expr: Coerced<bool>,
        msg: Opt<String<'js>>,
    ) -> rquickjs::Result<()> {
        if !expr.0 {
            throw!(ctx, "Assert error")
        }
        Ok(())
    }

    pub fn equal<'js>(
        &self,
        ctx: Ctx<'js>,
        actual: Value<'js>,
        expected: Value<'js>,
        msg: Opt<String<'js>>,
    ) -> rquickjs::Result<()> {
        if actual != expected {
            throw!(ctx, "Assert error")
        }
        Ok(())
    }
}
