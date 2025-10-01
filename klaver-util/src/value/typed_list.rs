use std::marker::PhantomData;

use rquickjs::{Array, Ctx, FromJs, IntoJs, Value, atom::PredefinedAtom, class::Trace};

use crate::{
    ArrayExt, BasePrimordials, FromJsIter, Iter, NativeIteratorExt, ObjectExt, Pair, core::Core,
};

pub type TypedListEntries<'js, T> = FromJsIter<Iter<'js>, Pair<usize, T>>;

pub type TypedListValues<'js, T> = FromJsIter<Iter<'js>, T>;

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

    pub fn pop(&self) -> rquickjs::Result<Option<T>> {
        self.i.pop()
    }

    pub fn len(&self) -> usize {
        self.i.len()
    }

    pub fn entries(&self) -> rquickjs::Result<TypedListEntries<'js, T>> {
        let iter: Iter<'js> = self.i.call_property(
            Core::instance(self.i.ctx())?
                .borrow()
                .primordials()
                .atom_entries
                .clone(),
            (),
        )?;
        Ok(iter.from_javascript())
    }

    pub fn values(&self) -> rquickjs::Result<TypedListValues<'js, T>> {
        let iter: Iter<'js> = self.i.call_property(PredefinedAtom::Values, ())?;
        Ok(iter.from_javascript())
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
    fn into_js(self, _ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.i.into_value())
    }
}

#[cfg(test)]
mod tests {
    use crate::NativeIteratorInterface;

    use super::*;

    #[test]
    fn entries() {
        let runtime = rquickjs::Runtime::new().unwrap();
        let context = rquickjs::Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                let list = TypedList::<i32>::new(ctx.clone())?;
                list.push(1)?;
                list.push(2)?;

                let entries = list.values()?;
                assert_eq!(entries.next(&ctx)?, Some(1));
                assert_eq!(entries.next(&ctx)?, Some(2));
                assert_eq!(entries.next(&ctx)?, None);

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }
}
