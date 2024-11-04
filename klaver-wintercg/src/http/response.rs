use reggie::Body;
use reqwest::Version;
use rquickjs::{
    class::Trace, function::Opt, ArrayBuffer, Class, Ctx, Exception, FromJs, Value,
};
use rquickjs_util::{throw, throw_if};

use crate::streams::ReadableStream;

use super::{
    body_init::BodyInit,
    body_state::ResponseBodyKind,
    headers::{Headers, HeadersInit},
};

// pub enum ResponseBodyKind<'js> {
//     Stream(Class<'js, ReadableStream<'js>>),
//     Body(Option<Body>),
//     Consumed,
// }

// impl<'js> ResponseBodyKind<'js> {
//     fn is_consumed(&self) -> bool {
//         match self {
//             Self::Consumed => true,
//             Self::Body(_) => false,
//             Self::Stream(stream) => stream.borrow().is_done(),
//         }
//     }
//     async fn bytes(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
//         match self {
//             Self::Body(body) => {
//                 let Some(body) = body else {
//                     throw!(ctx, "Body already consumed");
//                 };
//                 let bytes = throw_if!(ctx, reggie::body::to_bytes(body).await);
//                 *self = Self::Consumed;
//                 Ok(bytes.to_vec())
//             }
//             Self::Stream(stream) => {
//                 let bytes = stream.borrow_mut().to_bytes(ctx).await?;
//                 *self = Self::Consumed;
//                 Ok(bytes)
//             }
//             Self::Consumed => {
//                 throw!(ctx, "Body already consumed")
//             }
//         }
//     }

//     fn stream(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
//         match self {
//             Self::Body(body) => {
//                 let Some(body) = body.take() else {
//                     throw!(ctx, "Body already consumed")
//                 };
//                 let stream = ReadableStream::from_stream(
//                     ctx,
//                     Static(reggie::body::to_stream(body).map_ok(|m| Bytes(m.to_vec()))),
//                 )?;

//                 *self = Self::Stream(stream.clone());

//                 Ok(stream)
//             }
//             Self::Stream(stream) => Ok(stream.clone()),
//             Self::Consumed => {
//                 throw!(ctx, "Body already consumed")
//             }
//         }
//     }

//     async fn to_reggie(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Body> {
//         match self {
//             Self::Body(body) => {
//                 let Some(body) = body.take() else {
//                     throw!(ctx, "Body already consumed")
//                 };
//                 Ok(body)
//             }
//             Self::Stream(stream) => {
//                 let bytes = stream.borrow().to_bytes(ctx).await?;
//                 Ok(Body::from(bytes))
//             }
//             Self::Consumed => {
//                 throw!(ctx, "Body already consumed")
//             }
//         }
//     }
// }

// impl<'js> Trace<'js> for ResponseBodyKind<'js> {
//     fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
//         match self {
//             Self::Stream(stream) => stream.trace(tracer),
//             _ => {}
//         }
//     }
// }

#[rquickjs::class]
pub struct Response<'js> {
    #[qjs(get)]
    status: u16,
    #[qjs(get)]
    url: rquickjs::String<'js>,
    #[qjs(get)]
    headers: Class<'js, Headers<'js>>,
    body: Option<ResponseBodyKind<'js>>,
    version: reggie::http::Version,
}

impl<'js> Trace<'js> for Response<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.url.trace(tracer);
        self.headers.trace(tracer);
        self.body.trace(tracer);
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
            body: Some(ResponseBodyKind::Body(Some(body))),
            version: parts.version,
        })
    }

    pub async fn to_reggie(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<reggie::Response<Body>> {
        let mut builder = reggie::http::Response::builder()
            .status(self.status)
            // .url(throw_if!(ctx, Url::parse(&self.url.to_string()?)))
            .version(self.version);

        for (k, vals) in self.headers.borrow().inner.iter() {
            for v in vals {
                builder = builder.header(k, v.to_string()?);
            }
        }

        let body = if let Some(body) = self.body.as_mut() {
            body.to_reggie(ctx.clone()).await?
        } else {
            Body::empty()
        };

        let resp = throw_if!(ctx, builder.body(body));

        Ok(resp)
    }

    // fn take_body(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Body> {
    //     let Some(body) = self.body.take() else {
    //         throw!(ctx, "body is exhausted")
    //     };

    //     Ok(body)
    // }
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
        let body = if let Some(body) = body.0 {
            let body = ResponseBodyKind::Stream(body.to_stream(&ctx)?);
            Some(body)
        } else {
            None
        };

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
        let body = if let Some(body) = self.body.as_mut() {
            body.bytes(ctx.clone()).await?
        } else {
            throw!(ctx, "Response has no body")
        };

        match String::from_utf8(body) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    #[qjs(rename = "arrayBuffer")]
    pub async fn array_buffer(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        let body = if let Some(body) = self.body.as_mut() {
            body.bytes(ctx.clone()).await?
        } else {
            throw!(ctx, "Response has no body")
        };

        ArrayBuffer::new(ctx, body)
    }

    pub async fn json(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let body = if let Some(body) = self.body.as_mut() {
            body.bytes(ctx.clone()).await?
        } else {
            throw!(ctx, "Response has no body")
        };

        match serde_json::from_slice(&body) {
            Ok(ret) => Ok(super::convert::from_json(ctx.clone(), ret)?),
            Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                &err.to_string(),
            )?))),
        }
    }

    #[qjs(get, rename = "bodyUsed")]
    pub fn body_used(&self) -> rquickjs::Result<bool> {
        Ok(self.body.as_ref().map(|m| m.is_consumed()).unwrap_or(true))
    }

    #[qjs(get)]
    pub fn body(
        &mut self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Option<Class<'js, ReadableStream<'js>>>> {
        if let Some(body) = self.body.as_mut() {
            Ok(Some(body.stream(ctx)?))
        } else {
            Ok(None)
        }
    }

    #[qjs(get)]
    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status <= 299
    }

    // pub fn stream(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    //     let body = self.take_body(ctx.clone())?;

    //     let stream = reggie::body::to_stream(body);
    //     let stream = stream
    //         .map_ok(|m| m.to_vec())
    //         .map_err(|_| AsyncByteIterError);

    //     async_byte_iterator(ctx, stream)
    // }
}
