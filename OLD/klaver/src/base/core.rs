use std::{
    collections::HashMap,
    sync::atomic::{AtomicU16, Ordering},
};

pub use extensions::Extensions;
use klaver_shared::{format, FormatOptions};
use rquickjs::{
    class::{Trace, Tracer},
    function::Opt,
    Class, Ctx, Function, IntoJs, Object, Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegistryId(u16);

impl RegistryId {
    fn new() -> RegistryId {
        static COUNTER: AtomicU16 = AtomicU16::new(1);
        RegistryId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl<'js> Trace<'js> for RegistryId {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

use super::timers::Timers;

const CORE_KEY: &str = "Core";

pub fn get_core<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Core<'js>>> {
    ctx.globals().get(CORE_KEY)
}

#[rquickjs::class]
pub struct Core<'js> {
    #[qjs(get)]
    timers: Class<'js, Timers<'js>>,
    extensions: Extensions,
    ref_table: HashMap<RegistryId, Value<'js>>,
}

impl<'js> Trace<'js> for Core<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.timers.trace(tracer);
        self.ref_table.trace(tracer);
    }
}

impl<'js> Core<'js> {
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<Core<'js>> {
        let timers = Class::instance(ctx.clone(), Timers::default())?;
        Ok(Core {
            timers,
            extensions: Extensions::default(),
            ref_table: Default::default(),
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

    pub fn register_value(&mut self, callback: Value<'js>) -> RegistryId {
        let id = RegistryId::new();
        self.ref_table.insert(id, callback);
        id
    }

    pub fn remove_value(&mut self, id: RegistryId) {
        self.ref_table.remove(&id);
    }

    pub fn get_value(&mut self, id: RegistryId) -> Option<Value<'js>> {
        self.ref_table.get(&id).cloned()
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
        format(ctx, value, opt.0)
    }
}
