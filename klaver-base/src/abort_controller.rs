use rquickjs::{
    class::Trace,
    function::{Args, Opt, This},
    Class, Ctx, Function,
};

use crate::{
    dom_exception::DOMException,
    event_target::{Emitter, Event, EventList},
};

#[derive(Trace)]
#[rquickjs::class]
pub struct AbortController<'js> {
    #[qjs(get)]
    signal: Class<'js, AbortSignal<'js>>,
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
            Class::instance(
                ctx.clone(),
                DOMException::new(ctx.clone(), Opt(None), Opt(Some("AbortError".to_string())))?,
            )?
            .into_value()
        });

        let event = Class::instance(ctx.clone(), Event::new("abort".to_string())?)?;

        if let Some(onabort) = self.signal.borrow().onabort.as_ref().cloned() {
            let mut args = Args::new(ctx.clone(), 1);
            args.push_arg(event.clone())?;
            args.this(self.signal.clone())?;
            onabort.call_arg::<()>(args)?;
        }

        Emitter::dispatch_event(
            This(self.signal.clone()),
            Class::instance(ctx, Event::new("abort".to_string())?)?,
        )?;
        Ok(())
    }
}

#[rquickjs::class]
#[derive(Trace)]
pub struct AbortSignal<'js> {
    listeners: EventList<'js>,
    #[qjs(get)]
    aborted: bool,
    #[qjs(get)]
    reason: Option<rquickjs::Value<'js>>,
    #[qjs(get, set)]
    onabort: Option<Function<'js>>,
}

impl<'js> AbortSignal<'js> {}

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
}
