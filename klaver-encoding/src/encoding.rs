use core::fmt;
use klaver::module_info;
use rquickjs::{
    class::Trace,
    function::{Func, Opt},
    module::{Declarations, Exports, ModuleDef},
    Class, Ctx, Exception, Result, Value,
};

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    Class::<TextDecoder>::register(ctx)?;
    Class::<TextDecoder>::define(&globals)?;

    Class::<TextEncoder>::register(ctx)?;
    Class::<TextEncoder>::define(&globals)?;

    Ok(())
}

pub struct Encoding;

module_info!("@klaver/encoding" => Encoding);

impl ModuleDef for Encoding {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("TextDecoder")?;
        decl.declare("TextEncoder")?;
        decl.declare("atob")?;
        decl.declare("btoa")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        Class::<TextDecoder>::register(ctx)?;
        Class::<TextEncoder>::register(ctx)?;

        exports.export("TextDecoder", Class::<TextDecoder>::create_constructor(ctx))?;
        exports.export("TextEncoder", Class::<TextEncoder>::create_constructor(ctx))?;

        exports.export("atob", Func::new(crate::b64::atob))?;
        exports.export("btoa", Func::new(crate::b64::btoa))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct UnknownEncoding;

impl fmt::Display for UnknownEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown encoding")
    }
}

impl std::error::Error for UnknownEncoding {}

#[derive(Debug)]
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
        self.decoder.output_encoding().name().to_string()
    }

    pub fn decode<'js>(
        &self,
        ctx: Ctx<'js>,
        input: rquickjs::ArrayBuffer<'js>,
    ) -> Result<rquickjs::String<'js>> {
        let Some(bytes) = input.as_bytes() else {
            return Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                "buffer disconnected",
            )?)));
        };

        let (ret, _, _) = self.decoder.decode(bytes);

        rquickjs::String::from_str(ctx, &*ret)
    }
}

#[derive(Debug)]
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
        self.decoder.output_encoding().name().to_string()
    }

    pub fn encode<'js>(
        &self,
        ctx: Ctx<'js>,
        input: String,
    ) -> Result<rquickjs::TypedArray<'js, u8>> {
        let (ret, _, _) = self.decoder.encode(&input);
        rquickjs::TypedArray::<u8>::new(ctx.clone(), &*ret)
    }
}
