use crate::{
    abort_controller::AbortSignal,
    streams::{
        WritableStream,
        queue_strategy::QueuingStrategy,
        readable::{
            AsyncIteratorSource, NativeSource,
            byob_reader::ReadableStreamBYOBReader,
            from,
            reader::ReadableStreamDefaultReader,
            resource::ReadableStreamResource,
            source::{JsUnderlyingSource, UnderlyingSource},
            state::ReadableStreamData,
            tee,
        },
        writable::WritableStreamDefaultWriter,
    },
};
use futures::{TryStream, stream::LocalBoxStream};
use klaver_core::{
    RuntimeError, throw,
    value::{
        Buffer, StringRef,
        async_iterator::{
            AsyncIterableProtocol, NativeAsyncIteratorInterface, StreamAsyncIterator,
        },
        iterable::IteratorResult,
    },
};
use klaver_runtime::AsyncState;
use rquickjs::{
    CatchResultExt, Class, Ctx, FromJs, IntoJs, JsLifetime, Object, Value,
    class::{JsClass, Trace},
    prelude::{Opt, This},
};
use std::{cell::RefCell, rc::Rc};

/// Options accepted by `pipeTo`/`pipeThrough`, per
/// <https://streams.spec.whatwg.org/#dictdef-streampipeoptions>.
#[derive(Default, Clone)]
pub struct PipeOptions<'js> {
    pub prevent_close: bool,
    pub prevent_abort: bool,
    pub prevent_cancel: bool,
    pub signal: Option<Class<'js, AbortSignal<'js>>>,
}

impl<'js> FromJs<'js> for PipeOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(PipeOptions {
            prevent_close: obj.get::<_, Option<bool>>("preventClose")?.unwrap_or(false),
            prevent_abort: obj.get::<_, Option<bool>>("preventAbort")?.unwrap_or(false),
            prevent_cancel: obj
                .get::<_, Option<bool>>("preventCancel")?
                .unwrap_or(false),
            signal: obj.get("signal")?,
        })
    }
}

/// The `{ readable, writable }` pair `pipeThrough` accepts - not necessarily a real
/// `TransformStream` instance, per spec any object shaped like one works.
pub struct TransformPair<'js> {
    pub readable: Class<'js, ReadableStream<'js>>,
    pub writable: Class<'js, WritableStream<'js>>,
}

impl<'js> FromJs<'js> for TransformPair<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(TransformPair {
            readable: obj.get("readable")?,
            writable: obj.get("writable")?,
        })
    }
}

/// Options accepted by `getReader`, per
/// <https://streams.spec.whatwg.org/#dictdef-readablestreamgetreaderoptions>.
#[derive(Default)]
pub struct GetReaderOptions {
    pub mode: Option<std::string::String>,
}

impl<'js> FromJs<'js> for GetReaderOptions {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;
        Ok(GetReaderOptions {
            mode: obj.get("mode")?,
        })
    }
}

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

    /// The `getReader()` no-options (or `{mode: undefined}`) path, used both as the plain-JS
    /// binding and internally (`cancel`, `pipeTo`, `tee`, async iteration) wherever a concrete
    /// default reader - not a `Value` that might be a BYOB reader - is needed.
    pub fn get_reader_native(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        if self.state.borrow().is_locked() {
            throw!(@type ctx, "Stream is locked")
        }

        self.state.borrow_mut().locked.set(true);

        Ok(ReadableStreamDefaultReader {
            data: Some(self.state.clone()),
        })
    }

    /// Access to the shared internal state, for code elsewhere in the crate (e.g.
    /// `TransformStream`) that needs to push into/observe this stream directly, bypassing the
    /// normal `NativeSource`/`JsUnderlyingSource` pull model.
    pub(crate) fn data(&self) -> Class<'js, ReadableStreamData<'js>> {
        self.state.clone()
    }

    pub fn disturbed(&self) -> bool {
        self.state.borrow().disturbed
            || self.state.borrow().is_cancled()
            || self.state.borrow().is_failed()
    }

    pub async fn to_bytes(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
        let reader = self.get_reader_native(ctx.clone())?;

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

    fn pipe_check_signal(ctx: &Ctx<'js>, options: &PipeOptions<'js>) -> Option<Value<'js>> {
        let signal = options.signal.as_ref()?;
        let signal = signal.borrow();
        if signal.aborted {
            Some(
                signal
                    .reason
                    .clone()
                    .unwrap_or_else(|| Value::new_undefined(ctx.clone())),
            )
        } else {
            None
        }
    }

    async fn pipe_shutdown_from_signal(
        ctx: &Ctx<'js>,
        reader: &ReadableStreamDefaultReader<'js>,
        writer: &WritableStreamDefaultWriter<'js>,
        reason: Value<'js>,
        options: &PipeOptions<'js>,
    ) -> Result<(), Value<'js>> {
        if !options.prevent_abort {
            writer.abort(ctx.clone(), Opt(Some(reason.clone()))).ok();
        }
        if !options.prevent_cancel {
            reader.cancel_native(ctx, Some(reason.clone())).await.ok();
        }
        Err(reason)
    }

    async fn pipe_to_native(
        ctx: &Ctx<'js>,
        reader: &ReadableStreamDefaultReader<'js>,
        writer: &WritableStreamDefaultWriter<'js>,
        options: &PipeOptions<'js>,
    ) -> Result<(), Value<'js>> {
        if let Some(reason) = Self::pipe_check_signal(ctx, options) {
            return Self::pipe_shutdown_from_signal(ctx, reader, writer, reason, options).await;
        }

        loop {
            if let Some(reason) = Self::pipe_check_signal(ctx, options) {
                return Self::pipe_shutdown_from_signal(ctx, reader, writer, reason, options).await;
            }

            let next = match reader.read_native(ctx).await.catch(ctx) {
                Ok(next) => next,
                Err(err) => {
                    // The source errored: abort the destination with the same reason (unless
                    // told not to), then propagate.
                    let reason = Self::caught_to_value(ctx, err);
                    if !options.prevent_abort {
                        writer.abort(ctx.clone(), Opt(Some(reason.clone()))).ok();
                    }
                    return Err(reason);
                }
            };

            let Some(next) = next else {
                if !options.prevent_close {
                    if let Err(err) = writer.close(ctx.clone()).await.catch(ctx) {
                        return Err(Self::caught_to_value(ctx, err));
                    }
                }
                return Ok(());
            };

            if let Err(err) = writer.ready().await.catch(ctx) {
                let reason = Self::caught_to_value(ctx, err);
                if !options.prevent_cancel {
                    reader.cancel_native(ctx, Some(reason.clone())).await.ok();
                }
                return Err(reason);
            }

            let promise = match writer.write(ctx.clone(), next) {
                Ok(promise) => promise,
                Err(err) => {
                    return Err(Self::caught_to_value(
                        ctx,
                        rquickjs::CaughtError::from_error(ctx, err),
                    ));
                }
            };
            if let Err(err) = promise.into_future::<()>().await.catch(ctx) {
                // The destination errored: cancel the source with the same reason (unless told
                // not to), then propagate.
                let reason = Self::caught_to_value(ctx, err);
                if !options.prevent_cancel {
                    reader.cancel_native(ctx, Some(reason.clone())).await.ok();
                }
                return Err(reason);
            }
        }
    }

    fn caught_to_value(ctx: &Ctx<'js>, err: rquickjs::CaughtError<'js>) -> Value<'js> {
        match err {
            rquickjs::CaughtError::Error(e) => {
                rquickjs::String::from_str(ctx.clone(), &e.to_string())
                    .map(|s| s.into_value())
                    .unwrap_or_else(|_| Value::new_undefined(ctx.clone()))
            }
            rquickjs::CaughtError::Exception(e) => e.into_object().into_value(),
            rquickjs::CaughtError::Value(v) => v,
        }
    }

    pub fn to_stream(
        &self,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<LocalBoxStream<'js, rquickjs::Result<Value<'js>>>> {
        let reader = self.get_reader_native(ctx.clone())?;

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
        let reader = self.get_reader_native(ctx.clone())?;

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

        let mut data = ReadableStreamData::new(strategy);
        data.is_byte_stream = source.r#type.as_deref() == Some("bytes");
        let state = Class::instance(ctx.clone(), data)?;

        let resource = ReadableStreamResource {
            data: state.clone(),
            source: UnderlyingSource::Js(source),
        };

        AsyncState::push(&ctx, resource)?;

        Ok(ReadableStream { state })
    }

    #[qjs(rename = "getReader")]
    pub fn get_reader(
        &self,
        ctx: Ctx<'js>,
        options: Opt<GetReaderOptions>,
    ) -> rquickjs::Result<Value<'js>> {
        let byob = options
            .0
            .map(|o| o.mode.as_deref() == Some("byob"))
            .unwrap_or(false);

        if byob {
            if !self.state.borrow().is_byte_stream {
                throw!(@type ctx, "Cannot get a BYOB reader for a stream that isn't a byte stream")
            }
            if self.state.borrow().is_locked() {
                throw!(@type ctx, "Stream is locked")
            }
            self.state.borrow_mut().locked.set(true);

            return Class::instance(
                ctx.clone(),
                ReadableStreamBYOBReader {
                    data: Some(self.state.clone()),
                },
            )?
            .into_js(&ctx);
        }

        Class::instance(ctx.clone(), self.get_reader_native(ctx.clone())?)?.into_js(&ctx)
    }

    pub async fn cancel(
        This(this): This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        reason: Opt<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let reader = Class::instance(ctx.clone(), this.borrow().get_reader_native(ctx.clone())?)?;

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
        options: Opt<PipeOptions<'js>>,
    ) -> rquickjs::Result<()> {
        let options = options.0.unwrap_or_default();
        let mut reader = self.get_reader_native(ctx.clone())?;
        let mut writer = stream.borrow().get_writer(ctx.clone())?;

        let result = Self::pipe_to_native(&ctx, &reader, &writer, &options).await;

        // Per spec, `pipeTo()` releases both locks once the pipe settles (either way), so the
        // source/destination are free to be used (e.g. `getReader()`/`getWriter()` again)
        // immediately after.
        reader.release_lock();
        writer.release_lock().ok();

        match result {
            Ok(()) => Ok(()),
            Err(reason) => Err(ctx.throw(reason)),
        }
    }

    /// Unlike `pipeTo`, `pipeThrough` returns the transform's readable side *synchronously*
    /// (per spec) - the actual piping runs in the background, with failures surfacing through
    /// the piped streams themselves (the destination gets aborted / the source gets cancelled,
    /// same as a `pipeTo()` failure) rather than through a promise here.
    #[qjs(rename = "pipeThrough")]
    pub fn pipe_through(
        &self,
        ctx: Ctx<'js>,
        pair: TransformPair<'js>,
        options: Opt<PipeOptions<'js>>,
    ) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
        let mut reader = self.get_reader_native(ctx.clone())?;
        let mut writer = pair.writable.borrow().get_writer(ctx.clone())?;
        let options = options.0.unwrap_or_default();
        let readable = pair.readable.clone();

        let bg_ctx = ctx.clone();
        ctx.spawn(async move {
            Self::pipe_to_native(&bg_ctx, &reader, &writer, &options)
                .await
                .ok();
            reader.release_lock();
            writer.release_lock().ok();
        });

        Ok(readable)
    }

    pub fn tee(&self, ctx: Ctx<'js>) -> rquickjs::Result<Vec<Class<'js, ReadableStream<'js>>>> {
        let reader = self.get_reader_native(ctx.clone())?;

        let branch2 = Class::instance(
            ctx.clone(),
            ReadableStream::from_native(&ctx, tee::Passive, None)?,
        )?;

        let branch1 = Class::instance(
            ctx.clone(),
            ReadableStream::from_native(
                &ctx,
                tee::Driving::new(reader, branch2.borrow().data()),
                None,
            )?,
        )?;

        Ok(vec![branch1, branch2])
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
            readable: Class::instance(ctx.clone(), self.get_reader_native(ctx.clone())?)?,
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
            IteratorResult::Done => {
                self.readable.borrow_mut().release_lock();
                Ok(None)
            }
            IteratorResult::Value(value) => Ok(Some(value)),
        }
    }

    async fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        self.readable.borrow_mut().release_lock();
        Ok(())
    }
}

impl<'js> klaver_core::Exportable<'js> for ReadableStream<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
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
