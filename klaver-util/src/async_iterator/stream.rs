use futures::{Stream, StreamExt, stream::LocalBoxStream};
use pin_project_lite::pin_project;
use rquickjs::Ctx;

use crate::async_iterator::native::NativeAsyncIteratorInterface;

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
