use klaver::throw_if;
use rquickjs::{atom::PredefinedAtom, class::Trace, function::Opt, Class, Ctx, FromJs};

use super::url_search_params::{URLSearchParams, URLSearchParamsInit};

pub enum StringOrUrl<'js> {
    String(rquickjs::String<'js>),
    Url(Class<'js, Url<'js>>),
}

impl<'js> StringOrUrl<'js> {
    fn as_str(&self) -> rquickjs::Result<String> {
        match self {
            Self::String(s) => s.to_string(),
            Self::Url(u) => Ok(u.borrow().i.to_string()),
        }
    }

    pub fn to_url(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Url<'js>>> {
        match self {
            StringOrUrl::String(s) => Class::instance(
                ctx.clone(),
                Url::new(ctx.clone(), StringOrUrl::String(s.clone()), Opt(None))?,
            ),
            StringOrUrl::Url(url) => Ok(url.clone()),
        }
    }
}

impl<'js> FromJs<'js> for StringOrUrl<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(ret) = Class::<'js, Url>::from_js(ctx, value.clone()) {
            Ok(StringOrUrl::Url(ret))
        } else if let Ok(ret) = rquickjs::String::from_js(ctx, value) {
            Ok(StringOrUrl::String(ret))
        } else {
            Err(rquickjs::Error::new_from_js("value", "string or url"))
        }
    }
}

#[rquickjs::class(rename = "URL")]
pub struct Url<'js> {
    i: url::Url,
    #[qjs(rename = "searchParams")]
    search_params: Class<'js, URLSearchParams<'js>>,
}

impl<'js> Trace<'js> for Url<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.search_params.trace(tracer)
    }
}

impl<'js> Url<'js> {
    pub fn url(&self) -> &url::Url {
        &self.i
    }

    pub fn from_reggie(
        ctx: &Ctx<'js>,
        uri: &reggie::http::Uri,
    ) -> rquickjs::Result<Class<'js, Url<'js>>> {
        let i = throw_if!(ctx, url::Url::parse(&uri.to_string()));
        let search_params = Class::instance(
            ctx.clone(),
            URLSearchParams::new(URLSearchParamsInit::from_str(
                ctx.clone(),
                i.query().unwrap_or_default(),
            )?)?,
        )?;

        Class::instance(ctx.clone(), Url { i, search_params })
    }
}

#[rquickjs::methods]
impl<'js> Url<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        url: StringOrUrl<'js>,
        base: Opt<StringOrUrl<'js>>,
    ) -> rquickjs::Result<Url<'js>> {
        let i = if let Some(base) = base.0 {
            match base {
                StringOrUrl::String(s) => {
                    let out = throw_if!(ctx, url::Url::parse(&s.to_string()?));
                    throw_if!(ctx, out.join(&url.as_str()?))
                }
                StringOrUrl::Url(b) => {
                    throw_if!(ctx, b.borrow().i.join(&url.as_str()?))
                }
            }
        } else {
            match url {
                StringOrUrl::String(s) => {
                    throw_if!(ctx, url::Url::parse(&s.to_string()?))
                }
                StringOrUrl::Url(url) => url.borrow().i.clone(),
            }
        };

        let search_params = Class::instance(
            ctx.clone(),
            URLSearchParams::new(URLSearchParamsInit::from_str(
                ctx,
                i.query().unwrap_or_default(),
            )?)?,
        )?;

        Ok(Url { i, search_params })
    }

    #[qjs(get, rename = "hash")]
    pub fn get_hash(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.fragment().map(|m| m.to_string()))
    }

    #[qjs(set, rename = "hash")]
    pub fn set_hash(&mut self, _ctx: Ctx<'_>, search: Opt<String>) -> rquickjs::Result<()> {
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
    pub fn set_pathname(&mut self, _ctx: Ctx<'_>, search: String) -> rquickjs::Result<()> {
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

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&self) -> rquickjs::Result<String> {
        self.href()
    }

    #[qjs(rename = PredefinedAtom::ToJSON)]
    pub fn to_json(&self) -> rquickjs::Result<String> {
        self.href()
    }
}
