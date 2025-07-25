use std::hash::Hash;

use klaver_util::{Inheritable, StringExt, StringRef, SuperClass};
use rquickjs::{
    Class, Ctx, FromJs, IntoJs, JsLifetime, String, Symbol, Value,
    class::{JsClass, Trace},
    object::Accessor,
    prelude::This,
};

use crate::Exportable;

// #[derive(Clone, Debug, Hash)]
// pub enum EventKey<'js> {
//     Symbol(Symbol<'js>),
//     String(String<'js>),
// }

// impl<'js> Trace<'js> for EventKey<'js> {
//     fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
//         match self {
//             Self::Symbol(s) => s.trace(tracer),
//             Self::String(s) => s.trace(tracer),
//         }
//     }
// }

// impl<'js> EventKey<'js> {
//     fn from_value(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
//         if value.is_string() {
//             let key: String = value.get()?;
//             Ok(EventKey::String(key))
//         } else {
//             let sym = value
//                 .into_symbol()
//                 .ok_or_else(|| rquickjs::Error::new_from_js("value", "event key"))?;
//             Ok(EventKey::Symbol(sym))
//         }
//     }

//     pub fn to_string(&self) -> rquickjs::Result<String<'js>> {
//         match self {
//             Self::String(s) => Ok(s.clone()),
//             Self::Symbol(_) => panic!("Symbol"),
//         }
//     }
// }

// impl<'js> FromJs<'js> for EventKey<'js> {
//     fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
//         Self::from_value(ctx, value)
//     }
// }

// impl<'js> Eq for EventKey<'js> {}

// impl<'js> PartialEq for EventKey<'js> {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (EventKey::Symbol(symbol1), EventKey::Symbol(symbol2)) => symbol1 == symbol2,
//             (EventKey::String(str1), EventKey::String(str2)) => str1 == str2,
//             _ => false,
//         }
//     }
// }

#[derive(Debug, Trace)]
pub struct EventKey<'js> {
    string: StringRef<'js>,
}

impl<'js> EventKey<'js> {
    pub fn new(string: StringRef<'js>) -> EventKey<'js> {
        EventKey { string }
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
}

impl<'js> PartialEq for EventKey<'js> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
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

#[derive(Debug, Trace)]
#[rquickjs::class]
pub struct Event<'js> {
    pub ty: EventKey<'js>,
}

impl<'js> SuperClass<'js> for Event<'js> {}

impl<'js, T> Inheritable<'js, T> for Event<'js>
where
    T: JsClass<'js> + NativeEvent<'js>,
{
    fn additional_override(_ctx: &Ctx<'js>, proto: &rquickjs::Object<'js>) -> rquickjs::Result<()> {
        proto.prop("type", Accessor::new_get(T::ty).enumerable())?;

        Ok(())
    }
}

impl<'js> Exportable<'js> for Event<'js> {
    fn export<T>(ctx: &Ctx<'js>, _registry: &crate::Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::ExportTarget<'js>,
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
        Event::new(string.str_ref()?)
    }
}

#[rquickjs::methods]
impl<'js> Event<'js> {
    #[qjs(constructor)]
    pub fn new(ty: StringRef<'js>) -> rquickjs::Result<Event<'js>> {
        Ok(Event {
            ty: EventKey { string: ty },
        })
    }
}

impl<'js> NativeEvent<'js> for Event<'js> {
    fn ty(this: This<Class<'js, Self>>, _ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        Ok(this.borrow().ty.string.as_string().clone())
    }
}

pub trait NativeEvent<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    fn ty(this: This<Class<'js, Self>>, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>>;

    fn add_event_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx)?.expect("EventEmitter.prototype");
        proto.prop("type", Accessor::new_get(Self::ty).enumerable())?;

        Ok(())
    }
}
