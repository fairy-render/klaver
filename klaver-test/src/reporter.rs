use klaver_util::rquickjs::{self, Class, Ctx, FromJs, IntoJs, Value, class::Trace};

use crate::runner::{Suite, TestDesc};

pub enum LifeCicle<'js> {
    Registered,
    Started,
    Success,
    Failed(Value<'js>),
}

pub trait NativeReporter<'js>: Trace<'js> {
    fn prepare(
        &mut self,
        ctx: &Ctx<'js>,
        suites: &[Class<'js, Suite<'js>>],
    ) -> rquickjs::Result<()>;
}

// #[rquickjs::class(crate = "rquickjs")]
pub struct Reporter<'js> {
    pub ts: Option<Box<dyn NativeReporter<'js> + 'js>>,
}

impl<'js> Trace<'js> for Reporter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

// unsafe impl<'js> JsLifetime<'js> for Reporter<'js> {
//     type Changed<'to> = Reporter<'to>;
// }

impl<'js> FromJs<'js> for Reporter<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        todo!()
    }
}

impl<'js> IntoJs<'js> for Reporter<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        todo!()
    }
}

impl<'js> Reporter<'js> {
    pub fn prepare(
        &self,
        ctx: &Ctx<'js>,
        suites: &[Class<'js, Suite<'js>>],
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    pub fn test_started(
        &self,
        ctx: &Ctx<'js>,
        suites: &Class<'js, Suite<'js>>,
        test: &TestDesc<'js>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }
}
