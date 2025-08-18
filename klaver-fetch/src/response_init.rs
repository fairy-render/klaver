use http::StatusCode;
use klaver_util::throw_if;
use rquickjs::{Class, Ctx, FromJs, String};

use crate::{Headers, headers::HeadersInit};

pub struct ResponseInit<'js> {
    headers: Option<HeadersInit<'js>>,
    status: Option<u16>,
    status_text: Option<String<'js>>,
}

impl<'js> ResponseInit<'js> {
    pub fn build(
        self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<(Class<'js, Headers<'js>>, StatusCode, String<'js>)> {
        let headers = match self.headers {
            Some(ret) => ret.inner,
            None => Class::instance(ctx.clone(), Headers::new_native(ctx.clone())?)?,
        };

        let status = throw_if!(ctx, StatusCode::from_u16(self.status.unwrap_or(200)));

        let status_text = match self.status_text {
            Some(text) => text,
            None => String::from_str(ctx, "")?,
        };

        Ok((headers, status, status_text))
    }
}

impl<'js> FromJs<'js> for ResponseInit<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = value.into_object().ok_or_else(|| {
            rquickjs::Error::new_from_js("value", "object expected for ResponseInit")
        })?;

        Ok(ResponseInit {
            headers: obj.get("headers")?,
            status: obj.get("status")?,
            status_text: obj.get("statusText")?,
        })
    }
}
