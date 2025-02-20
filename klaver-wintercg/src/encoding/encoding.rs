use core::fmt;
use rquickjs::{class::Trace, function::Opt, Class, Ctx, Exception, Result, Value};
use rquickjs_util::{Buffer, StringRef};

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    Class::<TextDecoder>::define(&globals)?;
    Class::<TextEncoder>::define(&globals)?;

    Ok(())
}

// pub struct Encoding;

// module_info!("@klaver/encoding" @types: include_str!("../module.d.ts") => Encoding);

// impl ModuleDef for Encoding {
//     fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
//         decl.declare("TextDecoder")?;
//         decl.declare("TextEncoder")?;
//         decl.declare("atob")?;
//         decl.declare("btoa")?;
//         Ok(())
//     }

//     fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
//         Class::<TextDecoder>::register(ctx)?;
//         Class::<TextEncoder>::register(ctx)?;

//         exports.export("TextDecoder", Class::<TextDecoder>::create_constructor(ctx))?;
//         exports.export("TextEncoder", Class::<TextEncoder>::create_constructor(ctx))?;

//         exports.export("atob", Func::new(crate::b64::atob))?;
//         exports.export("btoa", Func::new(crate::b64::btoa))?;

//         Ok(())
//     }
// }

#[derive(Debug)]
pub struct UnknownEncoding;

impl fmt::Display for UnknownEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown encoding")
    }
}

impl std::error::Error for UnknownEncoding {}

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
        self.decoder.output_encoding().name().to_string()
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
        self.decoder.output_encoding().name().to_string()
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
