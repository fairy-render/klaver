use std::time::{Instant, SystemTime, UNIX_EPOCH};

use klaver_core::{Exportable, Subclass, throw};
use rquickjs::{
    Class, Ctx, JsLifetime, Object,
    class::{JsClass, Trace, Tracer},
};

use crate::events::{Emitter, EventList, EventTarget};

/// `Performance` (<https://w3c.github.io/hr-time/#performance-interface>), the interface the
/// WinterTC Minimum Common API requires `globalThis.performance` to implement.
///
/// QuickJS-ng already installs its own native `performance` global (`JS_AddPerformance` in
/// `quickjs.c`), with a working `now()`/`timeOrigin`. But its `timeOrigin` is seeded from
/// `CLOCK_MONOTONIC` (`js__hrtime_ns`), not wall-clock time, so it isn't actually epoch-relative -
/// `performance.timeOrigin + performance.now()` doesn't approximate `Date.now()` as
/// [HR-TIME's invariant](https://w3c.github.io/hr-time/#dfn-time-origin) requires. This replaces
/// the native object with one that keeps that invariant (`timeOrigin` from `SystemTime`, `now()`
/// from a monotonic `Instant` anchored at the same instant), and adds the `EventTarget`
/// inheritance and `toJSON()` the spec's WebIDL also calls for, neither of which the native
/// object has:
///
/// ```webidl
/// interface Performance : EventTarget {
///     DOMHighResTimeStamp now();
///     readonly attribute DOMHighResTimeStamp timeOrigin;
///     object toJSON();
/// };
/// ```
#[rquickjs::class]
pub struct Performance<'js> {
    listeners: EventList<'js>,
    origin_instant: Instant,
    #[qjs(get, rename = "timeOrigin")]
    time_origin: f64,
}

unsafe impl<'js> JsLifetime<'js> for Performance<'js> {
    type Changed<'to> = Performance<'to>;
}

impl<'js> Trace<'js> for Performance<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.listeners.trace(tracer);
    }
}

impl<'js> Performance<'js> {
    fn create() -> Performance<'js> {
        let time_origin = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0;

        Performance {
            listeners: Default::default(),
            origin_instant: Instant::now(),
            time_origin,
        }
    }
}

#[rquickjs::methods]
impl<'js> Performance<'js> {
    // Per spec, `Performance` has no constructor of its own; the only instance is the
    // `globalThis.performance` singleton `Performance::create` builds.
    #[qjs(constructor)]
    pub fn ctor(ctx: Ctx<'js>) -> rquickjs::Result<Performance<'js>> {
        throw!(@type ctx, "Illegal constructor")
    }

    pub fn now(&self) -> f64 {
        self.origin_instant.elapsed().as_secs_f64() * 1000.0
    }

    #[qjs(rename = "toJSON")]
    pub fn to_json(&self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let obj = Object::new(ctx)?;
        obj.set("timeOrigin", self.time_origin)?;
        Ok(obj)
    }
}

impl<'js> Emitter<'js> for Performance<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listeners
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listeners
    }
}

impl<'js> Subclass<'js, EventTarget<'js>> for Performance<'js> {}

impl<'js> Exportable<'js> for Performance<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        Performance::inherit(ctx)?;
        target.set(
            ctx,
            Performance::NAME,
            Class::<Performance>::create_constructor(ctx)?,
        )?;

        target.set(
            ctx,
            "performance",
            Class::instance(ctx.clone(), Performance::create())?,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, NativeEvent};
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

                ctx.globals()
                    .set("Event", Class::<Event>::create_constructor(&ctx)?)?;
                Event::add_event_prototype(&ctx)?;

                Performance::export(&ctx, &klaver_core::Registry::instance(&ctx)?, &ctx.globals())?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn now_is_a_non_negative_number_and_monotonic() {
        run(r#"
            const a = performance.now();
            const b = performance.now();
            if (typeof a !== "number") throw new Error("now() did not return a number");
            if (a < 0) throw new Error("now() was negative");
            if (b < a) throw new Error("now() went backwards");
        "#);
    }

    #[test]
    fn time_origin_plus_now_approximates_epoch_time() {
        run(r#"
            const estimate = performance.timeOrigin + performance.now();
            const delta = Math.abs(Date.now() - estimate);
            if (delta > 5000) throw new Error(`estimate was ${delta}ms off Date.now()`);
        "#);
    }

    #[test]
    fn to_json_returns_time_origin() {
        run(r#"
            const json = performance.toJSON();
            if (json.timeOrigin !== performance.timeOrigin) {
                throw new Error("toJSON().timeOrigin did not match performance.timeOrigin");
            }
        "#);
    }

    #[test]
    fn performance_is_an_event_target() {
        run(r#"
            if (!(performance instanceof EventTarget)) {
                throw new Error("performance is not an instance of EventTarget");
            }
            let called = false;
            performance.addEventListener("x", () => { called = true; });
            performance.dispatchEvent(new Event("x"));
            if (!called) throw new Error("listener did not run");
        "#);
    }

    #[test]
    fn constructor_throws() {
        run(r#"
            let threw = false;
            try {
                new Performance();
            } catch (err) {
                threw = err instanceof TypeError;
            }
            if (!threw) throw new Error("new Performance() did not throw a TypeError");
        "#);
    }
}
