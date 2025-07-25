use std::marker::PhantomData;

use rquickjs::{Ctx, IntoJs};

use crate::NativeIteratorInterface;

pub trait NativeIteratorExt<'js> {}

pub struct MapIter<T, F, U> {
    iter: T,
    map_fn: F,
    phantom: PhantomData<U>,
}

impl<'js, T, F, U> NativeIteratorInterface<'js> for MapIter<T, F, U>
where
    T: NativeIteratorInterface<'js>,
    F: FnMut(&Ctx<'js>, T::Item) -> rquickjs::Result<U>,
    U: IntoJs<'js>,
{
    type Item = U;

    fn next(&self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        todo!()
    }

    fn returns(&self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        todo!()
    }
}
