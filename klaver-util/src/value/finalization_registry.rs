use rquickjs::{
    Ctx, Function, Object, Value,
    class::{Trace, Tracer},
    function::Constructor,
};

use crate::ObjectExt;

pub struct FinalizationRegistry<'js> {
    obj: Object<'js>,
}

impl<'js> Trace<'js> for FinalizationRegistry<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.obj.trace(tracer);
    }
}

impl<'js> FinalizationRegistry<'js> {
    pub fn new(ctx: Ctx<'js>, hook: Function<'js>) -> rquickjs::Result<FinalizationRegistry<'js>> {
        let ctor: Constructor<'js> = ctx.globals().get("FinalizationRegistry")?;

        let obj = ctor.construct((hook,))?;

        Ok(FinalizationRegistry { obj })
    }

    pub fn register(
        &self,
        target: Value<'js>,
        value: Value<'js>,
        key: Option<Value<'js>>,
    ) -> rquickjs::Result<()> {
        self.obj
            .call_property::<_, _, ()>("register", (target, value, key))?;

        Ok(())
    }

    pub fn unregister(&self, value: Value<'js>) -> rquickjs::Result<()> {
        self.obj.call_property("unregister", (value,))
    }
}
