use rquickjs::{
    atom::PredefinedAtom, class::Trace, function::Opt, Array, Atom, Class, Ctx, FromAtom, FromJs,
    JsLifetime, String as JsString,
};
use rquickjs_util::{string::concat, throw_if, util::ArrayExt, util::StringExt};

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

unsafe impl<'js> JsLifetime<'js> for Url<'js> {
    type Changed<'to> = Url<'to>;
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

    pub fn from_url(ctx: &Ctx<'js>, i: url::Url) -> rquickjs::Result<Class<'js, Url<'js>>> {
        let search_params = Class::instance(
            ctx.clone(),
            URLSearchParams::new(
                ctx.clone(),
                Opt(Some(URLSearchParamsInit::from_str(
                    ctx.clone(),
                    i.query().unwrap_or_default(),
                )?)),
            )?,
        )?;

        Class::instance(ctx.clone(), Url { i, search_params })
    }

    pub fn from_reggie(
        ctx: &Ctx<'js>,
        uri: &reggie::http::Uri,
    ) -> rquickjs::Result<Class<'js, Url<'js>>> {
        let i = throw_if!(ctx, url::Url::parse(&uri.to_string()));
        Self::from_url(ctx, i)
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
            URLSearchParams::new(
                ctx.clone(),
                Opt(Some(URLSearchParamsInit::from_str(
                    ctx,
                    i.query().unwrap_or_default(),
                )?)),
            )?,
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

    #[qjs(set, rename = "port")]
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

#[derive(Trace)]
#[rquickjs::class]
pub struct Url2<'js> {
    #[qjs(get, set)]
    protocol: JsString<'js>,
    #[qjs(get, set)]
    password: Option<JsString<'js>>,
    #[qjs(get, set)]
    hostname: Option<JsString<'js>>,
    #[qjs(get, set)]
    port: JsString<'js>,
    pathname: JsString<'js>,
    hash: JsString<'js>,
    search: JsString<'js>,
    #[qjs(get, rename = "searchParams")]
    search_params: Class<'js, URLSearchParams<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for Url2<'js> {
    type Changed<'to> = Url2<'to>;
}

impl<'js> Url2<'js> {
    pub fn to_stdstring(&self, ctx: Ctx<'js>) -> rquickjs::Result<String> {
        self.get_href(ctx)?.to_string()
    }
    pub fn from_str(ctx: Ctx<'js>, url: &str) -> rquickjs::Result<Url2<'js>> {
        let url = throw_if!(ctx, url::Url::parse(url));
        Self::from_url(ctx, &url)
    }

    pub fn from_url(ctx: Ctx<'js>, url: &url::Url) -> rquickjs::Result<Url2<'js>> {
        let hostname = if let Some(host) = url.host_str() {
            Some(JsString::from_str(ctx.clone(), host)?)
        } else {
            None
        };

        let protocol = JsString::from_str(ctx.clone(), url.scheme())?;
        let password = if let Some(password) = url.password() {
            Some(JsString::from_str(ctx.clone(), password)?)
        } else {
            None
        };

        let empty_string = JsString::from_str(ctx.clone(), "")?;

        let pathname = JsString::from_str(ctx.clone(), url.path())?;
        let hash = if let Some(hash) = url.fragment() {
            JsString::from_str(ctx.clone(), &format!("#{hash}"))?
        } else {
            empty_string.clone()
        };

        let port = if let Some(port) = url.port() {
            JsString::from_str(ctx.clone(), &format!("{port}"))?
        } else {
            empty_string.clone()
        };

        let search = if let Some(search) = url.query() {
            JsString::from_str(ctx.clone(), &format!("?{search}"))?
        } else {
            empty_string.clone()
        };

        let search_params = if let Some(query) = url.query() {
            let sp = URLSearchParams::new(
                ctx.clone(),
                Opt(Some(URLSearchParamsInit::from_str(ctx.clone(), query)?)),
            )?;
            Class::instance(ctx.clone(), sp)?
        } else {
            Class::instance(ctx.clone(), URLSearchParams::new(ctx.clone(), Opt(None))?)?
        };

        Ok(Url2 {
            protocol,
            password,
            hostname,
            port,
            pathname,
            hash,
            search,
            search_params,
        })
    }

    pub fn from_reggie(
        ctx: &Ctx<'js>,
        uri: &reggie::http::Uri,
    ) -> rquickjs::Result<Class<'js, Url2<'js>>> {
        let i = throw_if!(ctx, url::Url::parse(&uri.to_string()));
        Class::instance(ctx.clone(), Self::from_url(ctx.clone(), &i)?)
    }
}

#[rquickjs::methods]
impl<'js> Url2<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        url: StringOrUrl<'js>,
        base: Opt<StringOrUrl<'js>>,
    ) -> rquickjs::Result<Url2<'js>> {
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

        Url2::from_url(ctx, &i)
    }

    #[qjs(get, rename = "pathname")]
    pub fn get_pathname(&self) -> rquickjs::Result<JsString<'js>> {
        Ok(self.pathname.clone())
    }

    #[qjs(set, rename = "pathname")]
    pub fn set_pathname(&mut self, ctx: Ctx<'js>, mut path: JsString<'js>) -> rquickjs::Result<()> {
        let sep = JsString::from_atom(Atom::from_str(ctx.clone(), "/")?)?;
        if !path.starts_with(ctx.clone(), sep.clone())? {
            path = concat(ctx.clone(), sep, path)?;
        }

        self.pathname = path;

        Ok(())
    }

    #[qjs(get, rename = "host")]
    pub fn get_host(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<JsString<'js>>> {
        if self.port.length(ctx.clone())? != 0 {
            let output = Array::new(ctx.clone())?;
            output.push(self.hostname.clone())?;
            output.push(":")?;
            output.push(self.port.clone())?;
            return output.join("");
        }

        Ok(self.hostname.clone())
    }

    #[qjs(get, rename = "href")]
    pub fn get_href(&self, ctx: Ctx<'js>) -> rquickjs::Result<JsString<'js>> {
        let output = Array::new(ctx.clone())?;

        output.push(self.protocol.clone())?;
        output.push("://")?;

        output.push(self.hostname.clone())?;
        if self.port.length(ctx.clone())? != 0 {
            output.push(":")?;
            output.push(self.port.clone())?;
        }

        output.push(self.pathname.clone())?;
        output.push(self.hash.clone())?;
        output.push(self.search.clone())?;

        output.join("")
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&self, ctx: Ctx<'js>) -> rquickjs::Result<JsString<'js>> {
        self.get_href(ctx)
    }

    #[qjs(rename = PredefinedAtom::ToJSON)]
    pub fn to_json(&self, ctx: Ctx<'js>) -> rquickjs::Result<JsString<'js>> {
        self.get_href(ctx)
    }
}
