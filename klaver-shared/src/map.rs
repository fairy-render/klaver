use core::fmt;

use rquickjs::{class::Trace, function::This, Ctx, FromJs, Function, IntoJs, Object, Value};

#[derive(Debug, Trace)]
pub struct Map<'js> {
    object: Object<'js>,
}

impl<'js> Map<'js> {
    pub fn get<T: FromJs<'js>>(&self, name: &Value<'js>) -> rquickjs::Result<T> {
        self.object.get::<_, Function>("get")?.call((name,))
    }

    pub fn has<T: FromJs<'js>>(&self, name: &Value<'js>) -> rquickjs::Result<T> {
        self.object.get::<_, Function>("get")?.call((name,))
    }

    pub fn set<T: IntoJs<'js>>(&self, name: &Value<'js>, value: T) -> rquickjs::Result<()> {
        self.object
            .get::<_, Function>("set")?
            .call::<_, Value>((name, value))?;
        Ok(())
    }

    pub fn is(ctx: &Ctx<'js>, value: &rquickjs::Value<'js>) -> rquickjs::Result<bool> {
        let map_ctor: Value<'_> = ctx.globals().get::<_, Value>("Map")?;

        let Some(obj) = value.as_object() else {
            return Ok(false);
        };

        Ok(obj.is_instance_of(&map_ctor))
    }

    pub fn to_string(&self) -> rquickjs::Result<String> {
        let func = self.object.get::<_, Function>("toString")?;
        func.call((This(self.object.clone()),))
    }
}

impl<'js> FromJs<'js> for Map<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let map_ctor: Value<'_> = ctx.globals().get::<_, Value>("Map")?;

        let obj = value
            .try_into_object()
            .map_err(|_| rquickjs::Error::new_from_js("value", "map"))?;

        if !obj.is_instance_of(&map_ctor) {
            return Err(rquickjs::Error::new_from_js("object", "map"));
        }

        Ok(Map { object: obj })
    }
}

impl<'js> IntoJs<'js> for Map<'js> {
    fn into_js(self, _ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object.into())
    }
}
