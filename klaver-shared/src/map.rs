use std::marker::PhantomData;

use rquickjs::{class::Trace, function::This, Array, Ctx, FromJs, Function, IntoJs, Object, Value};

use crate::util::ObjectExt;

#[derive(Debug, Trace, Clone)]
pub struct Map<'js> {
    object: Object<'js>,
}

impl<'js> Map<'js> {
    pub fn get<K: IntoJs<'js>, T: FromJs<'js>>(&self, name: K) -> rquickjs::Result<T> {
        self.object.get::<_, Function>("get")?.call((name,))
    }

    pub fn has<K: IntoJs<'js>>(&self, name: K) -> rquickjs::Result<bool> {
        self.object.get::<_, Function>("has")?.call((name,))
    }

    pub fn set<K: IntoJs<'js>, T: IntoJs<'js>>(&self, name: K, value: T) -> rquickjs::Result<()> {
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

pub struct TypedMap<'js, T> {
    i: Map<'js>,
    ty: PhantomData<T>,
}

impl<'js, T> Clone for TypedMap<'js, T> {
    fn clone(&self) -> Self {
        TypedMap {
            i: self.i.clone(),
            ty: PhantomData,
        }
    }
}

impl<'js, T> Trace<'js> for TypedMap<'js, T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.i.trace(tracer)
    }
}

impl<'js, T> TypedMap<'js, T>
where
    T: FromJs<'js> + IntoJs<'js>,
{
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<TypedMap<'js, T>> {
        let i = Object::new(ctx)?;
        Ok(TypedMap {
            i: Map { object: i },
            ty: PhantomData,
        })
    }
    pub fn set<K: IntoJs<'js>>(&self, key: K, value: T) -> rquickjs::Result<Option<T>> {
        self.i.set(key, value)?;
        Ok(None)
    }

    pub fn get<K: IntoJs<'js>>(&self, key: K) -> rquickjs::Result<Option<T>> {
        self.i.get(key)
    }

    pub fn has<K: IntoJs<'js>>(&self, key: K) -> rquickjs::Result<bool> {
        self.i.has(key)
    }
}

pub struct TypedStdArray<'js, T> {
    i: Array<'js>,
    ty: PhantomData<T>,
}

impl<'js, T> Clone for TypedStdArray<'js, T> {
    fn clone(&self) -> Self {
        TypedStdArray {
            i: self.i.clone(),
            ty: PhantomData,
        }
    }
}

impl<'js, T> TypedStdArray<'js, T>
where
    T: FromJs<'js> + IntoJs<'js>,
{
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<TypedStdArray<'js, T>> {
        let i = Array::new(ctx)?;
        Ok(TypedStdArray { i, ty: PhantomData })
    }

    pub fn push(&self, ctx: Ctx<'js>, value: T) -> rquickjs::Result<()> {
        self.i.as_object().call_property(ctx, "push", (value,))
    }

    pub fn len(&self) -> usize {
        self.i.len()
    }
}

impl<'js, T> Trace<'js> for TypedStdArray<'js, T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.i.trace(tracer)
    }
}

impl<'js, T> FromJs<'js> for TypedStdArray<'js, T> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let i = Array::from_js(ctx, value)?;

        Ok(TypedStdArray { i, ty: PhantomData })
    }
}

impl<'js, T> IntoJs<'js> for TypedStdArray<'js, T> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.i.into_value())
    }
}
