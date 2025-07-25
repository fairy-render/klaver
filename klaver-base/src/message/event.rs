use klaver_util::Subclass;
use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, Object, String, Value,
    class::{JsClass, Trace},
    prelude::Opt,
    qjs,
};

use crate::{DynEvent, Event, Exportable, IntoDynEvent, NativeEvent};

#[derive(Debug, Trace, JsLifetime)]
#[rquickjs::class]
pub struct MessageEvent<'js> {
    pub ty: String<'js>,
    #[qjs(get)]
    pub data: Option<Value<'js>>,
}

#[derive(Default)]
pub struct MessageEventOptions<'js> {
    pub data: Option<Value<'js>>,
}

impl<'js> FromJs<'js> for MessageEventOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(MessageEventOptions {
            data: obj.get("data")?,
        })
    }
}

#[rquickjs::methods]
impl<'js> MessageEvent<'js> {
    #[qjs(constructor)]
    pub fn new(
        ty: String<'js>,
        ops: Opt<MessageEventOptions<'js>>,
    ) -> rquickjs::Result<MessageEvent<'js>> {
        let opts = ops.0.unwrap_or_default();

        Ok(MessageEvent {
            data: opts.data,
            ty,
        })
    }
}

impl<'js> NativeEvent<'js> for MessageEvent<'js> {
    fn ty(
        this: rquickjs::prelude::This<Class<'js, Self>>,
        _ctx: Ctx<'js>,
    ) -> rquickjs::Result<String<'js>> {
        Ok(this.borrow().ty.clone())
    }
}

impl<'js> IntoDynEvent<'js> for MessageEvent<'js> {
    fn into_dynevent(self, ctx: &Ctx<'js>) -> rquickjs::Result<DynEvent<'js>> {
        let event = Class::instance(ctx.clone(), self)?.into_value();
        DynEvent::from_js(ctx, event)
    }
}

impl<'js> Subclass<'js, Event<'js>> for MessageEvent<'js> {}

impl<'js> Exportable<'js> for MessageEvent<'js> {
    fn export<T>(ctx: &Ctx<'js>, _registry: &crate::Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::ExportTarget<'js>,
    {
        target.set(
            ctx,
            MessageEvent::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        Self::inherit(ctx)?;

        Ok(())
    }
}
