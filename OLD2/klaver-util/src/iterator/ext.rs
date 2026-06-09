use std::{fmt::Debug, marker::PhantomData};

use rquickjs::{Ctx, FromJs, IntoJs, class::Trace};

use crate::{IteratorIter, NativeIteratorInterface};

pub trait NativeIteratorExt<'js>: NativeIteratorInterface<'js> + Sized {
    fn map<T, U>(self, map: T) -> MapIter<Self, T, U> {
        MapIter {
            iter: self,
            map_fn: map,
            phantom: PhantomData,
        }
    }

    fn into_iter(self, ctx: &Ctx<'js>) -> IteratorIter<'js, Self> {
        IteratorIter::new(ctx.clone(), self)
    }

    fn from_javascript<U: FromJs<'js> + IntoJs<'js>>(self) -> FromJsIter<Self, U> {
        FromJsIter {
            iter: self,
            item: PhantomData,
        }
    }
}

impl<'js, T> NativeIteratorExt<'js> for T where T: NativeIteratorInterface<'js> {}

pub struct MapIter<T, F, U> {
    iter: T,
    map_fn: F,
    phantom: PhantomData<U>,
}

impl<'js, T: Trace<'js>, F, U> Trace<'js> for MapIter<T, F, U> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.iter.trace(tracer);
    }
}

impl<'js, T, F, U> NativeIteratorInterface<'js> for MapIter<T, F, U>
where
    T: NativeIteratorInterface<'js>,
    F: Fn(&Ctx<'js>, T::Item) -> rquickjs::Result<U>,
    U: IntoJs<'js>,
{
    type Item = U;

    fn next(&self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        if let Some(next) = self.iter.next(ctx)? {
            Ok(Some((self.map_fn)(ctx, next)?))
        } else {
            Ok(None)
        }
    }

    fn returns(&self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        self.iter.returns(ctx)
    }
}

pub struct FromJsIter<T, U> {
    iter: T,
    item: PhantomData<U>,
}

impl<'js, T: Trace<'js>, U> Trace<'js> for FromJsIter<T, U> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.iter.trace(tracer);
    }
}

impl<'js, T, U> NativeIteratorInterface<'js> for FromJsIter<T, U>
where
    T: NativeIteratorInterface<'js>,
    T::Item: Debug,
    U: FromJs<'js> + IntoJs<'js>,
{
    type Item = U;

    fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        if let Some(next) = self.iter.next(ctx)? {
            let value = next.into_js(ctx)?;
            let value = U::from_js(ctx, value)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn returns(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        self.iter.returns(ctx)
    }
}
