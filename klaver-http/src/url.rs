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

    #[qjs(get, rename = "hash")]
    pub fn get_hash(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.fragment().map(|m| m.to_string()))
    }

    #[qjs(set, rename = "hash")]
    pub fn set_hash(&mut self, ctx: Ctx<'_>, search: Opt<String>) -> rquickjs::Result<()> {
        self.i.set_fragment(search.0.as_ref().map(|m| m.as_str()));
        Ok(())
    }

    #[qjs(get, rename = "host")]
    pub fn get_host(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.host_str().map(|m| m.to_string()))
    }

    #[qjs(set, rename = "host")]
    pub fn set_host(&mut self, ctx: Ctx<'_>, search: Opt<String>) -> rquickjs::Result<()> {
        throw_if!(
            ctx,
            self.i
                .set_host(search.0.as_ref().map(|m| m.as_str()))
                .map_err(|_| "invalid host")
        );
        Ok(())
    }

    #[qjs(get, rename = "username")]
    pub fn get_username(&self) -> rquickjs::Result<String> {
        Ok(self.i.username().to_string())
    }

    #[qjs(set, rename = "username")]
    pub fn set_username(&mut self, ctx: Ctx<'_>, search: String) -> rquickjs::Result<()> {
        throw_if!(
            ctx,
            self.i
                .set_username(&*search)
                .map_err(|_| "invalid username")
        );
        Ok(())
    }

    #[qjs(get, rename = "pathname")]
    pub fn get_pathname(&self) -> rquickjs::Result<String> {
        Ok(self.i.path().to_string())
    }

    #[qjs(set, rename = "pathname")]
    pub fn set_pathname(&mut self, ctx: Ctx<'_>, search: String) -> rquickjs::Result<()> {
        self.i.set_path(&*search);
        Ok(())
    }

    #[qjs(get, rename = "password")]
    pub fn get_password(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.password().map(|m| m.to_string()))
    }

    #[qjs(set, rename = "password")]
    pub fn set_password(&mut self, ctx: Ctx<'_>, search: Opt<String>) -> rquickjs::Result<()> {
        throw_if!(
            ctx,
            self.i
                .set_password(search.0.as_ref().map(|m| m.as_str()))
                .map_err(|_| "invalid password")
        );
        Ok(())
    }

    #[qjs(get, rename = "port")]
    pub fn get_port(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.port().map(|m| m.to_string()))
    }

    #[qjs(get, rename = "port")]
    pub fn set_port(&mut self, ctx: Ctx<'_>, port: Opt<String>) -> rquickjs::Result<()> {
        let port = if let Some(port) = port.0 {
            Some(throw_if!(ctx, port.parse::<u16>()))
        } else {
            None
        };
        throw_if!(ctx, self.i.set_port(port).map_err(|_| "invalid port"));
        Ok(())
    }

    #[qjs(get, rename = "protocol")]
    pub fn get_protocol(&self) -> rquickjs::Result<String> {
        Ok(self.i.scheme().to_string())
    }

    #[qjs(set, rename = "protocol")]
    pub fn set_protocol(&mut self, ctx: Ctx<'_>, search: String) -> rquickjs::Result<()> {
        throw_if!(
            ctx,
            self.i.set_scheme(&search).map_err(|_| "invalid schema")
        );
        Ok(())
    }

    #[qjs(get, rename = "search")]
    pub fn get_search(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.query().map(|m| m.to_string()))
    }

    #[qjs(set, rename = "search")]
    pub fn set_search(&mut self, search: String) -> rquickjs::Result<()> {
        self.i.set_query(Some(&search));
        Ok(())
    }

    #[qjs(get)]
    pub fn href(&self) -> rquickjs::Result<String> {
        Ok(self.i.to_string())
    }

    #[qjs(get)]
    pub fn origin(&self) -> rquickjs::Result<String> {
        Ok(self.i.origin().unicode_serialization())
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> rquickjs::Result<String> {
        self.href()
    }

    #[qjs(rename = "toJSON")]
    pub fn to_json(&self) -> rquickjs::Result<String> {
        self.href()
    }
}
