use std::cell::RefCell;

use async_trait::async_trait;
use encoding_rs::{Decoder, Encoding};
use klaver_core::{throw, value::Buffer};
use rquickjs::{
    Class, Ctx, Exception, FromJs, JsLifetime, TypedArray, Value, class::Trace, function::Opt,
};

use crate::streams::{
    QueuingStrategy, ReadableStream, WritableStream,
    transform::{PassiveSource, TransformStreamDefaultController},
    writable::{NativeSink, WritableStreamDefaultController},
};

fn encoding_for_label<'js>(
    ctx: &Ctx<'js>,
    label: Option<String>,
) -> rquickjs::Result<&'static Encoding> {
    let Some(label) = label else {
        return Ok(encoding_rs::UTF_8);
    };

    let Some(encoding) = Encoding::for_label(label.as_bytes()) else {
        let err = ctx.throw(Value::from_exception(Exception::from_message(
            ctx.clone(),
            "unknown encoding",
        )?));
        return Err(err);
    };

    Ok(encoding)
}

/// `TextEncoderStream`, per <https://encoding.spec.whatwg.org/#interface-textencoderstream>.
///
/// Implemented directly on top of the same `PassiveSource`/`TransformStreamDefaultController`
/// machinery `TransformStream` uses, rather than as a `TransformStream` wrapping a JS
/// `transformer` - the string chunks QuickJS hands back are always already valid UTF-8 (see
/// `StringRef::as_str`), so there's no cross-chunk encoder state to maintain.
#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct TextEncoderStream<'js> {
    #[qjs(get)]
    readable: Class<'js, ReadableStream<'js>>,
    #[qjs(get)]
    writable: Class<'js, WritableStream<'js>>,
}

struct TextEncoderSink<'js> {
    controller: Class<'js, TransformStreamDefaultController<'js>>,
}

impl<'js> Trace<'js> for TextEncoderSink<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.controller.trace(tracer);
    }
}

impl<'js> TextEncoderSink<'js> {
    fn encode_and_enqueue(&self, ctx: &Ctx<'js>, chunk: Value<'js>) -> rquickjs::Result<()> {
        let text = klaver_core::value::StringRef::from_js(ctx, chunk)?;
        let bytes = TypedArray::<u8>::new(ctx.clone(), text.as_bytes())?;
        self.controller
            .borrow()
            .enqueue(ctx.clone(), bytes.into_value())
    }
}

#[async_trait(?Send)]
impl<'js> NativeSink<'js> for TextEncoderSink<'js> {
    async fn start(
        &self,
        _ctx: &Ctx<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    async fn write(
        &self,
        ctx: &Ctx<'js>,
        chunk: Value<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        let result = self.encode_and_enqueue(ctx, chunk);
        if result.is_err() {
            let failure = ctx.catch();
            self.controller.borrow().error(ctx.clone(), failure).ok();
        }
        result
    }

    async fn close(
        &self,
        ctx: &Ctx<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        self.controller.borrow().terminate(ctx.clone())
    }

    async fn abort(&self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        let reason = reason.unwrap_or_else(|| Value::new_undefined(ctx.clone()));
        self.controller.borrow().error(ctx.clone(), reason)
    }
}

#[rquickjs::methods]
impl<'js> TextEncoderStream<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<TextEncoderStream<'js>> {
        // Per spec, the readable side's high water mark defaults to 0, same as `TransformStream`.
        let readable_strategy = QueuingStrategy::create_with_high_water_mark(&ctx, 0)?;

        let readable = Class::instance(
            ctx.clone(),
            ReadableStream::from_native(&ctx, PassiveSource, Some(readable_strategy))?,
        )?;

        let controller = Class::instance(
            ctx.clone(),
            TransformStreamDefaultController {
                readable: readable.borrow().data(),
            },
        )?;

        let sink = TextEncoderSink { controller };

        let writable =
            Class::instance(ctx.clone(), WritableStream::from_native(&ctx, sink, None)?)?;

        Ok(TextEncoderStream { readable, writable })
    }

    #[qjs(get)]
    pub fn encoding(&self) -> &'static str {
        "utf-8"
    }
}

klaver_core::create_export!(TextEncoderStream<'js>);

/// `TextDecoderStream`, per <https://encoding.spec.whatwg.org/#interface-textdecoderstream>.
///
/// Unlike `TextEncoderStream`, byte chunks arriving here genuinely can split a multi-byte
/// sequence across `write()` calls (e.g. a network stream), so this keeps a real streaming
/// `encoding_rs::Decoder` alive across writes rather than decoding each chunk independently.
#[derive(JsLifetime)]
#[rquickjs::class]
pub struct TextDecoderStream<'js> {
    #[qjs(get)]
    readable: Class<'js, ReadableStream<'js>>,
    #[qjs(get)]
    writable: Class<'js, WritableStream<'js>>,
    encoding: &'static Encoding,
}

impl<'js> Trace<'js> for TextDecoderStream<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.readable.trace(tracer);
        self.writable.trace(tracer);
    }
}

struct TextDecoderSink<'js> {
    controller: Class<'js, TransformStreamDefaultController<'js>>,
    decoder: RefCell<Decoder>,
}

impl<'js> Trace<'js> for TextDecoderSink<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.controller.trace(tracer);
    }
}

impl<'js> TextDecoderSink<'js> {
    fn decode_and_enqueue(&self, ctx: &Ctx<'js>, bytes: &[u8], last: bool) -> rquickjs::Result<()> {
        let mut decoder = self.decoder.borrow_mut();
        // `decode_to_string` treats the `String`'s existing capacity as a hard output limit
        // rather than growing it - it never reallocates - so we must reserve enough space
        // up front or every chunk gets silently truncated to nothing.
        let capacity = decoder
            .max_utf8_buffer_length(bytes.len())
            .unwrap_or(bytes.len().saturating_mul(3) + 32);
        let mut out = String::with_capacity(capacity);
        let _ = decoder.decode_to_string(bytes, &mut out, last);

        if !out.is_empty() {
            let text = rquickjs::String::from_str(ctx.clone(), &out)?;
            self.controller
                .borrow()
                .enqueue(ctx.clone(), text.into_value())?;
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl<'js> NativeSink<'js> for TextDecoderSink<'js> {
    async fn start(
        &self,
        _ctx: &Ctx<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    async fn write(
        &self,
        ctx: &Ctx<'js>,
        chunk: Value<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        let result: rquickjs::Result<()> = (|| {
            let buffer = Buffer::from_js(ctx, chunk)?;
            let Some(raw) = buffer.as_raw() else {
                throw!(ctx, "buffer disconnected")
            };
            self.decode_and_enqueue(ctx, raw.slice(), false)
        })();

        if result.is_err() {
            let failure = ctx.catch();
            self.controller.borrow().error(ctx.clone(), failure).ok();
        }
        result
    }

    async fn close(
        &self,
        ctx: &Ctx<'js>,
        _ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        match self.decode_and_enqueue(ctx, &[], true) {
            Ok(()) => self.controller.borrow().terminate(ctx.clone()),
            Err(err) => {
                self.controller
                    .borrow()
                    .error(ctx.clone(), ctx.catch())
                    .ok();
                Err(err)
            }
        }
    }

    async fn abort(&self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        let reason = reason.unwrap_or_else(|| Value::new_undefined(ctx.clone()));
        self.controller.borrow().error(ctx.clone(), reason)
    }
}

#[rquickjs::methods]
impl<'js> TextDecoderStream<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, Opt(label): Opt<String>) -> rquickjs::Result<TextDecoderStream<'js>> {
        let encoding = encoding_for_label(&ctx, label)?;

        let readable_strategy = QueuingStrategy::create_with_high_water_mark(&ctx, 0)?;

        let readable = Class::instance(
            ctx.clone(),
            ReadableStream::from_native(&ctx, PassiveSource, Some(readable_strategy))?,
        )?;

        let controller = Class::instance(
            ctx.clone(),
            TransformStreamDefaultController {
                readable: readable.borrow().data(),
            },
        )?;

        let sink = TextDecoderSink {
            controller,
            decoder: RefCell::new(encoding.new_decoder()),
        };

        let writable =
            Class::instance(ctx.clone(), WritableStream::from_native(&ctx, sink, None)?)?;

        Ok(TextDecoderStream {
            readable,
            writable,
            encoding,
        })
    }

    #[qjs(get)]
    pub fn encoding(&self) -> String {
        self.encoding.name().to_ascii_lowercase()
    }
}

klaver_core::create_export!(TextDecoderStream<'js>);

// Tests live in `crate::streams::tests` alongside the rest of the stream-machinery test suite -
// see the comment there for why (shared async-Vm harness, and test-module ordering matters: see
// git history for a hang this avoided).
