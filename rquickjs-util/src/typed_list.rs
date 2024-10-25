use std::marker::PhantomData;

use rquickjs::{
    array::ArrayIter, atom::PredefinedAtom, class::Trace, function::This, Array, Ctx, FromJs,
    Function, IntoJs, Object, Value,
};

use crate::util::{ArrayExt, ObjectExt};

pub struct TypedList<'js, T> {
    i: Array<'js>,
    ty: PhantomData<T>,
}

impl<'js, T> Clone for TypedList<'js, T> {
    fn clone(&self) -> Self {
        TypedList {
            i: self.i.clone(),
            ty: PhantomData,
        }
    }
}

impl<'js, T> TypedList<'js, T>
where
    T: FromJs<'js> + IntoJs<'js>,
{
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<TypedList<'js, T>> {
        let i = Array::new(ctx)?;
        Ok(TypedList { i, ty: PhantomData })
    }

    pub fn push(&self, value: T) -> rquickjs::Result<()> {
        self.i.push(value)
    }

    pub fn len(&self) -> usize {
        self.i.len()
    }

    pub fn iter(&self) -> ArrayIter<'js, T> {
        self.i.iter()
    }

    pub fn get(&self, index: usize) -> rquickjs::Result<Option<T>> {
        let Some(i) = self.i.get(index)? else {
            return Ok(None);
        };

        Ok(Some(i))
    }
}

impl<'js, T> Trace<'js> for TypedList<'js, T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.i.trace(tracer)
    }
}

impl<'js, T> FromJs<'js> for TypedList<'js, T> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let i = Array::from_js(ctx, value)?;

        Ok(TypedList { i, ty: PhantomData })
    }
}

impl<'js, T> IntoJs<'js> for TypedList<'js, T> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.i.into_value())
    }
}
