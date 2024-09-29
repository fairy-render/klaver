use futures::TryStreamExt;
use klaver::{throw, throw_if};
use klaver_streams::{async_byte_iterator, AsyncByteIterError};
use reggie::Body;
use reqwest::Version;
use rquickjs::{
    class::Trace, function::Opt, ArrayBuffer, Class, Ctx, Exception, FromJs, Object, Value,
};

use crate::{body::BodyInit, headers::HeadersInit, module::Headers};

#[rquickjs::class]
pub struct Response<'js> {
    #[qjs(get)]
    status: u16,
    #[qjs(get)]
    url: rquickjs::String<'js>,
    #[qjs(get)]
    headers: Class<'js, Headers<'js>>,
    body: Option<Body>,
    version: reggie::http::Version,
}

impl<'js> Trace<'js> for Response<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.url.trace(tracer);
        self.headers.trace(tracer);
    }
}

impl<'js> Response<'js> {
    pub fn from_response(
        ctx: Ctx<'js>,
        url: &str,
        resp: reggie::http::Response<Body>,
    ) -> rquickjs::Result<Response<'js>> {
        let (parts, body) = resp.into_parts();

        let status = parts.status;
        let url = rquickjs::String::from_str(ctx.clone(), url)?;
        let headers = Headers::from_headers(&ctx, parts.headers)?;

        Ok(Response {
            status: status.as_u16(),
            url,
            headers,
            body: body.into(),
            version: parts.version,
        })
    }

    pub fn to_reggie(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<reggie::Response<Body>> {
        let mut builder = reggie::http::Response::builder()
            .status(self.status)
            // .url(throw_if!(ctx, Url::parse(&self.url.to_string()?)))
            .version(self.version);

        for (k, vals) in self.headers.borrow().inner.iter() {
            for v in vals {
                builder = builder.header(k, v.to_string()?);
            }
        }

        let body = self
            .take_body(ctx.clone())
            .unwrap_or_else(|_| Body::empty());
        let resp = throw_if!(ctx, builder.body(body));

        Ok(resp)
    }

    fn take_body(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Body> {
        let Some(body) = self.body.take() else {
            throw!(ctx, "body is exhausted")
        };

        Ok(body)
    }
}

pub struct ResponseOptions<'js> {
    status: Option<u16>,
    headers: Option<HeadersInit<'js>>,
}

impl<'js> FromJs<'js> for ResponseOptions<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let Some(object) = value.as_object() else {
            return Err(rquickjs::Error::new_from_js(value.type_name(), "object"));
        };

        Ok(ResponseOptions {
            status: object.get("status")?,
            headers: object.get("headers")?,
        })
    }
}

#[rquickjs::methods]
impl<'js> Response<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        body: Opt<BodyInit<'js>>,
        options: Opt<ResponseOptions<'js>>,
    ) -> rquickjs::Result<Self> {
        let body = body.0.map(|m| Body::from(m.to_vec()));

        let (status, headers) = match options.0 {
            Some(ret) => (ret.status.unwrap_or(200), ret.headers),
            None => (200, None),
        };

        Ok(Response {
            status,
            url: rquickjs::String::from_str(ctx.clone(), "")?,
            headers: match headers {
                Some(header) => header.inner,
                None => Class::instance(ctx.clone(), Headers::default())?,
            },
            body,
            version: Version::HTTP_11,
        })
    }

    pub async fn text(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<String> {
        let body = self.take_body(ctx.clone())?;

        match reggie::body::to_text(body).await {
            Ok(ret) => Ok(ret),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    #[qjs(rename = "arrayBuffer")]
    pub async fn array_buffer(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        let body = self.take_body(ctx.clone())?;

        match reggie::body::to_bytes(body).await {
            Ok(ret) => ArrayBuffer::new(ctx, ret.to_vec()),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    pub async fn json(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let body = self.take_body(ctx.clone())?;

        match reggie::body::to_json::<serde_json::Value, _>(body).await {
            Ok(ret) => Ok(crate::convert::from_json(ctx.clone(), ret)?),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    pub fn stream(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
        let body = self.take_body(ctx.clone())?;

        let stream = reggie::body::to_stream(body);
        let stream = stream
            .map_ok(|m| m.to_vec())
            .map_err(|_| AsyncByteIterError);

        async_byte_iterator(ctx, stream)
    }
}
