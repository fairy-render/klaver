use futures::{Stream, StreamExt, TryStreamExt, future::LocalBoxFuture, stream::LocalBoxStream};
use klaver_base::{
    Blob,
    streams::{ReadableStream, readable::One},
};
use klaver_util::{Buffer, Bytes, RuntimeError, Static, throw, throw_if};
use pin_project_lite::pin_project;
use reggie::{
    Body,
    http_body::{self, Frame},
};
use rquickjs::{ArrayBuffer, Class, Ctx, String, TypedArray, Value, class::Trace};
use std::{
    cell::RefCell,
    task::{Poll, ready},
};

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
        let mut state = self.state.borrow_mut();

        match &mut *state {
            BodyState::Empty => Ok(None),
            BodyState::HttpBody(body) => {
                let Some(body) = body.take() else {
                    throw!(ctx, "Body is None")
                };

                let stream = ReadableStream::from_stream(
                    ctx,
                    Static(reggie::body::to_stream(body).map_ok(|m| Bytes(m.to_vec()))),
                    None,
                )?;

                let stream = Class::instance(ctx.clone(), stream)?;

                drop(state);

                self.state
                    .replace(BodyState::ReadableStream(stream.clone()));

                Ok(Some(stream))
            }
            BodyState::Bytes(bytes) => {
                let stream = ReadableStream::from_native(
                    ctx,
                    One::new(Buffer::ArrayBuffer(bytes.clone())),
                    None,
                )?;

                let stream = Class::instance(ctx.clone(), stream)?;

                drop(state);

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
        body.borrow().to_bytes(ctx).await
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

    pub async fn json(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        ctx.json_parse(self.to_bytes(&ctx).await?)
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

    // TODO: Take fast path, if body isnt a ReadableStream
    pub fn to_native_body(&self, ctx: &Ctx<'js>) -> rquickjs::Result<JsBody<'js>> {
        match self.body(ctx)? {
            Some(inner) => Ok(JsBody {
                inner: JsBodyState::Stream {
                    stream: inner.borrow().to_byte_stream(ctx.clone())?,
                },
            }),
            None => Ok(JsBody {
                inner: JsBodyState::Empty,
            }),
        }
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
pin_project! {
    #[project = BodyProj]
    enum JsBodyState<'js> {
        Empty,
        Stream {
            #[pin]
            stream: LocalBoxStream<'js, Result<Vec<u8>, RuntimeError>>,
        },
    }
}

impl<'js> Stream for JsBodyState<'js> {
    type Item = Result<Vec<u8>, RuntimeError>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.project() {
            BodyProj::Empty => Poll::Ready(None),
            BodyProj::Stream { stream } => stream.poll_next(cx),
        }
    }
}

pin_project! {
    pub struct JsBody<'js> {
        #[pin]
        inner: JsBodyState<'js>,
    }
}

impl<'js> JsBody<'js> {
    pub fn into_remote(self) -> (RemoteBody, RemoteBodyProducer<'js>) {
        let (sx, rx) = flume::bounded(1);

        let mut stream = self.inner;
        let producer = RemoteBodyProducer {
            inner: Box::pin(async move {
                //

                while let Some(next) = stream.next().await {
                    sx.send_async(next.map(bytes::Bytes::from).map(Frame::data))
                        .await
                        .map_err(|err| RuntimeError::Custom(Box::new(err)));
                }

                Ok(())
            }),
        };

        let body = RemoteBody {
            rx: rx.into_stream(),
        };

        (body, producer)
    }
}

impl<'js> http_body::Body for JsBody<'js> {
    type Data = bytes::Bytes;

    type Error = RuntimeError;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match ready!(self.project().inner.poll_next(cx)) {
            Some(Ok(ret)) => Poll::Ready(Some(Ok(Frame::data(ret.into())))),
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}

pin_project! {
    pub struct RemoteBody {
        #[pin]
        rx: flume::r#async::RecvStream<'static, Result<Frame<bytes::Bytes>, RuntimeError>>,
    }
}

impl http_body::Body for RemoteBody {
    type Data = bytes::Bytes;

    type Error = RuntimeError;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        self.project().rx.poll_next(cx)
    }
}

pin_project! {
    pub struct RemoteBodyProducer<'js> {
        #[pin]
        inner: LocalBoxFuture<'js, rquickjs::Result<()>>,
    }
}

impl<'js> Future for RemoteBodyProducer<'js> {
    type Output = rquickjs::Result<()>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}
