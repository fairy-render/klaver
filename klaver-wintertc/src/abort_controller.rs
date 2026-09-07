use klaver_core::Subclass;
use rquickjs::{
    Class, Ctx, Function, JsLifetime, String,
    class::{JsClass, Trace},
    function::Opt,
};

use crate::{
    dom_exception::DOMException,
    events::{Emitter, Event, EventCallback, EventKey, EventList, EventTarget},
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

        AbortSignal::dispatch_native(&self.signal, &ctx, Event::new_native(&ctx, "abort")?)?;

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
}

unsafe impl<'js> JsLifetime<'js> for AbortSignal<'js> {
    type Changed<'to> = AbortSignal<'to>;
}

impl<'js> Trace<'js> for AbortSignal<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.listeners.trace(tracer);
        self.reason.trace(tracer);
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
        })
    }

    #[qjs(rename = "throwIfAborted")]
    pub fn throw_if_aborted(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if let Some(aborted) = &self.reason {
            return Err(ctx.throw(aborted.clone()));
        }
        Ok(())
    }

    #[qjs(get, rename = "onabort")]
    pub fn get_onabort(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<Function<'js>>> {
        Ok(self.get_handler_function(&EventKey::from_str(ctx, "abort")?))
    }

    #[qjs(set, rename = "onabort")]
    pub fn set_onabort(
        &mut self,
        ctx: Ctx<'js>,
        func: Option<Function<'js>>,
    ) -> rquickjs::Result<()> {
        self.set_handler(
            EventKey::from_str(ctx, "abort")?,
            func.map(EventCallback::Function),
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventTarget, NativeEvent};
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                ctx.globals().set(
                    "EventTarget",
                    Class::<EventTarget>::create_constructor(&ctx)?,
                )?;
                EventTarget::add_event_target_prototype(&ctx)?;

                // `AbortController::abort()` constructs `Event::new_native` directly (not via
                // JS), but that instance still needs `Event`'s prototype accessors (`type`,
                // etc.) installed for `event.type`/`defaultPrevented`/... to work.
                Event::add_event_prototype(&ctx)?;

                AbortSignal::inherit(&ctx)?;
                ctx.globals().set(
                    "AbortSignal",
                    Class::<AbortSignal>::create_constructor(&ctx)?,
                )?;
                ctx.globals().set(
                    "AbortController",
                    Class::<AbortController>::create_constructor(&ctx)?,
                )?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn onabort_fires_synchronously_exactly_once() {
        run(r#"
            const controller = new AbortController();
            let n = 0;
            let seenThis;
            controller.signal.onabort = function (e) {
                n += 1;
                seenThis = this;
            };
            controller.abort();
            if (n !== 1) throw new Error(`onabort ran ${n} times`);
            if (seenThis !== controller.signal) throw new Error("onabort's `this` was not the signal");
        "#);
    }

    #[test]
    fn onabort_and_add_event_listener_both_fire() {
        run(r#"
            const controller = new AbortController();
            const order = [];
            controller.signal.onabort = () => order.push("onabort");
            controller.signal.addEventListener("abort", () => order.push("listener"));
            controller.abort();
            if (order.join(",") !== "onabort,listener") throw new Error(`order was ${order}`);
        "#);
    }

    #[test]
    fn reassigning_onabort_replaces_the_previous_handler() {
        run(r#"
            const controller = new AbortController();
            let first = 0;
            let second = 0;
            controller.signal.onabort = () => { first += 1; };
            controller.signal.onabort = () => { second += 1; };
            controller.abort();
            if (first !== 0) throw new Error(`first ran ${first} times`);
            if (second !== 1) throw new Error(`second ran ${second} times`);
        "#);
    }

    #[test]
    fn clearing_onabort_with_null_removes_it() {
        run(r#"
            const controller = new AbortController();
            let n = 0;
            controller.signal.onabort = () => { n += 1; };
            controller.signal.onabort = null;
            controller.abort();
            if (n !== 0) throw new Error(`onabort ran ${n} times`);
            if (controller.signal.onabort !== undefined) throw new Error("onabort getter did not return undefined");
        "#);
    }

    #[test]
    fn onabort_getter_reflects_currently_assigned_function() {
        run(r#"
            const controller = new AbortController();
            if (controller.signal.onabort !== undefined) throw new Error("expected undefined before assignment");
            const fn = () => {};
            controller.signal.onabort = fn;
            if (controller.signal.onabort !== fn) throw new Error("getter did not return the assigned function");
        "#);
    }
}
