use std::cell::RefCell;

use rquickjs::{Ctx, IntoJs, class::Trace};

use crate::value::iterable::native::NativeIteratorInterface;

/// A wrapper around a native iterator that implements the `NativeIteratorInterface` trait.
pub struct FromNativeIter<T>(RefCell<T>);

impl<T> FromNativeIter<T> {
    pub fn new(item: T) -> FromNativeIter<T> {
        FromNativeIter(RefCell::new(item))
    }
}

impl<'js, T: Trace<'js>> Trace<'js> for FromNativeIter<T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.0.borrow().trace(tracer);
    }
}

impl<'js, T: Trace<'js>> NativeIteratorInterface<'js> for FromNativeIter<T>
where
    T: Iterator,
    T::Item: IntoJs<'js>,
{
    type Item = T::Item;

    fn next(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        Ok(self.0.borrow_mut().next())
    }

    fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}
