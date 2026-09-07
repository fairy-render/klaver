use klaver_core::{Subclass, value::StringRef};
use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, Object, String, Value,
    class::{JsClass, Trace},
    prelude::{Opt, This},
};

use crate::events::{Event, EventInit, NativeEvent};

/// `CustomEvent`, per <https://dom.spec.whatwg.org/#customevent>.
#[derive(Debug, Trace, JsLifetime)]
#[rquickjs::class]
pub struct CustomEvent<'js> {
    pub base: Event<'js>,
    #[qjs(get)]
    pub detail: Value<'js>,
}

/// `CustomEventInit`, per <https://dom.spec.whatwg.org/#dictdef-customeventinit>.
pub struct CustomEventInit<'js> {
    pub base: EventInit,
    pub detail: Option<Value<'js>>,
}

impl<'js> FromJs<'js> for CustomEventInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value.clone())?;

        Ok(CustomEventInit {
            base: EventInit::from_js(ctx, value)?,
            detail: obj.get("detail")?,
        })
    }
}

#[rquickjs::methods]
impl<'js> CustomEvent<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        ty: StringRef<'js>,
        init: Opt<CustomEventInit<'js>>,
    ) -> rquickjs::Result<CustomEvent<'js>> {
        let (event_init, detail) = match init.0 {
            Some(init) => (init.base, init.detail),
            None => (EventInit::default(), None),
        };

        Ok(CustomEvent {
            base: Event::new(ty, Opt(Some(event_init)))?,
            detail: detail.unwrap_or_else(|| Value::new_null(ctx.clone())),
        })
    }

    /// Legacy method, per <https://dom.spec.whatwg.org/#dom-customevent-initcustomevent>. Per
    /// spec this is a no-op once the event's dispatch flag is set; this runtime has no notion of
    /// an in-flight event (`dispatchEvent` runs listeners synchronously to completion before
    /// returning), so there's never a dispatch in progress for it to no-op against here.
    #[qjs(rename = "initCustomEvent")]
    pub fn init_custom_event(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        ty: StringRef<'js>,
        bubbles: Opt<bool>,
        cancelable: Opt<bool>,
        detail: Opt<Value<'js>>,
    ) {
        let mut this = this.borrow_mut();
        this.base.ty = ty.into();
        this.base.bubbles = bubbles.0.unwrap_or(false);
        this.base.cancelable = cancelable.0.unwrap_or(false);
        this.detail = detail.0.unwrap_or_else(|| Value::new_null(ctx));
    }
}

impl<'js> NativeEvent<'js> for CustomEvent<'js> {
    fn ty(this: This<Class<'js, Self>>, _ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        Ok(this.borrow().base.ty.to_js_string())
    }

    fn event(&self) -> &Event<'js> {
        &self.base
    }
}

impl<'js> Subclass<'js, Event<'js>> for CustomEvent<'js> {}

impl<'js> klaver_core::Exportable<'js> for CustomEvent<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            CustomEvent::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        Self::inherit(ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Emitter, EventTarget};
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    /// Runs `body` as the contents of a plain function, with `EventTarget`, `Event` and
    /// `CustomEvent` available as globals. `body` is expected to throw on failure (e.g. via a
    /// plain `if (...) throw ...`).
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

                ctx.globals().set(
                    "CustomEvent",
                    Class::<CustomEvent>::create_constructor(&ctx)?,
                )?;
                CustomEvent::inherit(&ctx)?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn detail_defaults_to_null() {
        run(r#"
            const event = new CustomEvent("boom");
            if (event.detail !== null) throw new Error(`detail was ${event.detail}`);
            if (event.type !== "boom") throw new Error(`type was ${event.type}`);
        "#);
    }

    #[test]
    fn detail_and_event_init_fields_are_set() {
        run(r#"
            const event = new CustomEvent("boom", {
                detail: { foo: 1 },
                bubbles: true,
                cancelable: true,
            });
            if (event.detail.foo !== 1) throw new Error(`detail was ${JSON.stringify(event.detail)}`);
            if (event.bubbles !== true) throw new Error(`bubbles was ${event.bubbles}`);
            if (event.cancelable !== true) throw new Error(`cancelable was ${event.cancelable}`);
        "#);
    }

    #[test]
    fn is_instance_of_event_and_custom_event() {
        run(r#"
            const event = new CustomEvent("boom");
            if (!(event instanceof Event)) throw new Error("expected instanceof Event");
            if (!(event instanceof CustomEvent)) throw new Error("expected instanceof CustomEvent");
        "#);
    }

    #[test]
    fn dispatches_through_event_target_with_detail_intact() {
        run(r#"
            const target = new EventTarget();
            let seenDetail;
            target.addEventListener("boom", (e) => { seenDetail = e.detail; });
            target.dispatchEvent(new CustomEvent("boom", { detail: 42 }));
            if (seenDetail !== 42) throw new Error(`seenDetail was ${seenDetail}`);
        "#);
    }

    #[test]
    fn init_custom_event_legacy_method_reinitializes_fields() {
        run(r#"
            const event = new CustomEvent("boom");
            event.initCustomEvent("bang", true, true, "payload");
            if (event.type !== "bang") throw new Error(`type was ${event.type}`);
            if (event.bubbles !== true) throw new Error(`bubbles was ${event.bubbles}`);
            if (event.cancelable !== true) throw new Error(`cancelable was ${event.cancelable}`);
            if (event.detail !== "payload") throw new Error(`detail was ${event.detail}`);
        "#);
    }
}
