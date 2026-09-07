use klaver_core::{ExportTarget, Exportable, Inheritable, SuperClass};
use rquickjs::{
    Class, Ctx, JsLifetime, Object,
    class::{JsClass, Trace},
    prelude::Func,
};

use super::emitter::{Emitter, EventList};

#[rquickjs::class]
#[derive(Trace)]
pub struct EventTarget<'js> {
    listeners: EventList<'js>,
}

unsafe impl<'js> JsLifetime<'js> for EventTarget<'js> {
    type Changed<'to> = EventTarget<'to>;
}

#[rquickjs::methods]
impl<'js> EventTarget<'js> {
    #[qjs(constructor)]
    pub fn new() -> rquickjs::Result<EventTarget<'js>> {
        Ok(EventTarget {
            listeners: Default::default(),
        })
    }
}

impl<'js, T> Inheritable<'js, T> for EventTarget<'js>
where
    T: JsClass<'js> + Emitter<'js>,
{
    fn additional_override(_ctx: &Ctx<'js>, proto: &Object<'js>) -> rquickjs::Result<()> {
        proto.set("addEventListener", Func::new(T::add_event_listener))?;
        proto.set("removeEventListener", Func::new(T::remove_event_listener))?;
        proto.set("dispatchEvent", Func::new(T::dispatch_event))?;

        Ok(())
    }
}

impl<'js> SuperClass<'js> for EventTarget<'js> {}

impl<'js> Emitter<'js> for EventTarget<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listeners
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listeners
    }
}

impl<'js> Exportable<'js> for EventTarget<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::value::structured_clone::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: ExportTarget<'js>,
    {
        target.set(
            ctx,
            EventTarget::NAME,
            Class::<EventTarget>::create_constructor(ctx)?,
        )?;

        EventTarget::add_event_target_prototype(ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        abort_controller::{AbortController, AbortSignal},
        events::{Event, NativeEvent},
    };
    use klaver_core::Subclass;
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    /// Runs `body` as the contents of a plain function, with `EventTarget`, `Event`,
    /// `AbortController` and `AbortSignal` available as globals. `body` is expected to throw on
    /// failure (e.g. via a plain `if (...) throw ...`).
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
    fn listeners_run_synchronously() {
        run(r#"
            const t = new EventTarget();
            let called = false;
            t.addEventListener("x", () => { called = true; });
            t.dispatchEvent(new Event("x"));
            if (!called) throw new Error("listener did not run synchronously");
        "#);
    }

    #[test]
    fn plain_function_listener_receives_target_as_this() {
        // Arrow functions ignore call-time `this` binding, so the listener must be a plain
        // function expression to actually observe it.
        run(r#"
            const t = new EventTarget();
            let seenThis;
            t.addEventListener("x", function (e) { seenThis = this; });
            t.dispatchEvent(new Event("x"));
            if (seenThis !== t) throw new Error("listener's `this` was not the target");
        "#);
    }

    #[test]
    fn dispatch_event_returns_true_when_not_cancelled() {
        run(r#"
            const t = new EventTarget();
            t.addEventListener("x", () => {});
            const ret = t.dispatchEvent(new Event("x"));
            if (ret !== true) throw new Error(`dispatchEvent() returned ${ret}`);
        "#);
    }

    #[test]
    fn prevent_default_makes_dispatch_event_return_false_when_cancelable() {
        run(r#"
            const t = new EventTarget();
            t.addEventListener("x", (e) => e.preventDefault());
            const event = new Event("x", { cancelable: true });
            const ret = t.dispatchEvent(event);
            if (ret !== false) throw new Error(`dispatchEvent() returned ${ret}`);
            if (!event.defaultPrevented) throw new Error("defaultPrevented was false");
        "#);
    }

    #[test]
    fn prevent_default_is_a_no_op_when_not_cancelable() {
        run(r#"
            const t = new EventTarget();
            t.addEventListener("x", (e) => e.preventDefault());
            const event = new Event("x");
            const ret = t.dispatchEvent(event);
            if (ret !== true) throw new Error(`dispatchEvent() returned ${ret}`);
            if (event.defaultPrevented) throw new Error("defaultPrevented was true");
        "#);
    }

    #[test]
    fn event_init_reflects_bubbles_cancelable_composed() {
        run(r#"
            const event = new Event("x", { bubbles: true, cancelable: true, composed: true });
            if (!event.bubbles) throw new Error("bubbles was false");
            if (!event.cancelable) throw new Error("cancelable was false");
            if (!event.composed) throw new Error("composed was false");
            if (event.isTrusted) throw new Error("isTrusted was true");
            if (typeof event.timeStamp !== "number") throw new Error("timeStamp was not a number");

            const defaults = new Event("y");
            if (defaults.bubbles || defaults.cancelable || defaults.composed) {
                throw new Error("EventInit fields should default to false");
            }
        "#);
    }

    #[test]
    fn duplicate_identical_listeners_are_a_no_op() {
        run(r#"
            const t = new EventTarget();
            let n = 0;
            const fn = () => { n += 1; };
            t.addEventListener("x", fn);
            t.addEventListener("x", fn);
            t.dispatchEvent(new Event("x"));
            if (n !== 1) throw new Error(`listener ran ${n} times`);
        "#);
    }

    #[test]
    fn same_callback_with_different_capture_are_distinct_listeners() {
        run(r#"
            const t = new EventTarget();
            let n = 0;
            const fn = () => { n += 1; };
            t.addEventListener("x", fn, { capture: false });
            t.addEventListener("x", fn, { capture: true });
            t.dispatchEvent(new Event("x"));
            if (n !== 2) throw new Error(`listener ran ${n} times`);
        "#);
    }

    #[test]
    fn once_listener_is_removed_after_firing() {
        run(r#"
            const t = new EventTarget();
            let n = 0;
            t.addEventListener("x", () => { n += 1; }, { once: true });
            t.dispatchEvent(new Event("x"));
            t.dispatchEvent(new Event("x"));
            if (n !== 1) throw new Error(`listener ran ${n} times`);
        "#);
    }

    #[test]
    fn remove_event_listener_removes_matching_callback() {
        run(r#"
            const t = new EventTarget();
            let n = 0;
            const fn = () => { n += 1; };
            t.addEventListener("x", fn);
            t.removeEventListener("x", fn);
            t.dispatchEvent(new Event("x"));
            if (n !== 0) throw new Error(`listener ran ${n} times`);
        "#);
    }

    #[test]
    fn event_listener_object_form_is_called_with_correct_this() {
        run(r#"
            const t = new EventTarget();
            let seenThis;
            let seenType;
            const listener = {
                handleEvent(e) { seenThis = this; seenType = e.type; },
            };
            t.addEventListener("x", listener);
            t.dispatchEvent(new Event("x"));
            if (seenThis !== listener) throw new Error("handleEvent was not called with the listener as `this`");
            if (seenType !== "x") throw new Error(`event.type was ${seenType}`);
        "#);
    }

    #[test]
    fn stop_immediate_propagation_stops_remaining_listeners() {
        run(r#"
            const t = new EventTarget();
            const order = [];
            t.addEventListener("x", (e) => { order.push(1); e.stopImmediatePropagation(); });
            t.addEventListener("x", () => { order.push(2); });
            t.dispatchEvent(new Event("x"));
            if (order.join(",") !== "1") throw new Error(`order was ${order}`);
        "#);
    }

    #[test]
    fn a_throwing_listener_does_not_stop_others_or_propagate() {
        run(r#"
            const t = new EventTarget();
            let secondRan = false;
            t.addEventListener("x", () => { throw new Error("boom"); });
            t.addEventListener("x", () => { secondRan = true; });
            // Must not throw.
            const ret = t.dispatchEvent(new Event("x"));
            if (!secondRan) throw new Error("second listener did not run");
            if (ret !== true) throw new Error(`dispatchEvent() returned ${ret}`);
        "#);
    }

    #[test]
    fn listener_can_add_and_remove_listeners_on_the_same_target_reentrantly() {
        run(r#"
            const t = new EventTarget();
            let inner = 0;
            t.addEventListener("x", () => {
                // Must not panic on a re-entrant borrow.
                t.addEventListener("y", () => { inner += 1; });
                t.dispatchEvent(new Event("y"));
                t.removeEventListener("y", () => {});
            });
            t.dispatchEvent(new Event("x"));
            if (inner !== 1) throw new Error(`inner ran ${inner} times`);
        "#);
    }

    #[test]
    fn signal_option_skips_adding_an_already_aborted_listener() {
        run(r#"
            const controller = new AbortController();
            controller.abort();

            const t = new EventTarget();
            let called = false;
            t.addEventListener("x", () => { called = true; }, { signal: controller.signal });
            t.dispatchEvent(new Event("x"));
            if (called) throw new Error("listener should not have been added");
        "#);
    }

    #[test]
    fn signal_option_removes_listener_once_aborted() {
        run(r#"
            const controller = new AbortController();
            const t = new EventTarget();
            let n = 0;
            t.addEventListener("x", () => { n += 1; }, { signal: controller.signal });

            t.dispatchEvent(new Event("x"));
            if (n !== 1) throw new Error(`listener ran ${n} times before abort`);

            controller.abort();

            t.dispatchEvent(new Event("x"));
            if (n !== 1) throw new Error(`listener ran ${n} times after abort`);
        "#);
    }

    #[test]
    fn subclassing_event_in_js_keeps_base_behavior() {
        run(r#"
            class MyEvent extends Event {
                constructor(extra) {
                    super("custom", { cancelable: true });
                    this.extra = extra;
                }
            }

            const t = new EventTarget();
            let seenExtra;
            t.addEventListener("custom", (e) => { seenExtra = e.extra; e.preventDefault(); });

            const event = new MyEvent(42);
            const ret = t.dispatchEvent(event);

            if (seenExtra !== 42) throw new Error(`extra was ${seenExtra}`);
            if (ret !== false) throw new Error(`dispatchEvent() returned ${ret}`);
            if (!event.defaultPrevented) throw new Error("defaultPrevented was false");
        "#);
    }
}
