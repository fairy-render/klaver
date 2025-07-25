use rquickjs::{
    Ctx, FromJs, Function, IntoJs, JsLifetime, Object, Value, class::Trace, function::This,
};

use crate::{Iter, object::ObjectExt};

#[derive(Debug, Trace, Clone, PartialEq, Eq, JsLifetime)]
pub struct Map<'js> {
    object: Object<'js>,
}

impl<'js> Map<'js> {
    pub fn new(ctx: &Ctx<'js>) -> rquickjs::Result<Map<'js>> {
        let obj = ctx.eval::<Object<'js>, _>("new globalThis.Map()")?;
        Ok(Self { object: obj })
    }

    pub fn get<K: IntoJs<'js>, T: FromJs<'js>>(&self, name: K) -> rquickjs::Result<T> {
        self.object
            .get::<_, Function>("get")?
            .call((This(self.object.clone()), name))
    }

    pub fn has<T: IntoJs<'js>>(&self, name: T) -> rquickjs::Result<bool> {
        self.object
            .get::<_, Function>("has")?
            .call((This(self.object.clone()), name))
    }

    pub fn set<K: IntoJs<'js>, V: IntoJs<'js>>(&self, name: K, value: V) -> rquickjs::Result<()> {
        self.object.get::<_, Function>("set")?.call::<_, Value>((
            This(self.object.clone()),
            name,
            value,
        ))?;
        Ok(())
    }

    pub fn del<K: IntoJs<'js>>(&self, name: K) -> rquickjs::Result<()> {
        self.object
            .get::<_, Function>("delete")?
            .call::<_, Value>((This(self.object.clone()), name))?;
        Ok(())
    }

    pub fn clear(&self) -> rquickjs::Result<()> {
        self.object
            .get::<_, Function>("clear")?
            .call::<_, Value>((This(self.object.clone()),))?;
        Ok(())
    }

    pub fn is(ctx: &Ctx<'js>, value: &rquickjs::Value<'js>) -> rquickjs::Result<bool> {
        let map_ctor: Value<'_> = ctx.globals().get::<_, Value>("Map")?;

        let Some(obj) = value.as_object() else {
            return Ok(false);
        };

        Ok(obj.is_instance_of(&map_ctor))
    }

    pub fn entries(&self) -> rquickjs::Result<Iter<'js>> {
        let iter = self
            .object
            .call_property::<_, _, Value<'js>>("entries", ())?;

        let iter = Iter::from_js(&self.object.ctx(), iter)?;

        Ok(iter)
    }

    pub fn values(&self) -> rquickjs::Result<Iter<'js>> {
        let iter = self
            .object
            .call_property::<_, _, Value<'js>>("values", ())?;

        let iter = Iter::from_js(&self.object.ctx(), iter)?;

        Ok(iter)
    }

    pub fn keys(&self) -> rquickjs::Result<Iter<'js>> {
        let iter = self.object.call_property::<_, _, Value<'js>>("keys", ())?;

        let iter = Iter::from_js(&self.object.ctx(), iter)?;

        Ok(iter)
    }

    pub fn to_string(&self) -> rquickjs::Result<String> {
        let func = self.object.get::<_, Function>("toString")?;
        func.call((This(self.object.clone()),))
    }

    pub fn to_object(&self) -> Object<'js> {
        self.object.clone()
    }
}

impl<'js> FromJs<'js> for Map<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if !Map::is(ctx, &value)? {
            return Err(rquickjs::Error::new_from_js(value.type_name(), "Map"));
        }

        let obj = value
            .try_into_object()
            .map_err(|_| rquickjs::Error::new_from_js("value", "map"))?;

        Ok(Map { object: obj })
    }
}

impl<'js> IntoJs<'js> for Map<'js> {
    fn into_js(self, _ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object.into())
    }
}
