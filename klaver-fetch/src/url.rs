use klaver_base::create_export;
use klaver_util::{ArrayExt, StringExt, concat, throw_if};
use rquickjs::{
    Array, Atom, Class, Ctx, FromAtom, FromJs, JsLifetime, String as JsString,
    atom::PredefinedAtom, class::Trace, function::Opt,
};

use super::url_search_params::{URLSearchParams, URLSearchParamsInit};

pub enum StringOrUrl<'js> {
    String(rquickjs::String<'js>),
    Url(Class<'js, Url<'js>>),
}

impl<'js> StringOrUrl<'js> {
    pub fn as_str(&self, ctx: &Ctx<'js>) -> rquickjs::Result<String> {
        match self {
            Self::String(s) => s.to_string(),
            Self::Url(u) => u.borrow().to_stdstring(ctx),
        }
    }

    pub fn to_url(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Url<'js>>> {
        match self {
            StringOrUrl::String(s) => {
                Url::new(ctx.clone(), StringOrUrl::String(s.clone()), Opt(None))
            }
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

#[derive(Trace)]
#[rquickjs::class(rename = "URL")]
pub struct Url<'js> {
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

unsafe impl<'js> JsLifetime<'js> for Url<'js> {
    type Changed<'to> = Url<'to>;
}

impl<'js> Url<'js> {
    pub fn to_stdstring(&self, ctx: &Ctx<'js>) -> rquickjs::Result<String> {
        self.get_href(ctx.clone())?.to_string()
    }
    pub fn from_str(ctx: &Ctx<'js>, url: &str) -> rquickjs::Result<Url<'js>> {
        let url = throw_if!(ctx, url::Url::parse(url));
        Self::from_url(ctx, &url)
    }

    pub fn from_url(ctx: &Ctx<'js>, url: &url::Url) -> rquickjs::Result<Url<'js>> {
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

        Ok(Url {
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
}

#[rquickjs::methods]
impl<'js> Url<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        url: StringOrUrl<'js>,
        base: Opt<StringOrUrl<'js>>,
    ) -> rquickjs::Result<Class<'js, Url<'js>>> {
        let i = if let Some(base) = base.0 {
            match base {
                StringOrUrl::String(s) => {
                    let out = throw_if!(ctx, url::Url::parse(&s.to_string()?));
                    throw_if!(ctx, out.join(&url.as_str(&ctx)?))
                }
                StringOrUrl::Url(b) => return Ok(b),
            }
        } else {
            match url {
                StringOrUrl::String(s) => {
                    throw_if!(ctx, url::Url::parse(&s.to_string()?))
                }
                StringOrUrl::Url(url) => return Ok(url),
            }
        };

        Class::instance(ctx.clone(), Url::from_url(&ctx, &i)?)
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

create_export!(Url<'js>);
