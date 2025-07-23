use core::fmt;

use rquickjs::{Error, FromJs, IntoJs, Value, class::Trace};

#[derive(Clone)]
pub struct Method(http::Method);

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

        let method = match &*method.to_string()? {
            "GET" => http::Method::GET,
            "POST" => http::Method::POST,
            "PUT" => http::Method::PUT,
            "PATCH" => http::Method::PATCH,
            "DELETE" => http::Method::DELETE,
            "HEAD" => http::Method::HEAD,
            "OPTIONS" => http::Method::OPTIONS,
            "TRACE" => http::Method::TRACE,
            "CONNECT" => http::Method::CONNECT,
            _ => return Err(Error::new_from_js("string", "method")),
        };

        Ok(Method(method))
    }
}
