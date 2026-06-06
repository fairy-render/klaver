use rquickjs::Ctx;

use super::NativeIteratorInterface;

pub struct NativeIteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    iter: T,
    ctx: Ctx<'js>,
}

impl<'js, T> NativeIteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    pub fn new(ctx: Ctx<'js>, iter: T) -> Self {
        NativeIteratorIter { iter, ctx }
    }
}

impl<'js, T> Iterator for NativeIteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    type Item = rquickjs::Result<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next(&self.ctx).transpose()
    }
}

impl<'js, T> Drop for NativeIteratorIter<'js, T>
where
    T: NativeIteratorInterface<'js>,
{
    fn drop(&mut self) {
        let _ = self.iter.returns(&self.ctx);
    }
}
