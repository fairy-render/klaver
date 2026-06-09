use klaver_base::AbortSignal;
use rquickjs::{Class, Error, FromJs};

use crate::{Method, body_init::BodyInit, headers::HeadersInit};

pub struct RequestInit<'js> {
    pub method: Option<Method>,
    pub body: Option<BodyInit<'js>>,
    pub signal: Option<Class<'js, AbortSignal<'js>>>,
    pub headers: Option<HeadersInit<'js>>,
}

impl<'js> FromJs<'js> for RequestInit<'js> {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if value.is_null() || value.is_undefined() {
            return Ok(RequestInit {
                method: None,
                body: None,
                signal: None,
                headers: None,
            });
        }

        let Ok(obj) = value.try_into_object() else {
            return Err(Error::new_from_js("value", "object"));
        };

        let signal = obj.get("signal")?;
        let method = obj.get("method")?;
        let headers = obj.get("headers")?;

        Ok(RequestInit {
            signal,
            method,
            headers,
            body: obj.get("body")?,
        })
    }
}
