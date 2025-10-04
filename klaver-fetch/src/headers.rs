use http::HeaderMap;
use klaver_base::Exportable;
use klaver_util::{
    IterableProtocol, NativeIterator, StringExt, TypedMultiMap, TypedMultiMapEntries,
};
use rquickjs::{
    Class, Coerced, Ctx, FromJs, JsLifetime, String,
    class::{JsClass, Trace},
    function::Opt,
};

#[derive(Trace)]
pub struct HeadersInit<'js> {
    pub inner: Class<'js, Headers<'js>>,
}

impl<'js> FromJs<'js> for HeadersInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(ret) = Class::<'js, Headers<'js>>::from_js(ctx, value.clone()) {
            return Ok(HeadersInit { inner: ret });
        }

        let Some(obj) = value.into_object() else {
            return Err(rquickjs::Error::new_from_js("value", "oject"));
        };

        let inner = TypedMultiMap::new(ctx.clone())?;

        for k in obj.keys::<String<'js>>() {
            let k = k?;
            let v: rquickjs::String = obj.get(k.clone())?;
            inner.append(ctx, k, v)?;
        }

        Ok(HeadersInit {
            inner: Class::instance(ctx.clone(), Headers { inner })?,
        })
    }
}

#[derive(Trace)]
#[rquickjs::class]
pub struct Headers<'js> {
    pub inner: TypedMultiMap<'js, String<'js>, String<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Headers<'js> {
    type Changed<'to> = Headers<'to>;
}

impl<'js> Headers<'js> {
    pub fn new_native(ctx: Ctx<'js>) -> rquickjs::Result<Headers<'js>> {
        Ok(Headers {
            inner: TypedMultiMap::new(ctx)?,
        })
    }

    pub fn from_native(
        ctx: &Ctx<'js>,
        headers: HeaderMap,
    ) -> rquickjs::Result<Class<'js, Headers<'js>>> {
        let inner = TypedMultiMap::new(ctx.clone())?;

        for (k, v) in headers {
            let Some(k) = k else { continue };
            let Ok(v) = v.to_str() else { continue };

            let k = String::from_str(ctx.clone(), &k.as_str().to_lowercase())?;
            let v = String::from_str(ctx.clone(), v)?;

            inner.append(ctx, k, v)?;
        }

        Class::instance(ctx.clone(), Headers { inner })
    }
}

#[rquickjs::methods]
impl<'js> Headers<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, _init: Opt<rquickjs::Array<'js>>) -> rquickjs::Result<Self> {
        Ok(Headers {
            inner: TypedMultiMap::new(ctx)?,
        })
    }
    pub fn append(
        &mut self,
        ctx: Ctx<'js>,
        key: String<'js>,
        Coerced(value): Coerced<String<'js>>,
    ) -> rquickjs::Result<()> {
        self.inner
            .append(&ctx, key, value.to_lowercase(ctx.clone())?)
    }

    pub fn set(
        &mut self,
        ctx: Ctx<'js>,
        key: String<'js>,
        Coerced(value): Coerced<String<'js>>,
    ) -> rquickjs::Result<()> {
        self.inner.set(&ctx, key, value)
    }

    pub fn get(
        &self,
        ctx: Ctx<'js>,
        key: String<'js>,
    ) -> rquickjs::Result<Option<rquickjs::String<'js>>> {
        self.inner.get(key.to_lowercase(ctx)?)
    }

    pub fn has(&self, ctx: Ctx<'js>, key: String<'js>) -> rquickjs::Result<bool> {
        self.inner.has(key.to_lowercase(ctx)?)
    }

    pub fn entries(&self, ctx: Ctx<'js>) -> rquickjs::Result<Class<'js, NativeIterator<'js>>> {
        Class::instance(
            ctx.clone(),
            NativeIterator::new(self.create_iterator(&ctx)?),
        )
    }

    pub fn values(&self) -> rquickjs::Result<NativeIterator<'js>> {
        Ok(NativeIterator::new(self.inner.values()?))
    }

    pub fn keys(&self) -> rquickjs::Result<NativeIterator<'js>> {
        Ok(NativeIterator::new(self.inner.keys()?))
    }
}

impl<'js> IterableProtocol<'js> for Headers<'js> {
    type Iterator = TypedMultiMapEntries<'js, String<'js>, String<'js>>;

    fn create_iterator(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
        self.inner.entries()
    }
}

// create_export!(Headers<'js>);

impl<'js> Exportable<'js> for Headers<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        target.set(ctx, Self::NAME, Class::<Self>::create_constructor(ctx)?)?;
        Self::add_iterable_prototype(ctx)?;

        Ok(())
    }
}
