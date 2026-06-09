use rquickjs::Ctx;

use crate::iterator::native::NativeIteratorInterface;

pub struct IteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    iter: T,
    ctx: Ctx<'js>,
}

impl<'js, T> IteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    pub fn new(ctx: Ctx<'js>, iter: T) -> Self {
        IteratorIter { iter, ctx }
    }
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

impl<'js, T> Drop for IteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    fn drop(&mut self) {
        let _ = self.iter.returns(&self.ctx);
    }
}
