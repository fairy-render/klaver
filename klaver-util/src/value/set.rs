use rquickjs::{
    Ctx, FromJs, Function, IntoJs, JsLifetime, Object, Value, class::Trace, function::This,
};

use crate::{Iter, ObjectExt};

#[derive(Debug, Trace, Clone, JsLifetime)]
pub struct Set<'js> {
    object: Object<'js>,
}

impl<'js> Set<'js> {
    pub fn new(ctx: &Ctx<'js>) -> rquickjs::Result<Set<'js>> {
        let obj = ctx.eval::<Object<'js>, _>("new globalThis.Set()")?;
        Ok(Self { object: obj })
    }

    pub fn has<T: IntoJs<'js>>(&self, name: T) -> rquickjs::Result<bool> {
        self.object
            .get::<_, Function>("has")?
            .call((This(self.object.clone()), name))
    }

    pub fn add<T: IntoJs<'js>>(&self, value: T) -> rquickjs::Result<()> {
        self.object
            .get::<_, Function>("add")?
            .call::<_, Value>((This(self.object.clone()), value))?;
        Ok(())
    }

    pub fn del(&self, name: &Value<'js>) -> rquickjs::Result<()> {
        self.object
            .get::<_, Function>("delete")?
            .call::<_, Value>((This(self.object.clone()), name))?;
        Ok(())
    }

    pub fn is(ctx: &Ctx<'js>, value: &rquickjs::Value<'js>) -> rquickjs::Result<bool> {
        let map_ctor: Value<'_> = ctx.globals().get::<_, Value>("Set")?;

        let Some(obj) = value.as_object() else {
            return Ok(false);
        };

        Ok(obj.is_instance_of(&map_ctor))
    }

    pub fn clear(&self) -> rquickjs::Result<()> {
        self.object
            .get::<_, Function>("clear")?
            .call::<_, Value>((This(self.object.clone()),))?;
        Ok(())
    }

    pub fn entries(&self) -> rquickjs::Result<Iter<'js>> {
        let iter = self
            .object
            .call_property::<_, _, Value<'js>>("entries", ())?;

        let iter = Iter::from_js(&self.object.ctx(), iter)?;

        Ok(iter)
    }

    pub fn to_object(&self) -> Object<'js> {
        self.object.clone()
    }

    pub fn to_string(&self) -> rquickjs::Result<String> {
        let func = self.object.get::<_, Function>("toString")?;
        func.call((This(self.object.clone()),))
    }
}

impl<'js> FromJs<'js> for Set<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if !Set::is(ctx, &value)? {
            return Err(rquickjs::Error::new_from_js(value.type_name(), "Set"));
        }

        let obj = value
            .try_into_object()
            .map_err(|_| rquickjs::Error::new_from_js("value", "set"))?;

        Ok(Set { object: obj })
    }
}

impl<'js> IntoJs<'js> for Set<'js> {
    fn into_js(self, _ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object.into())
    }
}
