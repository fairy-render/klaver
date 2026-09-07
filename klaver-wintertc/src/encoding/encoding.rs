use klaver_core::value::{Buffer, StringRef};
use rquickjs::{Ctx, Exception, Result, Value, class::Trace, function::Opt};

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct TextDecoder {
    decoder: &'static encoding_rs::Encoding,
}

impl<'js> Trace<'js> for TextDecoder {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl TextDecoder {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>, Opt(label): Opt<String>) -> Result<TextDecoder> {
        if let Some(label) = label {
            let Some(encoding) = encoding_rs::Encoding::for_label(label.as_bytes()) else {
                let err = ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    "unknown encoding",
                )?));
                return Err(err);
            };

            Ok(TextDecoder { decoder: encoding })
        } else {
            Ok(TextDecoder {
                decoder: encoding_rs::UTF_8,
            })
        }
    }

    #[qjs(get)]
    pub fn encoding(&self) -> String {
        self.decoder.name().to_ascii_lowercase()
    }

    pub fn decode<'js>(&self, ctx: Ctx<'js>, input: Buffer<'js>) -> Result<rquickjs::String<'js>> {
        let Some(bytes) = input.as_raw() else {
            return Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                "buffer disconnected",
            )?)));
        };

        let (ret, _, _) = self.decoder.decode(bytes.slice());

        rquickjs::String::from_str(ctx, &*ret)
    }
}

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct TextEncoder {
    decoder: &'static encoding_rs::Encoding,
}

impl<'js> Trace<'js> for TextEncoder {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl TextEncoder {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>, Opt(label): Opt<String>) -> Result<TextEncoder> {
        if let Some(label) = label {
            let Some(encoding) = encoding_rs::Encoding::for_label(label.as_bytes()) else {
                let err = ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    "unknown encoding",
                )?));
                return Err(err);
            };

            Ok(TextEncoder { decoder: encoding })
        } else {
            Ok(TextEncoder {
                decoder: encoding_rs::UTF_8,
            })
        }
    }

    #[qjs(get)]
    pub fn encoding(&self) -> String {
        self.decoder.output_encoding().name().to_ascii_lowercase()
    }

    pub fn encode<'js>(
        &self,
        ctx: Ctx<'js>,
        input: StringRef<'js>,
    ) -> Result<rquickjs::TypedArray<'js, u8>> {
        let (ret, _, _) = self.decoder.encode(input.as_str());
        rquickjs::TypedArray::<u8>::new(ctx.clone(), &*ret)
    }
}

klaver_core::create_export!(TextDecoder);
klaver_core::create_export!(TextEncoder);

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    /// Runs `body` as the contents of a plain function, with global `TextEncoder` and
    /// `TextDecoder` constructors available. `body` is expected to throw on failure (e.g. via a
    /// plain `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                ctx.globals().set(
                    "TextEncoder",
                    rquickjs::Class::<TextEncoder>::create_constructor(&ctx)?,
                )?;
                ctx.globals().set(
                    "TextDecoder",
                    rquickjs::Class::<TextDecoder>::create_constructor(&ctx)?,
                )?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn text_encoder_defaults_to_utf8() {
        run(r#"
            const encoder = new TextEncoder();
            if (encoder.encoding !== "utf-8") throw new Error(`encoding was ${encoder.encoding}`);
        "#);
    }

    #[test]
    fn text_encoder_encodes_multibyte_utf8() {
        run(r#"
            const bytes = new TextEncoder().encode("é");
            // U+00E9 is encoded as the two UTF-8 bytes 0xC3 0xA9.
            if (bytes.length !== 2) throw new Error(`length was ${bytes.length}`);
            if (bytes[0] !== 0xc3 || bytes[1] !== 0xa9) throw new Error(`bytes were ${bytes}`);
        "#);
    }

    #[test]
    fn text_decoder_defaults_to_utf8() {
        run(r#"
            const decoder = new TextDecoder();
            if (decoder.encoding !== "utf-8") throw new Error(`encoding was ${decoder.encoding}`);
        "#);
    }

    #[test]
    fn round_trips_through_encoder_and_decoder() {
        run(r#"
            const bytes = new TextEncoder().encode("hello world");
            const text = new TextDecoder().decode(bytes);
            if (text !== "hello world") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn decoder_accepts_an_array_buffer_too() {
        run(r#"
            const buf = new TextEncoder().encode("hi").buffer;
            const text = new TextDecoder().decode(buf);
            if (text !== "hi") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn decoder_label_is_case_insensitive_and_normalizes_aliases() {
        run(r#"
            // "utf8" (no hyphen) is a registered alias for "utf-8" per the Encoding Standard.
            if (new TextDecoder("UTF8").encoding !== "utf-8") {
                throw new Error(`encoding was ${new TextDecoder("UTF8").encoding}`);
            }
        "#);
    }

    #[test]
    fn decoder_rejects_unknown_label() {
        run(r#"
            let threw = false;
            try {
                new TextDecoder("not-a-real-encoding");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected constructor to throw");
        "#);
    }

    #[test]
    fn encoder_rejects_unknown_label() {
        run(r#"
            let threw = false;
            try {
                new TextEncoder("not-a-real-encoding");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected constructor to throw");
        "#);
    }
}
