use klaver_core::create_export;
use klaver_core::throw_if;
use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, String as JsString, atom::PredefinedAtom, class::Trace,
    function::Opt,
};

use super::url_search_params::URLSearchParams;

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

/// Splits a `host[:port]` string, respecting bracketed IPv6 literals (`[::1]:8080`). The
/// trailing part is only treated as a port when it's non-empty and all-ASCII-digit.
fn split_host_port(input: &str) -> (&str, Option<&str>) {
    if input.starts_with('[') {
        if let Some(end) = input.find(']') {
            let host = &input[..=end];
            let rest = &input[end + 1..];
            let port = rest.strip_prefix(':').filter(|p| !p.is_empty());
            return (host, port);
        }
        return (input, None);
    }

    match input.rsplit_once(':') {
        Some((host, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => {
            (host, Some(port))
        }
        _ => (input, None),
    }
}

#[rquickjs::class(rename = "URL")]
pub struct Url<'js> {
    inner: url::Url,
    #[qjs(get, rename = "searchParams")]
    search_params: Class<'js, URLSearchParams<'js>>,
}

impl<'js> Trace<'js> for Url<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.search_params.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for Url<'js> {
    type Changed<'to> = Url<'to>;
}

impl<'js> Url<'js> {
    pub fn to_stdstring(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<String> {
        Ok(self.inner.as_str().to_string())
    }

    pub fn from_str(ctx: &Ctx<'js>, url: &str) -> rquickjs::Result<Url<'js>> {
        let url = throw_if!(ctx, url::Url::parse(url));
        Self::from_url(ctx, url)
    }

    pub fn from_url(ctx: &Ctx<'js>, url: url::Url) -> rquickjs::Result<Url<'js>> {
        let search_params = Class::instance(
            ctx.clone(),
            URLSearchParams::from_query(ctx.clone(), url.query())?,
        )?;

        Ok(Url {
            inner: url,
            search_params,
        })
    }

    /// Links a freshly constructed `Url` instance to its `searchParams`, so mutations made
    /// through the latter are reflected back (`https://url.spec.whatwg.org/#concept-urlsearchparams-update`).
    /// Must be called once, right after the `Url` is wrapped in a `Class`.
    fn link_search_params(this: &Class<'js, Url<'js>>) {
        let search_params = this.borrow().search_params.clone();
        search_params.borrow_mut().set_owner(this.clone());
    }

    /// Called by the linked `URLSearchParams` after it mutates, to push the new serialized
    /// query back into this URL. Does not re-sync `search_params` itself (that would be
    /// redundant, since it's the caller).
    pub fn set_query_from_search_params(&mut self, query: &str) {
        self.inner.set_query(if query.is_empty() {
            None
        } else {
            Some(query)
        });
    }

    fn resync_search_params(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let query = self.inner.query().map(|q| q.to_string());
        self.search_params
            .borrow_mut()
            .reset_from_query(ctx.clone(), query.as_deref())
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
        let parsed = match base.0 {
            Some(base) => {
                let base_url = match &base {
                    StringOrUrl::String(s) => throw_if!(ctx, url::Url::parse(&s.to_string()?)),
                    StringOrUrl::Url(u) => u.borrow().inner.clone(),
                };
                throw_if!(ctx, base_url.join(&url.as_str(&ctx)?))
            }
            None => match &url {
                StringOrUrl::String(s) => throw_if!(ctx, url::Url::parse(&s.to_string()?)),
                // `new URL(existingUrl)` must produce an independent copy, not alias it.
                StringOrUrl::Url(u) => u.borrow().inner.clone(),
            },
        };

        let instance = Class::instance(ctx.clone(), Url::from_url(&ctx, parsed)?)?;
        Url::link_search_params(&instance);
        Ok(instance)
    }

    #[qjs(static, rename = "canParse")]
    pub fn can_parse(
        ctx: Ctx<'js>,
        url: StringOrUrl<'js>,
        base: Opt<StringOrUrl<'js>>,
    ) -> rquickjs::Result<bool> {
        let url_str = url.as_str(&ctx)?;

        let ok = match base.0 {
            Some(base) => {
                let base_str = base.as_str(&ctx)?;
                url::Url::parse(&base_str)
                    .and_then(|b| b.join(&url_str))
                    .is_ok()
            }
            None => url::Url::parse(&url_str).is_ok(),
        };

        Ok(ok)
    }

    #[qjs(get, rename = "protocol")]
    pub fn get_protocol(&self) -> std::string::String {
        format!("{}:", self.inner.scheme())
    }

    #[qjs(set, rename = "protocol")]
    pub fn set_protocol(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        let scheme = s.trim_end_matches(':');
        // Per spec, an invalid scheme change is silently ignored rather than thrown.
        let _ = self.inner.set_scheme(scheme);
        Ok(())
    }

    #[qjs(get, rename = "username")]
    pub fn get_username(&self) -> std::string::String {
        self.inner.username().to_string()
    }

    #[qjs(set, rename = "username")]
    pub fn set_username(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        let _ = self.inner.set_username(&s);
        Ok(())
    }

    #[qjs(get, rename = "password")]
    pub fn get_password(&self) -> std::string::String {
        self.inner.password().unwrap_or("").to_string()
    }

    #[qjs(set, rename = "password")]
    pub fn set_password(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        let _ = self
            .inner
            .set_password(if s.is_empty() { None } else { Some(&s) });
        Ok(())
    }

    #[qjs(get, rename = "hostname")]
    pub fn get_hostname(&self) -> std::string::String {
        self.inner.host_str().unwrap_or("").to_string()
    }

    #[qjs(set, rename = "hostname")]
    pub fn set_hostname(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        let _ = self
            .inner
            .set_host(if s.is_empty() { None } else { Some(&s) });
        Ok(())
    }

    #[qjs(get, rename = "host")]
    pub fn get_host(&self) -> std::string::String {
        let Some(host) = self.inner.host_str() else {
            return std::string::String::new();
        };
        match self.inner.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_string(),
        }
    }

    #[qjs(set, rename = "host")]
    pub fn set_host(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        if s.is_empty() {
            let _ = self.inner.set_host(None);
            return Ok(());
        }

        let (host, port) = split_host_port(&s);
        if self.inner.set_host(Some(host)).is_ok() {
            if let Some(port) = port {
                if let Ok(port) = port.parse::<u16>() {
                    let _ = self.inner.set_port(Some(port));
                }
            }
        }

        Ok(())
    }

    #[qjs(get, rename = "port")]
    pub fn get_port(&self) -> std::string::String {
        self.inner
            .port()
            .map(|p| p.to_string())
            .unwrap_or_default()
    }

    #[qjs(set, rename = "port")]
    pub fn set_port(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        if s.is_empty() {
            let _ = self.inner.set_port(None);
        } else if let Ok(port) = s.parse::<u16>() {
            let _ = self.inner.set_port(Some(port));
        }
        // Non-numeric / out-of-range ports are silently ignored, matching the URL setter.
        Ok(())
    }

    #[qjs(get, rename = "pathname")]
    pub fn get_pathname(&self) -> std::string::String {
        self.inner.path().to_string()
    }

    #[qjs(set, rename = "pathname")]
    pub fn set_pathname(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        self.inner.set_path(&s);
        Ok(())
    }

    #[qjs(get, rename = "search")]
    pub fn get_search(&self) -> std::string::String {
        match self.inner.query() {
            Some(q) if !q.is_empty() => format!("?{q}"),
            _ => std::string::String::new(),
        }
    }

    #[qjs(set, rename = "search")]
    pub fn set_search(&mut self, ctx: Ctx<'js>, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        let trimmed = s.strip_prefix('?').unwrap_or(&s);
        self.inner
            .set_query(if trimmed.is_empty() { None } else { Some(trimmed) });
        self.resync_search_params(&ctx)
    }

    #[qjs(get, rename = "hash")]
    pub fn get_hash(&self) -> std::string::String {
        match self.inner.fragment() {
            Some(h) if !h.is_empty() => format!("#{h}"),
            _ => std::string::String::new(),
        }
    }

    #[qjs(set, rename = "hash")]
    pub fn set_hash(&mut self, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        let trimmed = s.strip_prefix('#').unwrap_or(&s);
        self.inner
            .set_fragment(if trimmed.is_empty() { None } else { Some(trimmed) });
        Ok(())
    }

    #[qjs(get, rename = "origin")]
    pub fn get_origin(&self) -> std::string::String {
        let Some(host) = self.inner.host_str() else {
            return "null".to_string();
        };

        match self.inner.port() {
            Some(port) => format!("{}://{host}:{port}", self.inner.scheme()),
            None => format!("{}://{host}", self.inner.scheme()),
        }
    }

    #[qjs(get, rename = "href")]
    pub fn get_href(&self) -> std::string::String {
        self.inner.as_str().to_string()
    }

    #[qjs(set, rename = "href")]
    pub fn set_href(&mut self, ctx: Ctx<'js>, value: JsString<'js>) -> rquickjs::Result<()> {
        let s = value.to_string()?;
        self.inner = throw_if!(ctx, url::Url::parse(&s));
        self.resync_search_params(&ctx)
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&self) -> std::string::String {
        self.get_href()
    }

    #[qjs(rename = PredefinedAtom::ToJSON)]
    pub fn to_json(&self) -> std::string::String {
        self.get_href()
    }
}

create_export!(Url<'js>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_port_splitting() {
        assert_eq!(split_host_port("example.com"), ("example.com", None));
        assert_eq!(
            split_host_port("example.com:8080"),
            ("example.com", Some("8080"))
        );
        assert_eq!(split_host_port("[::1]"), ("[::1]", None));
        assert_eq!(split_host_port("[::1]:8080"), ("[::1]", Some("8080")));
    }
}
