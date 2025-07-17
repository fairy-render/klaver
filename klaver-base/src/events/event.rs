use rquickjs::{
    Class, Ctx, FromJs, IntoJs, JsLifetime, String, Symbol, Value,
    class::{JsClass, Trace},
    object::{Accessor, Property},
    prelude::{Func, This},
};
use rquickjs_util::{Inheritable, StringRef, SuperClass, throw};

#[derive(Clone, Debug, Hash)]
pub enum EventKey<'js> {
    Symbol(Symbol<'js>),
    String(String<'js>),
}

impl<'js> Trace<'js> for EventKey<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Symbol(s) => s.trace(tracer),
            Self::String(s) => s.trace(tracer),
        }
    }
}

impl<'js> EventKey<'js> {
    fn from_value(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
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

    pub fn to_string(&self) -> rquickjs::Result<String<'js>> {
        match self {
            Self::String(s) => Ok(s.clone()),
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
    fn additional_override(ctx: &Ctx<'js>, proto: &rquickjs::Object<'js>) -> rquickjs::Result<()> {
        proto.prop("type", Accessor::new_get(T::ty).enumerable())?;

        Ok(())
    }
}

unsafe impl<'js> JsLifetime<'js> for Event<'js> {
    type Changed<'to> = Event<'to>;
}

impl<'js> Event<'js> {
    pub fn new_native(ctx: &Ctx<'js>, msg: impl AsRef<str>) -> rquickjs::Result<Event<'js>> {
        let string = String::from_str(ctx.clone(), msg.as_ref())?;

        Event::new(string)
    }
}

#[rquickjs::methods]
impl<'js> Event<'js> {
    #[qjs(constructor)]
    pub fn new(ty: String<'js>) -> rquickjs::Result<Event<'js>> {
        Ok(Event {
            ty: EventKey::String(ty),
        })
    }

    // #[qjs(get, rename = "type")]
    // pub fn ty(&self) -> rquickjs::Result<String<'js>> {
    //     self.ty.to_string()
    // }
}

impl<'js> NativeEvent<'js> for Event<'js> {
    fn ty(this: This<Class<'js, Self>>, _ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        this.borrow().ty.to_string()
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

#[derive(Trace, Clone)]
pub struct DynEvent<'js> {
    inner: Value<'js>,
}

impl<'js> DynEvent<'js> {
    pub fn ty(&self, ctx: &Ctx<'js>) -> rquickjs::Result<EventKey<'js>> {
        let Some(obj) = self.inner.as_object() else {
            throw!(@type ctx, "Expected object");
        };

        obj.get("type")
    }
}

impl<'js> AsRef<Value<'js>> for DynEvent<'js> {
    fn as_ref(&self) -> &Value<'js> {
        &self.inner
    }
}

impl<'js> FromJs<'js> for DynEvent<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if !Event::is_subclass(ctx, &value)? {
            return Err(rquickjs::Error::new_from_js_message(
                "value",
                "event",
                "Expected a sublcass of Event",
            ));
        }

        Ok(DynEvent { inner: value })
    }
}

impl<'js> IntoJs<'js> for DynEvent<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.inner)
    }
}

pub trait IntoDynEvent<'js> {
    fn into_dynevent(self, ctx: &Ctx<'js>) -> rquickjs::Result<DynEvent<'js>>;
}

impl<'js> IntoDynEvent<'js> for DynEvent<'js> {
    fn into_dynevent(self, _ctx: &Ctx<'js>) -> rquickjs::Result<DynEvent<'js>> {
        Ok(self)
    }
}

impl<'js> IntoDynEvent<'js> for Event<'js> {
    fn into_dynevent(self, ctx: &Ctx<'js>) -> rquickjs::Result<DynEvent<'js>> {
        let event = Class::instance(ctx.clone(), self)?.into_value();
        DynEvent::from_js(ctx, event)
    }
}
