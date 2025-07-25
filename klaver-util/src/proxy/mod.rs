use std::marker::PhantomData;

use rquickjs::{
    Array, Ctx, FromJs, IntoJs, JsLifetime, String, Value, class::Trace, function::Constructor,
};

#[derive(Clone)]
pub enum Prop<'js> {
    String(rquickjs::String<'js>),
    Symbol(rquickjs::Symbol<'js>),
}

impl<'js> Prop<'js> {
    pub fn to_string(&self, ctx: &Ctx<'js>) -> rquickjs::Result<String<'js>> {
        match self {
            Self::String(s) => Ok(s.clone()),
            Self::Symbol(s) => rquickjs::String::from_js(ctx, s.description()?),
        }
    }
}

impl<'js> FromJs<'js> for Prop<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if value.is_string() {
            Ok(Prop::String(rquickjs::String::from_js(ctx, value)?))
        } else if value.is_symbol() {
            Ok(Prop::Symbol(rquickjs::Symbol::from_js(ctx, value)?))
        } else {
            Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "string or symbol",
            ))
        }
    }
}

impl<'js> IntoJs<'js> for Prop<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        match self {
            Self::String(s) => Ok(s.into_value()),
            Self::Symbol(s) => Ok(s.into_value()),
        }
    }
}

impl<'a, 'js> IntoJs<'js> for &'a Prop<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        match self.clone() {
            Prop::String(s) => Ok(s.into_value()),
            Prop::Symbol(s) => Ok(s.into_value()),
        }
    }
}

#[allow(unused)]
pub trait ProxyHandler<'js, T> {
    fn get(
        &self,
        ctx: Ctx<'js>,
        target: T,
        prop: Prop<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        Ok(Value::new_undefined(ctx))
    }

    fn set(
        &self,
        ctx: Ctx<'js>,
        target: T,
        prop: Prop<'js>,
        value: Value<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<bool> {
        Ok(false)
    }

    fn has(&self, ctx: Ctx<'js>, target: T, prop: Prop<'js>) -> rquickjs::Result<bool> {
        Ok(false)
    }

    fn delete(&self, ctx: Ctx<'js>, target: T, prop: Prop<'js>) -> rquickjs::Result<bool> {
        Ok(false)
    }

    fn apply(
        &self,
        ctx: Ctx<'js>,
        target: T,
        this: Value<'js>,
        args: Array<'js>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    fn own_keys(&self, ctx: Ctx<'js>, target: T) -> rquickjs::Result<Array<'js>> {
        Array::new(ctx)
    }
}

trait DynamicProxy<'js>: Trace<'js> {
    fn get(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        prop: Prop<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<Value<'js>>;

    fn set(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        prop: Prop<'js>,
        value: Value<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<bool>;

    fn delete(&self, ctx: Ctx<'js>, target: Value<'js>, prop: Prop<'js>) -> rquickjs::Result<bool>;

    fn has(&self, ctx: Ctx<'js>, target: Value<'js>, prop: Prop<'js>) -> rquickjs::Result<bool>;

    fn apply(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        this: Value<'js>,
        args: Array<'js>,
    ) -> rquickjs::Result<()>;

    fn own_keys(&self, ctx: Ctx<'js>, target: Value<'js>) -> rquickjs::Result<Array<'js>>;
}

struct HandlerBox<H, T>(H, PhantomData<T>);

impl<'js, H, T> Trace<'js> for HandlerBox<H, T>
where
    H: Trace<'js>,
{
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.0.trace(tracer);
    }
}

impl<'js, H, T> DynamicProxy<'js> for HandlerBox<H, T>
where
    H: ProxyHandler<'js, T> + Trace<'js>,
    T: FromJs<'js>,
{
    fn get(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        prop: Prop<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let target = T::from_js(&ctx, target)?;
        self.0.get(ctx, target, prop, receiver)
    }

    fn set(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        prop: Prop<'js>,
        value: Value<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<bool> {
        let target = T::from_js(&ctx, target)?;
        self.0.set(ctx, target, prop, value, receiver)
    }

    fn apply(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        this: Value<'js>,
        args: Array<'js>,
    ) -> rquickjs::Result<()> {
        let target = T::from_js(&ctx, target)?;
        self.0.apply(ctx, target, this, args)
    }

    fn own_keys(&self, ctx: Ctx<'js>, target: Value<'js>) -> rquickjs::Result<Array<'js>> {
        let target = T::from_js(&ctx, target)?;
        self.0.own_keys(ctx, target)
    }

    fn delete(&self, ctx: Ctx<'js>, target: Value<'js>, prop: Prop<'js>) -> rquickjs::Result<bool> {
        let target = T::from_js(&ctx, target)?;
        self.0.delete(ctx, target, prop)
    }

    fn has(&self, ctx: Ctx<'js>, target: Value<'js>, prop: Prop<'js>) -> rquickjs::Result<bool> {
        let target = T::from_js(&ctx, target)?;
        self.0.has(ctx, target, prop)
    }
}

#[rquickjs::class]
struct NativeProxy<'js> {
    i: Box<dyn DynamicProxy<'js> + 'js>,
}

unsafe impl<'js> JsLifetime<'js> for NativeProxy<'js> {
    type Changed<'to> = NativeProxy<'to>;
}

impl<'js> Trace<'js> for NativeProxy<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.i.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> NativeProxy<'js> {
    pub fn get(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        prop: Prop<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        self.i.get(ctx, target, prop, receiver)
    }

    pub fn set(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        prop: Prop<'js>,
        value: Value<'js>,
        receiver: Value<'js>,
    ) -> rquickjs::Result<bool> {
        self.i.set(ctx, target, prop, value, receiver)
    }

    fn apply(
        &self,
        ctx: Ctx<'js>,
        target: Value<'js>,
        this: Value<'js>,
        args: Array<'js>,
    ) -> rquickjs::Result<()> {
        self.i.apply(ctx, target, this, args)
    }

    fn delete(&self, ctx: Ctx<'js>, target: Value<'js>, prop: Prop<'js>) -> rquickjs::Result<bool> {
        self.i.delete(ctx, target, prop)
    }

    fn has(&self, ctx: Ctx<'js>, target: Value<'js>, prop: Prop<'js>) -> rquickjs::Result<bool> {
        self.i.has(ctx, target, prop)
    }

    #[qjs(rename = "ownKeys")]
    fn own_keys(&self, ctx: Ctx<'js>, target: Value<'js>) -> rquickjs::Result<Array<'js>> {
        self.i.own_keys(ctx, target)
    }
}

pub fn create_proxy<'js, T, H: ProxyHandler<'js, T>>(
    ctx: Ctx<'js>,
    target: T,
    handler: H,
) -> rquickjs::Result<Value<'js>>
where
    T: FromJs<'js> + IntoJs<'js> + 'js,
    H: ProxyHandler<'js, T> + Trace<'js> + 'js,
{
    let handler = HandlerBox::<H, T>(handler, PhantomData);

    let proxy = NativeProxy {
        i: Box::new(handler),
    };

    let func = ctx.globals().get::<_, Constructor<'js>>("Proxy")?;

    func.construct::<_, Value<'js>>((target, proxy))
}
