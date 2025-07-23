use std::cell::RefCell;

use futures::{TryStreamExt, stream::LocalBoxStream};
use klaver_base::{
    Blob,
    streams::{ReadableStream, readable::One},
};
use reggie::{Body, http_body};
use rquickjs::{ArrayBuffer, Class, Ctx, String, TypedArray, Value, class::Trace};
use rquickjs_util::{Buffer, Bytes, RuntimeError, Static, throw, throw_if};

pub enum BodyState<'js> {
    Empty,
    HttpBody(Option<Body>),
    Bytes(ArrayBuffer<'js>),
    ReadableStream(Class<'js, ReadableStream<'js>>),
}

impl<'js> Trace<'js> for BodyState<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            BodyState::ReadableStream(stream) => stream.trace(tracer),
            BodyState::Bytes(bs) => {
                bs.trace(tracer);
            }
            _ => {}
        }
    }
}

pub struct BodyMixin<'js> {
    state: RefCell<BodyState<'js>>,
}

impl<'js> BodyMixin<'js> {
    pub fn empty() -> BodyMixin<'js> {
        BodyMixin {
            state: RefCell::new(BodyState::Empty),
        }
    }

    pub fn body_read(&self) -> bool {
        match &*self.state.borrow() {
            BodyState::Empty => true,
            BodyState::HttpBody(state) => state.is_none(),
            BodyState::Bytes(_) => false,
            BodyState::ReadableStream(stream) => stream.borrow().disturbed(),
        }
    }

    pub fn body(
        &self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<Option<Class<'js, ReadableStream<'js>>>> {
        match &mut *self.state.borrow_mut() {
            BodyState::Empty => Ok(None),
            BodyState::HttpBody(body) => {
                let Some(body) = body.take() else {
                    throw!(ctx, "Body is None")
                };

                let stream = ReadableStream::from_stream(
                    ctx.clone(),
                    Static(reggie::body::to_stream(body).map_ok(|m| Bytes(m.to_vec()))),
                )?;

                let stream = Class::instance(ctx.clone(), stream)?;

                self.state
                    .replace(BodyState::ReadableStream(stream.clone()));

                Ok(Some(stream))
            }
            BodyState::Bytes(bytes) => {
                let stream = ReadableStream::from_native(
                    ctx.clone(),
                    One::new(Buffer::ArrayBuffer(bytes.clone())),
                )?;

                let stream = Class::instance(ctx.clone(), stream)?;

                self.state
                    .replace(BodyState::ReadableStream(stream.clone()));

                Ok(Some(stream))
            }
            BodyState::ReadableStream(stream) => Ok(Some(stream.clone())),
        }
    }

    pub async fn to_bytes(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
        let Some(body) = self.body(ctx)? else {
            throw!(ctx, "Body already ready")
        };

        body.borrow().to_bytes(ctx.clone()).await
    }

    pub async fn to_text(&self, ctx: &Ctx<'js>) -> rquickjs::Result<String<'js>> {
        let bytes = self.to_bytes(ctx).await?;
        let string = throw_if!(ctx, std::string::String::from_utf8(bytes));
        String::from_str(ctx.clone(), &string)
    }

    pub async fn array_buffer(&self, ctx: &Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        let bytes = self.to_bytes(ctx).await?;
        ArrayBuffer::new(ctx.clone(), bytes)
    }

    pub async fn bytes(&self, ctx: &Ctx<'js>) -> rquickjs::Result<TypedArray<'js, u8>> {
        TypedArray::from_arraybuffer(self.array_buffer(ctx).await?)
    }

    pub async fn blob(
        &self,
        ctx: &Ctx<'js>,
        content_type: Option<String<'js>>,
    ) -> rquickjs::Result<Blob<'js>> {
        let array_buffer = self.array_buffer(ctx).await?;
        Ok(Blob {
            buffer: array_buffer,
            ty: content_type,
        })
    }
}

impl<'js> Trace<'js> for BodyMixin<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.state.borrow().trace(tracer);
    }
}

impl<'js> From<Class<'js, ReadableStream<'js>>> for BodyMixin<'js> {
    fn from(value: Class<'js, ReadableStream<'js>>) -> Self {
        BodyMixin {
            state: RefCell::new(BodyState::ReadableStream(value)),
        }
    }
}

impl<'js> From<ArrayBuffer<'js>> for BodyMixin<'js> {
    fn from(value: ArrayBuffer<'js>) -> Self {
        BodyMixin {
            state: RefCell::new(BodyState::Bytes(value)),
        }
    }
}

impl<'js> From<Body> for BodyMixin<'js> {
    fn from(value: Body) -> Self {
        BodyMixin {
            state: RefCell::new(BodyState::HttpBody(Some(value))),
        }
    }
}

pub struct JsBody<'js> {
    inner: LocalBoxStream<'js, rquickjs::Result<Vec<u8>>>,
}

impl<'js> http_body::Body for JsBody<'js> {
    type Data = bytes::Bytes;

    type Error = RuntimeError;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        todo!()
    }
}
