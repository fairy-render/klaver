use core::fmt;
use klaver_util::rquickjs::{self, FromJs, IntoJs, Value, class::Trace};
use std::usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsyncId(pub(crate) usize);

impl AsyncId {
    pub const fn root() -> AsyncId {
        AsyncId(0)
    }
}

impl fmt::Display for AsyncId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'js> Trace<'js> for AsyncId {
    fn trace<'a>(&self, _tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for AsyncId {
    fn into_js(
        self,
        ctx: &klaver_util::rquickjs::Ctx<'js>,
    ) -> klaver_util::rquickjs::Result<klaver_util::rquickjs::Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self.0 as _))
    }
}

impl<'js> FromJs<'js> for AsyncId {
    fn from_js(_ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        Ok(AsyncId(value.get()?))
    }
}
