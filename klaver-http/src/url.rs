use klaver::throw_if;
use rquickjs::{class::Trace, function::Opt, Class, Ctx, FromJs};

pub enum StringOrUrl<'js> {
    String(String),
    Url(Class<'js, Url>),
}

impl<'js> StringOrUrl<'js> {
    fn as_str(&self) -> String {
        match self {
            Self::String(s) => s.as_str().to_string(),
            Self::Url(u) => u.borrow().i.to_string(),
        }
    }
}

impl<'js> FromJs<'js> for StringOrUrl<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(ret) = Class::<'js, Url>::from_js(ctx, value.clone()) {
            Ok(StringOrUrl::Url(ret))
        } else if let Ok(ret) = String::from_js(ctx, value) {
            Ok(StringOrUrl::String(ret))
        } else {
            Err(rquickjs::Error::new_from_js("value", "string or url"))
        }
    }
}

#[derive(Debug)]
#[rquickjs::class]
pub struct Url {
    i: url::Url,
}

impl<'js> Trace<'js> for Url {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Url {
    #[qjs(constructor)]
    pub fn new<'js>(
        ctx: Ctx<'js>,
        url: StringOrUrl<'js>,
        base: Opt<StringOrUrl<'js>>,
    ) -> rquickjs::Result<Url> {
        let i = if let Some(base) = base.0 {
            match base {
                StringOrUrl::String(s) => {
                    throw_if!(ctx, url::Url::parse(&s).and_then(|m| m.join(&url.as_str())))
                }
                StringOrUrl::Url(b) => throw_if!(ctx, b.borrow().i.join(&url.as_str())),
            }
        } else {
            match url {
                StringOrUrl::String(s) => {
                    throw_if!(ctx, url::Url::parse(&s))
                }
                StringOrUrl::Url(url) => url.borrow().i.clone(),
            }
        };

        Ok(Url { i })
    }

    #[qjs(get)]
    pub fn hash(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.fragment().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn host(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.host_str().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn password(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.password().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn port(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.port().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn protocol(&self) -> rquickjs::Result<String> {
        Ok(self.i.scheme().to_string())
    }

    #[qjs(get)]
    pub fn href(&self) -> rquickjs::Result<String> {
        Ok(self.i.to_string())
    }

    #[qjs(get)]
    pub fn search(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.query().map(|m| m.to_string()))
    }
}
