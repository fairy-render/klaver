use klaver_util::rquickjs::{self, Ctx, JsLifetime, Value, class::Trace};

pub enum LifeCicle<'js> {
    Registered,
    Started,
    Success,
    Failed(Value<'js>),
}

pub trait NativeReporter<'js> {
    fn started(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<()>;
    fn finished(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<()>;
    fn test_started(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<()>;
    fn test_finished(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<()>;
}

#[rquickjs::class(crate = "rquickjs")]
pub struct Reporter<'js> {
    ts: &'js (),
}

impl<'js> Trace<'js> for Reporter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for Reporter<'js> {
    type Changed<'to> = Reporter<'to>;
}
