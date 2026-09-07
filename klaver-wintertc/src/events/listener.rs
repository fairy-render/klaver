use std::rc::Rc;

use futures::SinkExt;
use rquickjs::{Ctx, FromJs, Function, Object, Value, class::Trace, function::Args};

use super::DynEvent;

/// The `callback` argument to `addEventListener`/`removeEventListener`, per
/// <https://dom.spec.whatwg.org/#callbackdef-eventlistener>: either a plain function, or an
/// object implementing the `EventListener` interface (a `handleEvent(event)` method), called
/// with `this` set to that object.
#[derive(Clone)]
pub enum EventCallback<'js> {
    Function(Function<'js>),
    Object(Object<'js>),
}

impl<'js> EventCallback<'js> {
    /// Invokes the callback. `target` is the `EventTarget` the listener was registered on
    /// (`event.currentTarget`); per spec, the plain-function form of a listener is called with
    /// `this` set to it. The `handleEvent` object form instead always uses the listener object
    /// itself as `this`, regardless of `target`.
    pub fn call(
        &self,
        ctx: &Ctx<'js>,
        target: Value<'js>,
        event: DynEvent<'js>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Function(f) => {
                let mut args = Args::new(ctx.clone(), 1);
                args.this(target)?;
                args.push_arg(event)?;
                f.call_arg::<Value>(args)?;
            }
            Self::Object(o) => {
                let handle_event: Function = o.get("handleEvent")?;
                let mut args = Args::new(ctx.clone(), 1);
                args.this(o.clone())?;
                args.push_arg(event)?;
                handle_event.call_arg::<Value>(args)?;
            }
        }
        Ok(())
    }
}

impl<'js> Trace<'js> for EventCallback<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Function(f) => f.trace(tracer),
            Self::Object(o) => o.trace(tracer),
        }
    }
}

impl<'js> PartialEq for EventCallback<'js> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Function(a), Self::Function(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl<'js> FromJs<'js> for EventCallback<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        // Functions are objects too, so check for a callable value first.
        if let Ok(f) = Function::from_js(ctx, value.clone()) {
            Ok(Self::Function(f))
        } else if let Some(o) = value.as_object() {
            Ok(Self::Object(o.clone()))
        } else {
            Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "function or EventListener",
            ))
        }
    }
}

pub enum Listener<'js> {
    Js(EventCallback<'js>),
    Native(Rc<dyn NativeListener<'js> + 'js>),
}

impl<'js> Listener<'js> {
    pub fn call(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        event: DynEvent<'js>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Js(js) => js.call(&ctx, target, event),
            Self::Native(native) => native.on_event(ctx, event),
        }
    }
}

impl<'js> Clone for Listener<'js> {
    fn clone(&self) -> Self {
        match self {
            Self::Js(js) => Self::Js(js.clone()),
            Self::Native(native) => Self::Native(native.clone()),
        }
    }
}

impl<'js> Trace<'js> for Listener<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Js(js) => js.trace(tracer),
            _ => {}
        }
    }
}

impl<'js> PartialEq<EventCallback<'js>> for Listener<'js> {
    fn eq(&self, other: &EventCallback<'js>) -> bool {
        match self {
            Self::Js(js) => js == other,
            _ => false,
        }
    }
}

pub trait NativeListener<'js> {
    fn on_event(&self, ctx: Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()>;
}

impl<'js> NativeListener<'js> for flume::Sender<DynEvent<'js>> {
    fn on_event(&self, ctx: Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()> {
        let this = self.clone();
        ctx.spawn(async move {
            this.send_async(event).await.ok();
        });

        Ok(())
    }
}

impl<'js> NativeListener<'js> for futures::channel::mpsc::Sender<DynEvent<'js>> {
    fn on_event(&self, ctx: Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()> {
        let mut this = self.clone();
        ctx.spawn(async move {
            this.send(event).await.ok();
        });

        Ok(())
    }
}
