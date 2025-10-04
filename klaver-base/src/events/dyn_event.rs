use super::{Event, EventKey};
use klaver_util::{SuperClass, throw};
use rquickjs::{Class, Ctx, FromJs, IntoJs, Value, class::Trace};

#[derive(Trace, Clone)]
pub struct DynEvent<'js> {
    inner: Value<'js>,
}

impl<'js> DynEvent<'js> {
    pub fn ty(&self, ctx: &Ctx<'js>) -> rquickjs::Result<EventKey<'js>> {
        let Some(obj) = self.inner.as_object() else {
            throw!(@type ctx, "Expected object");
        };

        obj.get("type")
    }
}

impl<'js> AsRef<Value<'js>> for DynEvent<'js> {
    fn as_ref(&self) -> &Value<'js> {
        &self.inner
    }
}

impl<'js> FromJs<'js> for DynEvent<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if !Event::is_subclass(ctx, &value)? {
            return Err(rquickjs::Error::new_from_js_message(
                "value",
                "event",
                "Expected a sublcass of Event",
            ));
        }

        Ok(DynEvent { inner: value })
    }
}

impl<'js> IntoJs<'js> for DynEvent<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.inner)
    }
}

pub trait IntoDynEvent<'js> {
    fn into_dynevent(self, ctx: &Ctx<'js>) -> rquickjs::Result<DynEvent<'js>>;
}

impl<'js> IntoDynEvent<'js> for DynEvent<'js> {
    fn into_dynevent(self, _ctx: &Ctx<'js>) -> rquickjs::Result<DynEvent<'js>> {
        Ok(self)
    }
}

impl<'js> IntoDynEvent<'js> for Event<'js> {
    fn into_dynevent(self, ctx: &Ctx<'js>) -> rquickjs::Result<DynEvent<'js>> {
        let event = Class::instance(ctx.clone(), self)?.into_value();
        DynEvent::from_js(ctx, event)
    }
}
