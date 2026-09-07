use std::{
    cell::Cell,
    hash::Hash,
    time::{SystemTime, UNIX_EPOCH},
};

use klaver_core::{Inheritable, StringExt, SuperClass, value::StringRef};
use rquickjs::{
    Class, Ctx, FromJs, IntoJs, JsLifetime, Object, String, Value,
    class::{JsClass, Trace},
    object::Accessor,
    prelude::{Func, Opt, This},
};

use klaver_core::Exportable;

#[derive(Debug, Trace)]
pub struct EventKey<'js> {
    string: StringRef<'js>,
}

impl<'js> EventKey<'js> {
    pub fn new(string: StringRef<'js>) -> EventKey<'js> {
        EventKey { string }
    }

    pub fn from_str(ctx: Ctx<'js>, value: &str) -> rquickjs::Result<EventKey<'js>> {
        Ok(EventKey {
            string: String::from_str(ctx, value)?.str_ref()?,
        })
    }
}

impl<'js> From<StringRef<'js>> for EventKey<'js> {
    fn from(value: StringRef<'js>) -> Self {
        EventKey::new(value)
    }
}

impl<'js> EventKey<'js> {
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    pub fn to_js_string(&self) -> String<'js> {
        self.string.as_string().clone()
    }
}

impl<'js> Clone for EventKey<'js> {
    fn clone(&self) -> Self {
        // `StringRef` can't cheaply be cloned (it re-derives a `*const char` from the
        // underlying `JSString`), but that's exactly what `try_clone` does.
        EventKey {
            string: self.string.try_clone().expect("clone JS string"),
        }
    }
}

impl<'js> PartialEq for EventKey<'js> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'js, 'a> PartialEq<&'a str> for EventKey<'js> {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl<'js> PartialEq<str> for EventKey<'js> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'js> Eq for EventKey<'js> {}

impl<'js> Hash for EventKey<'js> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'js> FromJs<'js> for EventKey<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        Ok(EventKey {
            string: value.get()?,
        })
    }
}

impl<'js> IntoJs<'js> for EventKey<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        self.string.into_js(ctx)
    }
}

/// `EventInit`, per <https://dom.spec.whatwg.org/#dictdef-eventinit>.
#[derive(Debug, Default, Clone, Copy)]
pub struct EventInit {
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
}

impl<'js> FromJs<'js> for EventInit {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(EventInit {
            bubbles: obj.get::<_, Option<bool>>("bubbles")?.unwrap_or(false),
            cancelable: obj.get::<_, Option<bool>>("cancelable")?.unwrap_or(false),
            composed: obj.get::<_, Option<bool>>("composed")?.unwrap_or(false),
        })
    }
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

/// The base `Event` data, per <https://dom.spec.whatwg.org/#interface-event>.
///
/// Subclasses (e.g. `MessageEvent`) embed this as a field rather than duplicating its
/// fields/behavior, and implement [`NativeEvent::event`] to expose it - that's what lets
/// `preventDefault()`, `cancelable`, `defaultPrevented`, etc. work correctly on every event
/// subtype, not just plain `Event` instances (see the comment on [`NativeEvent`]).
#[derive(Debug)]
#[rquickjs::class]
pub struct Event<'js> {
    pub ty: EventKey<'js>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub default_prevented: Cell<bool>,
    pub stop_immediate_propagation: Cell<bool>,
    pub time_stamp: f64,
}

impl<'js> Trace<'js> for Event<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.ty.trace(tracer);
    }
}

impl<'js> SuperClass<'js> for Event<'js> {}

impl<'js, T> Inheritable<'js, T> for Event<'js>
where
    T: JsClass<'js> + NativeEvent<'js>,
{
    fn additional_override(_ctx: &Ctx<'js>, proto: &rquickjs::Object<'js>) -> rquickjs::Result<()> {
        T::add_event_prototype_to(proto)
    }
}

impl<'js> Exportable<'js> for Event<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::value::structured_clone::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(ctx, Event::NAME, Class::<Self>::create_constructor(ctx)?)?;
        Event::add_event_prototype(ctx)?;

        Ok(())
    }
}

unsafe impl<'js> JsLifetime<'js> for Event<'js> {
    type Changed<'to> = Event<'to>;
}

impl<'js> Event<'js> {
    pub fn new_native(ctx: &Ctx<'js>, msg: impl AsRef<str>) -> rquickjs::Result<Event<'js>> {
        let string = String::from_str(ctx.clone(), msg.as_ref())?;
        Event::new(string.str_ref()?, Opt(None))
    }
}

#[rquickjs::methods]
impl<'js> Event<'js> {
    #[qjs(constructor)]
    pub fn new(ty: StringRef<'js>, init: Opt<EventInit>) -> rquickjs::Result<Event<'js>> {
        let init = init.0.unwrap_or_default();
        Ok(Event {
            ty: EventKey { string: ty },
            bubbles: init.bubbles,
            cancelable: init.cancelable,
            composed: init.composed,
            default_prevented: Cell::new(false),
            stop_immediate_propagation: Cell::new(false),
            time_stamp: now_ms(),
        })
    }
}

impl<'js> NativeEvent<'js> for Event<'js> {
    fn ty(this: This<Class<'js, Self>>, _ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        Ok(this.borrow().ty.to_js_string())
    }

    fn event(&self) -> &Event<'js> {
        self
    }
}

/// Every event class (`Event` itself, and any subclass such as `MessageEvent`) implements this
/// so the base `Event` behavior - `type`, `bubbles`/`cancelable`/`composed`, `defaultPrevented`,
/// `preventDefault()`, etc. - works correctly regardless of the concrete Rust type behind the JS
/// object.
///
/// This indirection exists because rquickjs classes aren't really JS-prototype-polymorphic at
/// the Rust binding layer: a native method bound via `#[rquickjs::methods] impl Event` expects
/// `this` to literally *be* a `Class<'js, Event>`, so calling it on a `Class<'js, MessageEvent>`
/// (even though `MessageEvent.prototype`'s prototype chain includes `Event.prototype`) would
/// fail to unwrap `this`. Instead, each subtype implements [`NativeEvent::event`] to expose its
/// embedded [`Event`] data, and `add_event_prototype`/`add_event_prototype_to` bind every
/// Event-family method/accessor generically per concrete subtype (parameterized on `Self`, not
/// hardcoded to `Event`), the same trick already used for `type`.
pub trait NativeEvent<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    fn ty(this: This<Class<'js, Self>>, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>>;

    /// The shared `Event` data embedded in this concrete event type.
    fn event(&self) -> &Event<'js>;

    fn bubbles(this: This<Class<'js, Self>>) -> bool {
        this.borrow().event().bubbles
    }

    fn cancelable(this: This<Class<'js, Self>>) -> bool {
        this.borrow().event().cancelable
    }

    fn composed(this: This<Class<'js, Self>>) -> bool {
        this.borrow().event().composed
    }

    fn default_prevented(this: This<Class<'js, Self>>) -> bool {
        this.borrow().event().default_prevented.get()
    }

    fn is_trusted(_this: This<Class<'js, Self>>) -> bool {
        // Every event reaching script here was created by script (there's no notion of a
        // user-driven/browser-generated event in this runtime).
        false
    }

    fn time_stamp(this: This<Class<'js, Self>>) -> f64 {
        this.borrow().event().time_stamp
    }

    fn prevent_default(this: This<Class<'js, Self>>) {
        let this = this.borrow();
        let event = this.event();
        // Per spec, `preventDefault()` is a no-op unless the event is cancelable.
        if event.cancelable {
            event.default_prevented.set(true);
        }
    }

    fn stop_propagation(_this: This<Class<'js, Self>>) {
        // No-op: this runtime dispatches to a single target with no containing tree, so
        // there's nothing for propagation to continue on to in the first place.
    }

    fn stop_immediate_propagation(this: This<Class<'js, Self>>) {
        this.borrow().event().stop_immediate_propagation.set(true);
    }

    /// Internal, non-enumerable channel for [`super::DynEvent::stop_immediate_propagation_called`]
    /// to read the flag back generically after `stopImmediatePropagation()` is called - not part
    /// of the public `Event` API surface (the DOM spec doesn't expose a getter for this flag).
    fn stop_immediate_propagation_flag(this: This<Class<'js, Self>>) -> bool {
        this.borrow().event().stop_immediate_propagation.get()
    }

    fn add_event_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx)?.expect("Event.prototype");
        Self::add_event_prototype_to(&proto)
    }

    fn add_event_prototype_to(proto: &rquickjs::Object<'js>) -> rquickjs::Result<()> {
        // No "already installed" guard here: `proto.contains_key("type")` would find `type`
        // *inherited* from `Event.prototype` (property lookup walks the chain, like the JS `in`
        // operator) even before this subclass's own copy is added - wrongly skipping it. That
        // would leave e.g. `MessageEvent.prototype` relying on `Event.prototype`'s accessors,
        // which are bound to `Class<'js, Event>` specifically and fail to unwrap `this` for any
        // other concrete subtype. Each subtype needs its own copies, parameterized on `Self`.
        //
        // Every accessor must be `.configurable()`, matching WebIDL's interface-prototype-object
        // rules: this function can run again for the same `Self` against the *same* prototype
        // object (rquickjs caches class prototypes per `Runtime`, shared by every `Context` built
        // on it - see `klaver_vm::Vm::create_context`), and redefining a non-configurable accessor
        // with a new getter identity throws.
        proto.prop(
            "type",
            Accessor::new_get(Self::ty).enumerable().configurable(),
        )?;
        proto.prop(
            "bubbles",
            Accessor::new_get(Self::bubbles).enumerable().configurable(),
        )?;
        proto.prop(
            "cancelable",
            Accessor::new_get(Self::cancelable)
                .enumerable()
                .configurable(),
        )?;
        proto.prop(
            "composed",
            Accessor::new_get(Self::composed).enumerable().configurable(),
        )?;
        proto.prop(
            "defaultPrevented",
            Accessor::new_get(Self::default_prevented)
                .enumerable()
                .configurable(),
        )?;
        proto.prop(
            "isTrusted",
            Accessor::new_get(Self::is_trusted)
                .enumerable()
                .configurable(),
        )?;
        proto.prop(
            "timeStamp",
            Accessor::new_get(Self::time_stamp)
                .enumerable()
                .configurable(),
        )?;
        proto.set("preventDefault", Func::new(Self::prevent_default))?;
        proto.set("stopPropagation", Func::new(Self::stop_propagation))?;
        proto.set(
            "stopImmediatePropagation",
            Func::new(Self::stop_immediate_propagation),
        )?;
        proto.prop(
            "$$stopImmediatePropagation",
            Accessor::new_get(Self::stop_immediate_propagation_flag).configurable(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function};

    /// Regression test: `Event`'s prototype is cached per `Runtime` (rquickjs shares class
    /// prototype objects across every `Context` built on the same `Runtime`), but two separate
    /// `Context`s each independently run their own global registration, which used to try to
    /// redefine the (already-installed, non-configurable) `bubbles`/`cancelable`/etc. accessors a
    /// second time and throw. This is what `klaver_vm::Vm::create_context()` does in practice.
    #[test]
    fn add_event_prototype_is_idempotent_across_contexts_on_the_same_runtime() {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();

            let ctx1 = AsyncContext::full(&rt).await.unwrap();
            ctx1.async_with(async |ctx| {
                ctx.globals()
                    .set("Event", Class::<Event>::create_constructor(&ctx)?)?;
                Event::add_event_prototype(&ctx)?;
                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();

            // Second `Context` on the *same* `Runtime` - this used to throw.
            let ctx2 = AsyncContext::full(&rt).await.unwrap();
            ctx2.async_with(async |ctx| {
                ctx.globals()
                    .set("Event", Class::<Event>::create_constructor(&ctx)?)?;
                Event::add_event_prototype(&ctx)?;

                let test_fn: Function = ctx.eval(
                    r#"(() => {
                        const event = new Event("boom", { bubbles: true, cancelable: true });
                        if (event.type !== "boom") throw new Error(`type was ${event.type}`);
                        if (event.bubbles !== true) throw new Error(`bubbles was ${event.bubbles}`);
                        if (event.cancelable !== true) {
                            throw new Error(`cancelable was ${event.cancelable}`);
                        }
                        event.preventDefault();
                        if (event.defaultPrevented !== true) {
                            throw new Error(`defaultPrevented was ${event.defaultPrevented}`);
                        }
                    })"#,
                )?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();
        });
    }
}
