use std::collections::HashMap;

use super::listener::{EventCallback, Listener, NativeListener};
use super::{DynEvent, IntoDynEvent};

use super::event::EventKey;

use crate::abort_controller::AbortSignal;
use rquickjs::class::{JsClass, Trace};
use rquickjs::prelude::{Func, Opt, This};
use rquickjs::{CaughtError, Class, Ctx, Object, Value};

#[derive(Clone, Trace)]
pub struct EventItem<'js> {
    pub callback: Listener<'js>,
    pub once: bool,
    /// Tracked purely for listener identity (spec: a listener is the tuple
    /// `(type, callback, capture)` - two `addEventListener` calls with the same type/callback
    /// but different `capture` are distinct listeners). This runtime has no target tree, so
    /// `capture` otherwise has no effect on dispatch order/phase.
    pub capture: bool,
}

pub type EventList<'js> = HashMap<EventKey<'js>, Vec<EventItem<'js>>>;

/// Options accepted by `addEventListener` (all fields) and `removeEventListener` (only
/// `capture` is meaningful there), per
/// <https://dom.spec.whatwg.org/#dictdef-addeventlisteneroptions>.
#[derive(Default, Clone)]
pub struct EventListenerOptions<'js> {
    pub capture: bool,
    pub once: bool,
    /// Accepted so callers that pass it don't get a type error, but otherwise inert: this
    /// runtime doesn't model passive-listener scroll-blocking semantics.
    pub passive: bool,
    pub signal: Option<Class<'js, AbortSignal<'js>>>,
}

impl<'js> rquickjs::FromJs<'js> for EventListenerOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        // Legacy shorthand: `addEventListener(type, cb, true)` means `{ capture: true }`.
        if let Some(capture) = value.as_bool() {
            return Ok(EventListenerOptions {
                capture,
                ..Default::default()
            });
        }

        let obj = Object::from_js(ctx, value)?;

        Ok(EventListenerOptions {
            capture: obj.get::<_, Option<bool>>("capture")?.unwrap_or(false),
            once: obj.get::<_, Option<bool>>("once")?.unwrap_or(false),
            passive: obj.get::<_, Option<bool>>("passive")?.unwrap_or(false),
            signal: obj.get("signal")?,
        })
    }
}

pub trait Emitter<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    fn get_listeners(&self) -> &EventList<'js>;
    fn get_listeners_mut(&mut self) -> &mut EventList<'js>;

    fn add_native_listener<T>(&mut self, event_name: EventKey<'js>, listener: T)
    where
        T: NativeListener<'js> + 'js,
    {
        self.get_listeners_mut()
            .entry(event_name)
            .or_default()
            .push(EventItem {
                callback: Listener::Native(std::rc::Rc::new(listener)),
                once: false,
                capture: false,
            });
    }

    /// Hook for a subclass to react to every dispatched event (e.g. `AbortSignal`'s `onabort`,
    /// `MessagePort`'s `onmessage`), invoked before the generic listener list. Not part of the
    /// public `EventTarget` API - `dispatchEvent`'s return value only reflects `preventDefault()`
    /// calls made through the normal listener list below.
    #[allow(unused)]
    fn dispatch(&self, ctx: &Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()> {
        Ok(())
    }

    /// Dispatches `event` to this target: runs the [`Self::dispatch`] hook, then synchronously
    /// invokes every listener registered for the event's type (in registration order,
    /// stopping early if a listener calls `stopImmediatePropagation()`), and returns whether the
    /// event was *not* cancelled (i.e. `false` iff it's cancelable and some listener called
    /// `preventDefault()`), matching `EventTarget.prototype.dispatchEvent`'s return value.
    ///
    /// Takes `this` as a `Class` handle (rather than `&self`/`&mut self`) so it can drop its
    /// mutable borrow of the target *before* invoking any listener: listener callbacks commonly
    /// call `addEventListener`/`removeEventListener`/`dispatchEvent` back on the same target
    /// (e.g. a `once` listener re-subscribing itself), and holding a borrow across those calls
    /// would panic on the reentrant `borrow_mut()`.
    fn dispatch_native<T>(
        this: &Class<'js, Self>,
        ctx: &Ctx<'js>,
        event: T,
    ) -> rquickjs::Result<bool>
    where
        T: IntoDynEvent<'js>,
    {
        let event = event.into_dynevent(ctx)?;

        this.borrow().dispatch(ctx, event.clone())?;

        let ty = event.ty(ctx)?;

        let to_call = {
            let mut this_mut = this.borrow_mut();
            match this_mut.get_listeners_mut().get_mut(&ty) {
                Some(listeners) => {
                    let snapshot = listeners.clone();
                    // `once` listeners fire at most once - drop them now, before invoking
                    // anything, so a listener can't observe itself (or another `once`
                    // listener for this same dispatch) as still registered.
                    listeners.retain(|item| !item.once);
                    snapshot
                }
                None => Vec::new(),
            }
        };

        for item in &to_call {
            if let Err(err) = item.callback.call(ctx.clone(), event.clone()) {
                // Per spec, an exception thrown by a listener is reported, not propagated to
                // the `dispatchEvent()` caller, and doesn't stop the remaining listeners.
                eprintln!("Uncaught {}", CaughtError::from_error(ctx, err));
            }

            if event.stop_immediate_propagation_called(ctx)? {
                break;
            }
        }

        Ok(!event.default_prevented(ctx)?)
    }

    fn add_event_target_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx)?.expect("EventEmitter.prototype");
        proto.set("addEventListener", Func::new(Self::add_event_listener))?;
        proto.set(
            "removeEventListener",
            Func::new(Self::remove_event_listener),
        )?;
        proto.set("dispatchEvent", Func::new(Self::dispatch_event))?;

        Ok(())
    }

    fn add_event_listener_native(
        this: &Class<'js, Self>,
        ctx: &Ctx<'js>,
        event_name: EventKey<'js>,
        listener: EventCallback<'js>,
        options: EventListenerOptions<'js>,
    ) -> rquickjs::Result<()> {
        if let Some(signal) = &options.signal {
            // Per spec: if the signal is already aborted, don't add the listener at all.
            if signal.borrow().aborted {
                return Ok(());
            }
        }

        {
            let mut this_mut = this.borrow_mut();
            let listeners = this_mut
                .get_listeners_mut()
                .entry(event_name.clone())
                .or_default();

            // Per spec, adding the exact same (type, callback, capture) combination again is a
            // no-op rather than registering a second, duplicate listener.
            if listeners
                .iter()
                .any(|item| item.capture == options.capture && item.callback == listener)
            {
                return Ok(());
            }

            listeners.push(EventItem {
                callback: Listener::Js(listener.clone()),
                once: options.once,
                capture: options.capture,
            });
        }

        if let Some(signal) = options.signal {
            signal.borrow_mut().add_native_listener(
                EventKey::from_str(ctx.clone(), "abort")?,
                RemoveOnAbort {
                    target: this.clone(),
                    event_name,
                    listener,
                    capture: options.capture,
                },
            );
        }

        Ok(())
    }

    fn add_event_listener(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event_name: EventKey<'js>,
        listener: EventCallback<'js>,
        options: Opt<EventListenerOptions<'js>>,
    ) -> rquickjs::Result<()> {
        Self::add_event_listener_native(
            &this,
            &ctx,
            event_name,
            listener,
            options.0.unwrap_or_default(),
        )
    }

    fn remove_event_listener_native(
        &mut self,
        event_name: EventKey<'js>,
        listener: EventCallback<'js>,
        capture: bool,
    ) {
        let Some(listeners) = self.get_listeners_mut().get_mut(&event_name) else {
            return;
        };

        listeners.retain(|item| !(item.capture == capture && item.callback == listener));
    }

    fn remove_event_listener(
        this: This<Class<'js, Self>>,
        event_name: EventKey<'js>,
        listener: EventCallback<'js>,
        options: Opt<EventListenerOptions<'js>>,
    ) -> rquickjs::Result<()> {
        let capture = options.0.map(|o| o.capture).unwrap_or(false);
        this.borrow_mut()
            .remove_event_listener_native(event_name, listener, capture);
        Ok(())
    }

    fn dispatch_event(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event: DynEvent<'js>,
    ) -> rquickjs::Result<bool> {
        Self::dispatch_native(&this, &ctx, event)
    }
}

/// A native listener registered on an `AbortSignal` (via the `signal` option of
/// `addEventListener`) that removes the original listener from its original target once the
/// signal fires its `abort` event.
struct RemoveOnAbort<'js, T: Emitter<'js>> {
    target: Class<'js, T>,
    event_name: EventKey<'js>,
    listener: EventCallback<'js>,
    capture: bool,
}

impl<'js, T: Emitter<'js>> NativeListener<'js> for RemoveOnAbort<'js, T> {
    fn on_event(&self, _ctx: Ctx<'js>, _event: DynEvent<'js>) -> rquickjs::Result<()> {
        self.target.borrow_mut().remove_event_listener_native(
            self.event_name.clone(),
            self.listener.clone(),
            self.capture,
        );
        Ok(())
    }
}
