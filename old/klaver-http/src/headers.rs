use reggie::http::HeaderMap;
use rquickjs::{class::Trace, function::Opt, Class, Ctx};
use std::collections::HashMap;

#[derive(Trace, Default)]
#[rquickjs::class]
pub struct Headers<'js> {
    pub inner: HashMap<String, Vec<rquickjs::String<'js>>>,
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
                .entry(k.to_string())
                .or_default()
                .push(rquickjs::String::from_str(ctx.clone(), v)?);
        }

        Class::instance(ctx.clone(), Headers { inner })
    }
}

#[rquickjs::methods]
impl<'js> Headers<'js> {
    #[qjs(constructor)]
    pub fn new(init: Opt<rquickjs::Array<'js>>) -> Self {
        Self::default()
    }
    pub fn append(&mut self, key: String, value: rquickjs::String<'js>) {
        self.inner.entry(key).or_default().push(value);
    }
}
