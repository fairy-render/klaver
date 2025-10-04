use klaver_runtime::AsyncId;
use rquickjs::{FromJs, IntoJs, class::Trace};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeId(pub AsyncId);

impl<'js> Trace<'js> for TimeId {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> FromJs<'js> for TimeId {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        Ok(TimeId(AsyncId::from_js(ctx, value)?))
    }
}

impl<'js> IntoJs<'js> for TimeId {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        self.0.into_js(ctx)
    }
}
