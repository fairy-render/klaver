use std::marker::PhantomData;

use rquickjs::{
    atom::PredefinedAtom, class::Trace, function::This, Array, Ctx, FromJs, Function, IntoJs,
    Object, Value,
};

use crate::util::ObjectExt;

#[derive(Debug, Trace, Clone, PartialEq, Eq)]
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
        let map_ctor: Value<'_> = ctx.globals().get::<_, Value>("globalThis.Map")?;

        let Some(obj) = value.as_object() else {
            return Ok(false);
        };

        Ok(obj.is_instance_of(&map_ctor))
    }

    pub fn entries<K, V>(&self, ctx: Ctx<'js>) -> rquickjs::Result<MapEntries<'js, K, V>>
    where
        K: FromJs<'js>,
        V: FromJs<'js>,
    {
        let iter = self
            .object
            .get::<_, Function>("get")?
            .call::<_, Object>((This(self.object.clone()),))?;

        let next = iter.get(PredefinedAtom::Next)?;

        Ok(MapEntries {
            ctx,
            this: iter,
            next,
            extract: PhantomData,
        })
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

pub struct MapEntries<'js, K, V> {
    ctx: Ctx<'js>,
    this: Object<'js>,
    next: Function<'js>,
    extract: PhantomData<(K, V)>,
}

impl<'js, K, V> Trace<'js> for MapEntries<'js, K, V> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.ctx.trace(tracer);
        self.this.trace(tracer);
        self.next.trace(tracer);
    }
}

impl<'js, K, V> Iterator for MapEntries<'js, K, V>
where
    K: FromJs<'js>,
    V: FromJs<'js>,
{
    type Item = rquickjs::Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.next.call::<_, Next<Entry<K, V>>>(()) {
            Ok(ret) => ret,
            Err(err) => return Some(Err(err)),
        };

        if next.done {
            None
        } else {
            next.value.map(|m| Ok((m.key, m.value)))
        }
    }
}

struct Entry<K, V> {
    key: K,
    value: V,
}

impl<'js, K, V> FromJs<'js> for Entry<K, V>
where
    K: FromJs<'js>,
    V: FromJs<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let array = Array::from_js(ctx, value)?;

        let key = array.get(0)?;
        let value = array.get(1)?;

        Ok(Entry { key, value })
    }
}

struct Next<V> {
    done: bool,
    value: Option<V>,
}

impl<'js, V> FromJs<'js> for Next<V>
where
    V: FromJs<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(Next {
            done: obj.get(PredefinedAtom::Done)?,
            value: obj.get(PredefinedAtom::Value)?,
        })
    }
}

#[cfg(test)]
mod test {
    use rquickjs::{Context, Runtime};

    use super::Map;

    #[test]
    fn test_map() {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        let ret = ctx
            .with(|ctx| {
                let set = Map::new(&ctx)?;

                set.set("key", "Hello, World!")?;

                set.has("key")
            })
            .unwrap();

        assert!(ret, "has")
    }
}
