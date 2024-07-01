pub use extensions::Extensions;
use rquickjs::{
    class::{Trace, Tracer},
    function::Opt,
    Class, Ctx, Object, Value,
};

use super::{
    format::{format, FormatOptions},
    timers::Timers,
};

const CORE_KEY: &str = "Core";

pub fn get_core<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Core<'js>>> {
    ctx.globals().get(CORE_KEY)
}

#[rquickjs::class]
pub struct Core<'js> {
    #[qjs(get)]
    timers: Class<'js, Timers<'js>>,
    extensions: Extensions,
}

impl<'js> Trace<'js> for Core<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.timers.trace(tracer)
    }
}

impl<'js> Core<'js> {
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<Core<'js>> {
        let timers = Class::instance(ctx.clone(), Timers::default())?;
        Ok(Core {
            timers,
            extensions: Extensions::default(),
        })
    }

    pub fn timers(&self) -> Class<'js, Timers<'js>> {
        self.timers.clone()
    }

    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

#[rquickjs::methods]
impl<'js> Core<'js> {
    pub fn format(
        &self,
        ctx: Ctx<'js>,
        value: Value<'js>,
        opt: Opt<FormatOptions>,
    ) -> rquickjs::Result<String> {
        format(ctx, value, opt)
    }
}
