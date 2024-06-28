use core::fmt;

use rquickjs::{class::Trace, function::Opt, Ctx, Exception, Result, Value};
use slotmap::DefaultKey;

pub const BASE_KEY: &str = "$_klaver";

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

#[rquickjs::class]
pub struct TimerId {
    key: DefaultKey,
}

impl<'js> Trace<'js> for TimerId {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::module]
pub mod base_mod {
    use rquickjs::{function::Opt, Ctx, Function, Result};
    use rquickjs::{Class, Error};
    use tokio::sync::oneshot;

    use crate::base::Base;
    use crate::get_base;

    use super::TimerId;
    pub use super::{TextDecoder, TextEncoder};

    #[rquickjs::function]
    pub fn set_timeout<'js>(
        ctx: Ctx<'js>,
        func: Function<'js>,
        Opt(duration): Opt<u64>,
    ) -> Result<TimerId> {
        let base_class: Class<'js, Base<'js>> = get_base(&ctx)?;

        let mut base = base_class.try_borrow_mut()?;
        let (sx, rx) = oneshot::channel();
        let id = base.timeouts.insert(sx);
        drop(base);

        let cloned_ctx = ctx.clone();

        ctx.spawn(async move {
            let sleep = tokio::time::sleep(std::time::Duration::from_millis(duration.unwrap_or(0)));
            tokio::select! {
              _ = rx => {
                return
              }
              _ = sleep => {
                let mut base = base_class.try_borrow_mut().unwrap();
                base.timeouts.remove(id);
                drop(base);
                if let Err(Error::Exception) = func.call::<_, ()>(()) {
                  let mut base = base_class.try_borrow_mut().unwrap();
                  base.uncaugth_exceptions.push_back(cloned_ctx.catch());

                }

              }
            }
        });
        Ok(TimerId { key: id })
    }

    #[rquickjs::function]
    pub fn clear_timeout(ctx: Ctx<'_>, key: Class<'_, TimerId>) -> Result<()> {
        let base_class: Class<'_, Base<'_>> = get_base(&ctx)?;
        let mut base = base_class.try_borrow_mut()?;
        if let Some(sx) = base.timeouts.remove(key.try_borrow()?.key) {
            sx.send(()).ok();
        }

        Ok(())
    }

    #[rquickjs::function]
    pub fn set_interval<'js>(
        ctx: Ctx<'js>,
        func: Function<'js>,
        Opt(duration): Opt<u64>,
    ) -> Result<TimerId> {
        let base_class: Class<'js, Base<'js>> = get_base(&ctx)?;

        let mut base = base_class.try_borrow_mut()?;
        let (sx, mut rx) = oneshot::channel();
        let id = base.timeouts.insert(sx);
        drop(base);

        let cloned_ctx = ctx.clone();

        ctx.spawn(async move {
            let mut sleep =
                tokio::time::interval(std::time::Duration::from_millis(duration.unwrap_or(0)));

            loop {
                tokio::select! {
                 _ = &mut rx => {
                    return
                  }
                  _ = sleep.tick() => {
                    if let Err(Error::Exception) = func.call::<_, ()>(()) {
                      let mut base = base_class.try_borrow_mut().unwrap();
                      base.uncaugth_exceptions.push_back(cloned_ctx.catch());

                    }

                  }
                }
            }
        });
        Ok(TimerId { key: id })
    }

    #[rquickjs::function]
    pub fn clear_interval(ctx: Ctx<'_>, key: Class<'_, TimerId>) -> Result<()> {
        let base_class: Class<'_, Base<'_>> = get_base(&ctx)?;
        let mut base = base_class.try_borrow_mut()?;
        if let Some(sx) = base.timeouts.remove(key.try_borrow()?.key) {
            sx.send(()).ok();
        }

        Ok(())
    }

    #[rquickjs::function]
    pub async fn delay<'js>(Opt(duration): Opt<u64>) -> Result<()> {
        let sleep = tokio::time::sleep(std::time::Duration::from_millis(duration.unwrap_or(0)));
        sleep.await;
        Ok(())
    }

    #[rquickjs::function]
    pub async fn throw_uncaught<'js>(ctx: Ctx<'js>) -> Result<()> {
        let base_class: Class<'_, Base<'_>> = get_base(&ctx)?;
        let mut base = base_class.try_borrow_mut()?;
        base.uncaught(ctx)
    }
}
