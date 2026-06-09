use klaver_core::Subclass;
use rquickjs::{
    Class, Ctx, Function, JsLifetime, String,
    class::{JsClass, Trace},
    function::{Args, Opt},
};

use crate::{
    dom_exception::DOMException,
    events::{DynEvent, Emitter, Event, EventList, EventTarget},
};

#[derive(Trace)]
#[rquickjs::class]
pub struct AbortController<'js> {
    #[qjs(get)]
    signal: Class<'js, AbortSignal<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for AbortController<'js> {
    type Changed<'to> = AbortController<'to>;
}

#[rquickjs::methods]
impl<'js> AbortController<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<AbortController<'js>> {
        Ok(AbortController {
            signal: Class::instance(ctx, AbortSignal::new()?)?,
        })
    }

    pub fn abort(&self, ctx: Ctx<'js>, reason: Opt<rquickjs::Value<'js>>) -> rquickjs::Result<()> {
        if self.signal.borrow().aborted {
            return Ok(());
        }
        self.signal.borrow_mut().aborted = true;
        self.signal.borrow_mut().reason = Some(if let Some(value) = reason.0 {
            value
        } else {
            let error = String::from_str(ctx.clone(), "AbortError")?;

            Class::instance(
                ctx.clone(),
                DOMException::new(ctx.clone(), Opt(None), Opt(Some(error)))?,
            )?
            .into_value()
        });

        let event = Class::instance(ctx.clone(), Event::new_native(&ctx, "abort")?)?;

        if let Some(onabort) = self.signal.borrow().onabort.as_ref().cloned() {
            let mut args = Args::new(ctx.clone(), 1);
            args.push_arg(event.clone())?;
            args.this(self.signal.clone())?;
            onabort.call_arg::<()>(args)?;
        }

        self.signal
            .borrow()
            .dispatch_native(&ctx, Event::new_native(&ctx, "abort")?)?;

        Ok(())
    }
}

klaver_core::create_export!(AbortController<'js>);

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

    fn dispatch(&self, ctx: &Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()> {
        if event.ty(ctx)? == "abort" {
            if let Some(onabort) = &self.onabort {
                onabort.defer((event,))?;
            }
        }

        Ok(())
    }
}

impl<'js> Subclass<'js, EventTarget<'js>> for AbortSignal<'js> {}

impl<'js> klaver_core::Exportable<'js> for AbortSignal<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
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
