mod binding;
mod from_iter;
use rquickjs::{Ctx, IntoJs, class::Trace};
mod iter;
mod protocol;

pub use self::{
    binding::JsNativeIterator, from_iter::FromNativeIter, iter::NativeIteratorIter,
    protocol::IterableProtocol,
};

pub trait NativeIteratorInterface<'js>: Trace<'js> {
    type Item: IntoJs<'js>;

    fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>>;

    fn returns(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()>;
}
