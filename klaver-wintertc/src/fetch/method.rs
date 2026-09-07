use core::fmt;

use rquickjs::{Error, FromJs, IntoJs, Value, class::Trace};

#[derive(Clone)]
pub struct Method(pub http::Method);

impl<'js> Trace<'js> for Method {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl Method {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'js> IntoJs<'js> for Method {
    fn into_js(self, ctx: &rquickjs::prelude::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let str = self.as_str();

        Ok(Value::from_string(rquickjs::String::from_str(
            ctx.clone(),
            str,
        )?))
    }
}

impl<'js> FromJs<'js> for Method {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let Some(method) = value.as_string() else {
            return Err(Error::new_from_js("value", "string"));
        };

        let method_str = method.to_string()?;

        // Per <https://fetch.spec.whatwg.org/#concept-method>, the method must match the HTTP
        // `token` production; `http::Method::from_bytes` enforces exactly that grammar (and,
        // unlike matching a fixed list, still accepts custom/extension methods like `PROPFIND`).
        let parsed = http::Method::from_bytes(method_str.as_bytes())
            .map_err(|_| Error::new_from_js("string", "HTTP method"))?;

        let upper = parsed.as_str().to_ascii_uppercase();

        // Forbidden methods, per <https://fetch.spec.whatwg.org/#forbidden-method> - these are
        // never valid on a `Request`/in `fetch()`, regardless of case.
        if matches!(upper.as_str(), "CONNECT" | "TRACE" | "TRACK") {
            return Err(Error::new_from_js("string", "forbidden HTTP method"));
        }

        // "To normalize a method": byte-uppercase it if it case-insensitively matches one of
        // these six well-known methods; any other (still-valid) token is kept exactly as given.
        let normalized = match upper.as_str() {
            "DELETE" => http::Method::DELETE,
            "GET" => http::Method::GET,
            "HEAD" => http::Method::HEAD,
            "OPTIONS" => http::Method::OPTIONS,
            "POST" => http::Method::POST,
            "PUT" => http::Method::PUT,
            _ => parsed,
        };

        Ok(Method(normalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{Context, Runtime};

    fn parse(method: &str) -> Result<std::string::String, ()> {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let value = method.into_js(&ctx).unwrap();
            Method::from_js(&ctx, value)
                .map(|m| m.as_str().to_string())
                .map_err(|_| ())
        })
    }

    #[test]
    fn normalizes_well_known_methods_case_insensitively() {
        assert_eq!(parse("get").unwrap(), "GET");
        assert_eq!(parse("Get").unwrap(), "GET");
        assert_eq!(parse("post").unwrap(), "POST");
        assert_eq!(parse("Delete").unwrap(), "DELETE");
    }

    #[test]
    fn preserves_case_of_other_valid_tokens() {
        assert_eq!(parse("PROPFIND").unwrap(), "PROPFIND");
        assert_eq!(parse("PATCH").unwrap(), "PATCH");
        assert_eq!(parse("MySuperMethod").unwrap(), "MySuperMethod");
    }

    #[test]
    fn rejects_forbidden_methods_regardless_of_case() {
        assert!(parse("CONNECT").is_err());
        assert!(parse("connect").is_err());
        assert!(parse("TRACE").is_err());
        assert!(parse("Track").is_err());
    }

    #[test]
    fn rejects_invalid_tokens() {
        assert!(parse("GET /foo").is_err());
        assert!(parse("").is_err());
        assert!(parse("get\r\nX-Injected: 1").is_err());
    }
}
