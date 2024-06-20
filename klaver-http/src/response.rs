use futures::TryStreamExt;
use reggie::{Body, ResponseExt};
use klaver_base::streams::{async_byte_iterator, AsyncByteIterError};
use rquickjs::{class::Trace, Class, Ctx, Exception, Object, Value};

use crate::module::Headers;

#[rquickjs::class]
pub struct Response<'js> {
    #[qjs(get)]
    status: u16,
    #[qjs(get)]
    url: rquickjs::String<'js>,
    headers: Class<'js, Headers<'js>>,
    resp: Option<reggie::http::Response<Body>>,
}

impl<'js> Trace<'js> for Response<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.url.trace(tracer)
    }
}

impl<'js> Response<'js> {
    pub fn from_reqest(
        ctx: Ctx<'js>,
        url: &str,
        mut resp: reggie::http::Response<Body>,
    ) -> rquickjs::Result<Response<'js>> {
        let status = resp.status();
        let url = rquickjs::String::from_str(ctx.clone(), url)?;
        let headers = Headers::from_headers(&ctx, std::mem::take(resp.headers_mut()))?;

        Ok(Response {
            status: status.as_u16(),
            url,
            headers,
            resp: Some(resp),
        })
    }
}

#[rquickjs::methods]
impl<'js> Response<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        Ok(Response {
            status: 200,
            url: rquickjs::String::from_str(ctx.clone(), "")?,
            headers: Class::instance(ctx.clone(), Headers::default())?,
            resp: None,
        })
    }

    pub async fn text(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<String> {
        let Some(resp) = self.resp.take() else {
            return Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                "body is exhausted",
            )?)));
        };

        match resp.text().await {
            Ok(ret) => Ok(ret),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    pub async fn json(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let Some(resp) = self.resp.take() else {
            return Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                "body is exhausted",
            )?)));
        };

        match resp.json::<serde_json::Value>().await {
            Ok(ret) => Ok(crate::convert::from_json(ctx.clone(), ret)?),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    pub async fn stream(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let Some(resp) = self.resp.take() else {
            return Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                "body is exhausted",
            )?)));
        };

        let stream = resp.bytes_stream();

        let stream = stream
            .map_ok(|m| m.to_vec())
            .map_err(|_| AsyncByteIterError);

        async_byte_iterator(ctx, stream)
    }
}
