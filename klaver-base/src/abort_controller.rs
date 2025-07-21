use rquickjs::{
    Class, Ctx, JsLifetime,
    class::Trace,
    function::{Args, Opt},
};

use crate::{
    AbortSignal,
    dom_exception::DOMException,
    events::{Emitter, Event},
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
            Class::instance(
                ctx.clone(),
                DOMException::new(ctx.clone(), Opt(None), Opt(Some("AbortError".to_string())))?,
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

create_export!(AbortController<'js>);
