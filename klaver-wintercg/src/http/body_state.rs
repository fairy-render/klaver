use futures::TryStreamExt;
use reggie::Body;
use rquickjs::{
    class::Trace, Class, Ctx,
};
use rquickjs_util::{throw, throw_if};
use rquickjs_util::{Bytes, Static};

use crate::streams::ReadableStream;


pub enum ResponseBodyKind<'js> {
    Stream(Class<'js, ReadableStream<'js>>),
    Body(Option<Body>),
    Consumed,
}

impl<'js> ResponseBodyKind<'js> {
    pub fn is_consumed(&self) -> bool {
        match self {
            Self::Consumed => true,
            Self::Body(_) => false,
            Self::Stream(stream) => stream.borrow().is_done(),
        }
    }
    pub async fn bytes(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
        match self {
            Self::Body(body) => {
                let Some(body) = body else {
                    throw!(ctx, "Body already consumed");
                };
                let bytes = throw_if!(ctx, reggie::body::to_bytes(body).await);
                *self = Self::Consumed;
                Ok(bytes.to_vec())
            }
            Self::Stream(stream) => {
                let bytes = stream.borrow_mut().to_bytes(ctx).await?;
                *self = Self::Consumed;
                Ok(bytes)
            }
            Self::Consumed => {
                throw!(ctx, "Body already consumed")
            }
        }
    }

    pub fn stream(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        match self {
            Self::Body(body) => {
                let Some(body) = body.take() else {
                    throw!(ctx, "Body already consumed")
                };
                let stream = ReadableStream::from_stream(
                    ctx,
                    Static(reggie::body::to_stream(body).map_ok(|m| Bytes(m.to_vec()))),
                )?;

                *self = Self::Stream(stream.clone());

                Ok(stream)
            }
            Self::Stream(stream) => Ok(stream.clone()),
            Self::Consumed => {
                throw!(ctx, "Body already consumed")
            }
        }
    }

    pub async fn to_reggie(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Body> {
        match self {
            Self::Body(body) => {
                let Some(body) = body.take() else {
                    throw!(ctx, "Body already consumed")
                };
                Ok(body)
            }
            Self::Stream(stream) => {
                let bytes = stream.borrow().to_bytes(ctx).await?;
                Ok(Body::from(bytes))
            }
            Self::Consumed => {
                throw!(ctx, "Body already consumed")
            }
        }
    }
}

impl<'js> Trace<'js> for ResponseBodyKind<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Stream(stream) => stream.trace(tracer),
            _ => {}
        }
    }
}
