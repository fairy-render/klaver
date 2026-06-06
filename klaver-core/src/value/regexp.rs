use rquickjs::{Ctx, FromJs, IntoJs, Object, Value, class::Trace};

use crate::{
    ObjectExt,
    value::{StringRef, primordials::BasePrimordials},
};

#[derive(Debug, Clone)]
pub struct RegExp<'js> {
    object: Object<'js>,
}

impl<'js> RegExp<'js> {
    pub fn new(ctx: &Ctx<'js>, pattern: &str, flags: Option<&str>) -> rquickjs::Result<Self> {
        let primordials = BasePrimordials::from_ctx(ctx)?;
        let regexp_obj = match flags {
            Some(flags) => primordials.construct_regexp((pattern, flags))?,
            None => primordials.construct_regexp((pattern,))?,
        };
        Ok(Self { object: regexp_obj })
    }

    pub fn exec(&self, input: &str) -> rquickjs::Result<Value<'js>> {
        self.object.call_property("exec", (input,))
    }

    pub fn test(&self, input: &str) -> rquickjs::Result<bool> {
        self.object.call_property("test", (input,))
    }

    pub fn flags(&self) -> rquickjs::Result<StringRef<'js>> {
        self.object.call_property("flags", ())
    }

    pub fn source(&self) -> rquickjs::Result<StringRef<'js>> {
        self.object.call_property("source", ())
    }

    pub fn is(ctx: &Ctx<'js>, value: &rquickjs::Value<'js>) -> rquickjs::Result<bool> {
        let regexp_ctor = BasePrimordials::from_ctx(ctx)?.constructor_regexp.clone();

        let Some(obj) = value.as_object() else {
            return Ok(false);
        };

        Ok(obj.is_instance_of(&regexp_ctor))
    }
}

impl<'js> Trace<'js> for RegExp<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.object.trace(tracer);
    }
}

impl<'js> FromJs<'js> for RegExp<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if !RegExp::is(ctx, &value)? {
            return Err(rquickjs::Error::new_from_js(value.type_name(), "RegExp"));
        }

        let obj = value
            .try_into_object()
            .map_err(|_| rquickjs::Error::new_from_js("value", "regexp"))?;

        Ok(RegExp { object: obj })
    }
}

impl<'js> IntoJs<'js> for RegExp<'js> {
    fn into_js(self, _ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object.into())
    }
}
