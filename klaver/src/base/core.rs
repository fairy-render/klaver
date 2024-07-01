use rquickjs::{
    class::{Trace, Tracer},
    Class, Ctx, Object,
};

use super::timers::Timers;

const CORE_KEY: &str = "Core";

pub fn get_core<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Core<'js>>> {
    ctx.globals().get(CORE_KEY)
}

#[rquickjs::class]
pub struct Core<'js> {
    #[qjs(get)]
    timers: Class<'js, Timers<'js>>,
}

impl<'js> Core<'js> {
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<Core<'js>> {
        let timers = Class::instance(ctx.clone(), Timers::default())?;
        Ok(Core { timers })
    }

    pub fn timers(&self) -> Class<'js, Timers<'js>> {
        self.timers.clone()
    }
}

impl<'js> Trace<'js> for Core<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.timers.trace(tracer)
    }
}
#[rquickjs::methods]
impl<'js> Core<'js> {}
