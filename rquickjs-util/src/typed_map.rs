use std::marker::PhantomData;

use rquickjs::{class::Trace, Ctx, FromJs, IntoJs};

use crate::{Map, MapEntries};

pub struct TypedMap<'js, K, T> {
    i: Map<'js>,
    ty: PhantomData<(K, T)>,
}

impl<'js, K, T> Clone for TypedMap<'js, K, T> {
    fn clone(&self) -> Self {
        TypedMap {
            i: self.i.clone(),
            ty: PhantomData,
        }
    }
}

impl<'js, K, T> Trace<'js> for TypedMap<'js, K, T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.i.trace(tracer)
    }
}

impl<'js, K, T> FromJs<'js> for TypedMap<'js, K, T> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        Ok(TypedMap {
            i: Map::from_js(ctx, value)?,
            ty: PhantomData,
        })
    }
}

impl<'js, K, T> IntoJs<'js> for TypedMap<'js, K, T> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        self.i.into_js(ctx)
    }
}

impl<'js, K, T> TypedMap<'js, K, T>
where
    T: FromJs<'js> + IntoJs<'js>,
    K: IntoJs<'js> + FromJs<'js>,
{
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<TypedMap<'js, K, T>> {
        let i = Map::new(&ctx)?;
        Ok(TypedMap { i, ty: PhantomData })
    }
    pub fn set(&self, key: K, value: T) -> rquickjs::Result<Option<T>> {
        self.i.set(key, value)?;
        Ok(None)
    }

    pub fn get(&self, key: K) -> rquickjs::Result<Option<T>> {
        self.i.get(key)
    }

    pub fn has(&self, key: K) -> rquickjs::Result<bool> {
        self.i.has(key)
    }

    pub fn del(&self, key: K) -> rquickjs::Result<()> {
        self.i.del(key)
    }

    pub fn entries(&self) -> rquickjs::Result<MapEntries<'js, K, T>> {
        self.i.entries()
    }

    pub fn clear(&self) -> rquickjs::Result<()> {
        self.i.clear()
    }
}
