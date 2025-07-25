use rquickjs::Ctx;

use crate::iterator::native::{NativeIterator, NativeIteratorInterface};

pub struct IteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    iter: T,
    ctx: Ctx<'js>,
}

impl<'js, T> Iterator for IteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    type Item = rquickjs::Result<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next(&self.ctx).transpose()
    }
}
