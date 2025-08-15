use std::{cell::RefCell, rc::Rc};

use futures::{TryStream, stream::LocalBoxStream};
use klaver_task::AsyncState;
use klaver_util::{
    AsyncIterableProtocol, Buffer, IteratorResult, NativeAsyncIteratorInterface, RuntimeError,
    StreamAsyncIterator, StringRef, throw,
};
use rquickjs::{
    CatchResultExt, Class, Ctx, FromJs, IntoJs, JsLifetime, Value,
    class::{JsClass, Trace},
    prelude::{Opt, This},
};

use crate::{
    Exportable,
    streams::{
        WritableStream,
        queue_strategy::QueuingStrategy,
        readable::{
            AsyncIteratorSource, NativeSource, from,
            reader::ReadableStreamDefaultReader,
            resource::ReadableStreamResource,
            source::{JsUnderlyingSource, UnderlyingSource},
            state::ReadableStreamData,
        },
    },
};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStream<'js> {
    state: Class<'js, ReadableStreamData<'js>>,
}

impl<'js> ReadableStream<'js> {
    pub fn from_stream<T>(
        ctx: &Ctx<'js>,
        stream: T,
        strategy: Option<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>>
    where
        T: TryStream + Trace<'js> + Unpin + 'js,
        T::Error: std::error::Error,
        T::Ok: IntoJs<'js>,
    {
        let stream = StreamAsyncIterator::new(stream);

        Self::from_native(ctx, AsyncIteratorSource(stream), strategy)
    }

    pub fn from_native<S: NativeSource<'js> + 'js>(
        ctx: &Ctx<'js>,
        source: S,
        strategy: Option<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        let strategy = match strategy {
            Some(ret) => ret,
            None => QueuingStrategy::create_default(ctx)?,
        };

        let data = ReadableStreamData::new(strategy);
        let state = Class::instance(ctx.clone(), data)?;

        let resource = ReadableStreamResource {
            data: state.clone(),
            source: UnderlyingSource::Native(Rc::new(RefCell::new(source))),
        };

        AsyncState::push(&ctx, resource)?;

        Ok(ReadableStream { state })
    }

    pub fn is(value: &Value<'js>) -> bool {
        Class::<Self>::from_value(value).is_ok()
    }

    pub fn disturbed(&self) -> bool {
        self.state.borrow().disturbed
            || self.state.borrow().is_cancled()
            || self.state.borrow().is_failed()
    }

    pub async fn to_bytes(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
        let reader = self.get_reader(ctx.clone())?;

        let mut output = Vec::default();

        loop {
            let next = reader.read_native(ctx).await?;

            if let Some(chunk) = next {
                if chunk.is_string() {
                    let chunk = StringRef::from_js(&ctx, chunk)?;
                    output.extend(chunk.as_bytes())
                } else {
                    let buffer = Buffer::from_js(&ctx, chunk)?;
                    if let Some(bytes) = buffer.as_raw() {
                        output.extend_from_slice(bytes.slice());
                    }
                }
            } else {
                break;
            }
        }

        Ok(output)
    }

    pub fn to_stream(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<LocalBoxStream<'js, rquickjs::Result<Value<'js>>>> {
        let reader = self.get_reader(ctx.clone())?;

        let stream = async_stream::try_stream! {
            loop {
                let next = reader.read_native(&ctx).await?;

                if  let Some(value) = next {
                    yield value
                } else {
                    break;
                }

            }
        };
        Ok(Box::pin(stream))
    }

    pub fn to_byte_stream(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<LocalBoxStream<'js, Result<Vec<u8>, RuntimeError>>> {
        let reader = self.get_reader(ctx.clone())?;

        let stream = async_stream::try_stream! {
            loop {
                let next = reader.read_native(&ctx).await.catch(&ctx)?;

                if  let Some(value) = next {
                    if value.is_string() {
                        let chunk = StringRef::from_js(&ctx, value).catch(&ctx)?;
                        yield chunk.as_bytes().to_vec()
                    } else  {
                        let buffer = Buffer::from_js(&ctx, value).catch(&ctx)?;
                        if let Some(bytes) = buffer.as_raw() {
                            yield bytes.slice().to_vec()
                        }
                    }
                } else {
                    break;
                }

            }
        };
        Ok(Box::pin(stream))
    }
}

#[rquickjs::methods]
impl<'js> ReadableStream<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        source: JsUnderlyingSource<'js>,
        strategy: Opt<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        let strategy = match strategy.0 {
            Some(ret) => ret,
            None => QueuingStrategy::create_default(&ctx)?,
        };

        let data = ReadableStreamData::new(strategy);
        let state = Class::instance(ctx.clone(), data)?;

        let resource = ReadableStreamResource {
            data: state.clone(),
            source: UnderlyingSource::Js(source),
        };

        AsyncState::push(&ctx, resource)?;

        Ok(ReadableStream { state })
    }

    #[qjs(rename = "getReader")]
    pub fn get_reader(&self, ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        if self.state.borrow().is_locked() {
            throw!(@type ctx, "Stream is locked")
        }

        self.state.borrow_mut().locked.set(true);

        Ok(ReadableStreamDefaultReader {
            data: Some(self.state.clone()),
        })
    }

    pub async fn cancel(
        This(this): This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        reason: Opt<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let reader = Class::instance(ctx.clone(), this.borrow().get_reader(ctx.clone())?)?;

        ReadableStreamDefaultReader::cancel(This(reader), ctx, reason).await?;

        Ok(())
    }

    #[qjs(get)]
    pub fn locked(&self) -> rquickjs::Result<bool> {
        Ok(self.state.borrow().is_locked())
    }

    #[qjs(rename = "pipeTo")]
    pub async fn pipe_to(
        &self,
        ctx: Ctx<'js>,
        stream: Class<'js, WritableStream<'js>>,
    ) -> rquickjs::Result<()> {
        let reader = self.get_reader(ctx.clone())?;
        let writer = stream.borrow().get_writer(ctx.clone())?;

        loop {
            let next = reader.read_native(&ctx).await?;

            let Some(next) = next else {
                writer.close(ctx.clone()).await?;
                break;
            };

            writer.ready().await?;

            writer.write(ctx.clone(), next)?;
        }

        Ok(())
    }

    #[qjs(static)]
    pub fn from(
        ctx: Ctx<'js>,
        value: Value<'js>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        from(&ctx, value)
    }
}

impl<'js> AsyncIterableProtocol<'js> for ReadableStream<'js> {
    type Iterator = ReadableStreamIterator<'js>;

    fn create_stream(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
        Ok(ReadableStreamIterator {
            readable: Class::instance(ctx.clone(), self.get_reader(ctx.clone())?)?,
        })
    }
}

pub struct ReadableStreamIterator<'js> {
    readable: Class<'js, ReadableStreamDefaultReader<'js>>,
}

impl<'js> Trace<'js> for ReadableStreamIterator<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.readable.trace(tracer);
    }
}

impl<'js> NativeAsyncIteratorInterface<'js> for ReadableStreamIterator<'js> {
    type Item = Value<'js>;

    async fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let ret =
            ReadableStreamDefaultReader::read(This(self.readable.clone()), ctx.clone()).await?;
        match ret {
            IteratorResult::Done => Ok(None),
            IteratorResult::Value(value) => Ok(Some(value)),
        }
    }

    async fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        self.readable.borrow_mut().release_lock();
        Ok(())
    }
}

impl<'js> Exportable<'js> for ReadableStream<'js> {
    fn export<T>(ctx: &Ctx<'js>, _registry: &crate::Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::ExportTarget<'js>,
    {
        target.set(
            ctx,
            ReadableStream::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        Self::add_iterable_prototype(ctx)?;

        Ok(())
    }
}
