use std::cell::RefCell;

use futures::{Stream, StreamExt, TryStream, TryStreamExt, stream::LocalBoxStream};
use pin_project_lite::pin_project;
use rquickjs::{Ctx, IntoJs, class::Trace};

use crate::{async_iterator::native::NativeAsyncIteratorInterface, throw, throw_if};

pin_project! {
  pub struct AsyncIteratorStream<'js, T>
  where
    T: NativeAsyncIteratorInterface<'js>
   {
    #[pin]
    iterator: LocalBoxStream<'js, rquickjs::Result<T::Item>>
  }
}

impl<'js, T> AsyncIteratorStream<'js, T>
where
    T: NativeAsyncIteratorInterface<'js> + 'js,
{
    pub fn new(ctx: Ctx<'js>, iterator: T) -> AsyncIteratorStream<'js, T> {
        let stream = async_stream::stream! {

          loop {

            let next = iterator.next(&ctx).await;

            match next {
              Ok(Some(next)) => {
                yield Ok(next);
              }
              Ok(None) => {
                break;
              }
              Err(err) => {
                yield Err(err)
              }
            }
          }

          iterator.returns(&ctx).await?;

        };

        AsyncIteratorStream {
            iterator: stream.boxed_local(),
        }
    }
}

impl<'js, T> Stream for AsyncIteratorStream<'js, T>
where
    T: NativeAsyncIteratorInterface<'js>,
{
    type Item = rquickjs::Result<T::Item>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().iterator.poll_next(cx)
    }
}

pub struct StreamAsyncIterator<T> {
    stream: RefCell<T>,
}

impl<T> StreamAsyncIterator<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream: RefCell::new(stream),
        }
    }
}

impl<'js, T> Trace<'js> for StreamAsyncIterator<T> {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js, T> NativeAsyncIteratorInterface<'js> for StreamAsyncIterator<T>
where
    T: TryStream + Unpin,
    T::Ok: IntoJs<'js>,
    T::Error: std::error::Error,
{
    type Item = T::Ok;

    async fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let next = throw_if!(ctx, self.stream.borrow_mut().try_next().await);
        Ok(next)
    }

    async fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}
