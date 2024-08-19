use std::{collections::HashMap, rc::Rc, sync::Arc};

use klaver::quick::{Ctx, Symbol};
use rquickjs::{
    class::{JsClass, Trace},
    function::{Func, This},
    qjs, Class, FromJs, Function, Value,
};

pub struct NativeEvent {
    ty: Arc<str>,
}

#[rquickjs::class]
#[derive(Trace)]
pub struct EventTarget<'js> {
    listeners: EventList<'js>,
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

impl<'js> Emitter<'js> for EventTarget<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listeners
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listeners
    }
}

#[derive(Debug, Trace)]
#[rquickjs::class]
pub struct Event<'js> {
    ty: EventKey<'js>,
}

#[rquickjs::methods]
impl<'js> Event<'js> {
    #[qjs(constructor)]
    pub fn new(ty: String) -> rquickjs::Result<Event<'js>> {
        Ok(Event {
            ty: EventKey::String(ty.into()),
        })
    }

    #[qjs(get, rename = "type")]
    pub fn ty(&self) -> rquickjs::Result<String> {
        self.ty.to_string()
    }
}

pub enum Listener<'js> {
    Js(Function<'js>),
    Native(Box<dyn Fn(NativeEvent)>),
}

impl<'js> Listener<'js> {
    pub fn call(&self, event: Class<'js, Event<'js>>) -> rquickjs::Result<()> {
        match self {
            Self::Js(js) => js.call((event,)),
            Self::Native(native) => {
                (native)(NativeEvent {
                    ty: event.borrow().ty.to_string()?.into(),
                });
                Ok(())
            }
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

impl<'js> PartialEq<Function<'js>> for Listener<'js> {
    fn eq(&self, other: &Function<'js>) -> bool {
        match self {
            Self::Js(js) => js == other,
            _ => false,
        }
    }
}

pub trait Emitter<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    fn get_listeners(&self) -> &EventList<'js>;
    fn get_listeners_mut(&mut self) -> &mut EventList<'js>;

    fn add_event_target_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx.clone()).unwrap();
        proto.set("addEventListener", Func::new(Self::add_event_listener))?;
        proto.set(
            "removeEventListener",
            Func::new(Self::remove_event_listener),
        )?;
        proto.set("dispatchEvent", Func::new(Self::dispatch_event))?;

        Ok(())
    }

    fn add_event_listener(
        this: This<Class<'js, Self>>,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        this.0
            .borrow_mut()
            .get_listeners_mut()
            .entry(event_name)
            .or_default()
            .push(EventItem {
                callback: Listener::Js(listener),
                once: false,
            });

        Ok(())
    }

    fn remove_event_listener(
        this: This<Class<'js, Self>>,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        let mut this = this.0.borrow_mut();
        let Some(listeners) = this.get_listeners_mut().get_mut(&event_name) else {
            return Ok(());
        };

        listeners.retain(|m| m.callback != listener);

        Ok(())
    }

    fn dispatch_event(
        this: This<Class<'js, Self>>,
        event: Class<'js, Event<'js>>,
    ) -> rquickjs::Result<()> {
        let this = this.0.borrow();
        let Some(listeners) = this.get_listeners().get(&event.borrow().ty) else {
            return Ok(());
        };

        for listener in listeners {
            listener.callback.call(event.clone())?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Hash)]
pub enum EventKey<'js> {
    Symbol(Symbol<'js>),
    String(Rc<str>),
}

impl<'js> Trace<'js> for EventKey<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Symbol(s) => s.trace(tracer),
            _ => {}
        }
    }
}

impl<'js> EventKey<'js> {
    fn from_value(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if value.is_string() {
            let key: String = value.get()?;
            Ok(EventKey::String(key.into()))
        } else {
            let sym = value
                .into_symbol()
                .ok_or_else(|| rquickjs::Error::new_from_js("value", "event key"))?;
            Ok(EventKey::Symbol(sym))
        }
    }

    pub fn to_string(&self) -> rquickjs::Result<String> {
        match self {
            Self::String(s) => Ok(s.to_string()),
            Self::Symbol(s) => panic!("string"),
        }
    }
}

impl<'js> FromJs<'js> for EventKey<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        Self::from_value(ctx, value)
    }
}

impl<'js> Eq for EventKey<'js> {}

impl<'js> PartialEq for EventKey<'js> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventKey::Symbol(symbol1), EventKey::Symbol(symbol2)) => symbol1 == symbol2,
            (EventKey::String(str1), EventKey::String(str2)) => str1 == str2,
            _ => false,
        }
    }
}

#[derive(Trace)]
pub struct EventItem<'js> {
    callback: Listener<'js>,
    once: bool,
}

pub type EventList<'js> = HashMap<EventKey<'js>, Vec<EventItem<'js>>>;
