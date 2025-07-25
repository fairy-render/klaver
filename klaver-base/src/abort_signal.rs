use klaver_util::Subclass;
use rquickjs::{
    Class, Ctx, Function, JsLifetime,
    class::{JsClass, Trace},
};

use crate::{DynEvent, Emitter, EventList, EventTarget, export::Exportable};

#[rquickjs::class]
pub struct AbortSignal<'js> {
    listeners: EventList<'js>,
    #[qjs(get)]
    pub aborted: bool,
    #[qjs(get)]
    pub reason: Option<rquickjs::Value<'js>>,
    #[qjs(get, set)]
    pub onabort: Option<Function<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for AbortSignal<'js> {
    type Changed<'to> = AbortSignal<'to>;
}

impl<'js> Trace<'js> for AbortSignal<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.listeners.trace(tracer);
        self.reason.trace(tracer);
        self.onabort.trace(tracer);
        self.onabort.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> AbortSignal<'js> {
    #[qjs(constructor)]
    pub fn new() -> rquickjs::Result<AbortSignal<'js>> {
        Ok(AbortSignal {
            listeners: Default::default(),
            aborted: false,
            reason: None,
            onabort: None,
        })
    }

    #[qjs(rename = "throwIfAborted")]
    pub fn throw_if_aborted(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if let Some(aborted) = &self.reason {
            return Err(ctx.throw(aborted.clone()));
        }
        Ok(())
    }
}

impl<'js> Emitter<'js> for AbortSignal<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listeners
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listeners
    }

    fn dispatch(&self, event: DynEvent<'js>) -> rquickjs::Result<()> {
        if let Some(onabort) = &self.onabort {
            onabort.call::<_, ()>((event,))?;
        }

        Ok(())
    }
}

impl<'js> Subclass<'js, EventTarget<'js>> for AbortSignal<'js> {}

impl<'js> Exportable<'js> for AbortSignal<'js> {
    fn export<T>(ctx: &Ctx<'js>, _registry: &crate::Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::export::ExportTarget<'js>,
    {
        AbortSignal::inherit(ctx)?;
        target.set(
            ctx,
            AbortSignal::NAME,
            Class::<AbortSignal>::create_constructor(ctx)?,
        )?;
        Ok(())
    }
}
