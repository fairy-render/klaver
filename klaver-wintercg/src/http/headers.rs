use reggie::http::HeaderMap;
use rquickjs::{class::Trace, function::Opt, Class, Ctx, FromJs, JsLifetime};
use std::collections::HashMap;

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

        let mut inner = HashMap::default();

        for k in obj.keys::<String>() {
            let k = k?;
            let v: rquickjs::String = obj.get(&k)?;
            inner
                .entry(k)
                .and_modify(|ve: &mut Vec<rquickjs::String<'js>>| {
                    ve.push(v.clone());
                })
                .or_insert_with(|| vec![v]);
        }

        Ok(HeadersInit {
            inner: Class::instance(ctx.clone(), Headers { inner })?,
        })
    }
}

#[derive(Trace, Debug, Default)]
#[rquickjs::class]
pub struct Headers<'js> {
    pub inner: HashMap<String, Vec<rquickjs::String<'js>>>,
}

unsafe impl<'js> JsLifetime<'js> for Headers<'js> {
    type Changed<'to> = Headers<'to>;
}

impl<'js> Headers<'js> {
    pub fn from_headers(
        ctx: &Ctx<'js>,
        headers: HeaderMap,
    ) -> rquickjs::Result<Class<'js, Headers<'js>>> {
        let mut inner: HashMap<String, Vec<rquickjs::String<'_>>> = HashMap::default();
        for (k, v) in headers {
            let Some(k) = k else { continue };
            let Ok(v) = v.to_str() else { continue };
            inner
                .entry(k.to_string().to_lowercase())
                .or_default()
                .push(rquickjs::String::from_str(ctx.clone(), v)?);
        }

        Class::instance(ctx.clone(), Headers { inner })
    }
}

#[rquickjs::methods]
impl<'js> Headers<'js> {
    #[qjs(constructor)]
    pub fn new(_init: Opt<rquickjs::Array<'js>>) -> Self {
        Self::default()
    }
    pub fn append(&mut self, key: String, value: rquickjs::String<'js>) {
        self.inner.entry(key).or_default().push(value);
    }

    pub fn get(&self, key: String) -> rquickjs::Result<Option<rquickjs::String<'js>>> {
        Ok(self.inner.get(&key).and_then(|m| m.first()).cloned())
    }

    pub fn has(&self, key: String) -> rquickjs::Result<bool> {
        Ok(self.inner.contains_key(&key))
    }
}
