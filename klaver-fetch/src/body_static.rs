use bytes::Bytes;
use core::{fmt, pin::Pin, task::Poll};
use futures::ready;
use http_body_util::combinators::BoxBody;
use klaver_util::BoxError;

#[derive(Debug)]
pub struct BodyError {
    inner: BoxError,
}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for BodyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.inner)
    }
}

enum Inner {
    Reusable(Bytes),
    Streaming(BoxBody<Bytes, BoxError>),
}

pub struct Body {
    inner: Inner,
}

impl Body {
    pub fn empty() -> Body {
        Body {
            inner: Inner::Reusable(Bytes::new()),
        }
    }

    pub fn from_streaming<B: http_body::Body>(inner: B) -> Body
    where
        B: Send + Sync + 'static,
        B::Error: Into<BoxError>,
        B::Data: Into<Bytes>,
    {
        use http_body_util::BodyExt;

        let boxed = inner
            .map_frame(|f| f.map_data(Into::into))
            .map_err(Into::into)
            .boxed();

        Body {
            inner: Inner::Streaming(boxed),
        }
    }
}

impl http_body::Body for Body {
    type Data = bytes::Bytes;

    type Error = BodyError;

    fn poll_frame(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.inner {
            Inner::Reusable(ref mut bytes) => {
                let out = bytes.split_off(0);
                if out.is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(http_body::Frame::data(out))))
                }
            }
            Inner::Streaming(ref mut body) => Poll::Ready(
                ready!(Pin::new(body).poll_frame(cx))
                    .map(|m| m.map_err(|err| BodyError { inner: err })),
            ),
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        match self.inner {
            Inner::Reusable(ref bytes) => http_body::SizeHint::with_exact(bytes.len() as u64),
            Inner::Streaming(ref body) => body.size_hint(),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self.inner {
            Inner::Reusable(ref bytes) => bytes.is_empty(),
            Inner::Streaming(ref body) => body.is_end_stream(),
        }
    }
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        value.as_bytes().to_vec().into()
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body {
            inner: Inner::Reusable(value.into()),
        }
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        Body {
            inner: Inner::Reusable(value),
        }
    }
}

pub async fn to_text<T: http_body::Body>(body: T) -> Result<String, BoxError>
where
    T::Error: Into<BoxError>,
{
    use http_body_util::BodyExt;

    let bytes = BodyExt::collect(body)
        .await
        .map(|buf| buf.to_bytes())
        .map_err(Into::into)?;

    String::from_utf8(bytes.to_vec()).map_err(|err| Box::new(err) as BoxError)
}

pub async fn to_bytes<T: http_body::Body>(body: T) -> Result<Bytes, BoxError>
where
    T::Error: Into<BoxError>,
{
    use http_body_util::BodyExt;

    BodyExt::collect(body)
        .await
        .map(|buf| buf.to_bytes())
        .map_err(Into::into)
}

#[cfg(feature = "json")]
pub async fn to_json<T: serde::de::DeserializeOwned, B: http_body::Body>(
    body: B,
) -> Result<T, Error>
where
    B::Error: Into<Error>,
{
    use http_body_util::BodyExt;

    let bytes = BodyExt::collect(body)
        .await
        .map(|buf| buf.to_bytes())
        .map_err(Into::into)?;

    serde_json::from_slice::<T>(&bytes).map_err(|err| Error::Body(Box::new(err)))
}

// pub fn to_stream<T: http_body::Body>(body: T) -> DataStream<T> {
//     DataStream(body)
// }
