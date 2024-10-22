use core::fmt;

use rquickjs::{class::Trace, Error, FromJs, IntoJs, Value};

#[derive(Trace, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::PATCH => "PATCH",
        }
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
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            _ => return Err(Error::new_from_js("string", "method")),
        };

        Ok(method)
    }
}
