use klaver::throw;
use rquickjs::{
    atom::PredefinedAtom, class::Trace, CaughtError, Class, Ctx, IntoJs, Object, Value,
};

use super::{controller::ControllerWrap, ReadableStream};

#[derive(Trace)]
#[rquickjs::class]
pub struct ReadableStreamDefaultReader<'js> {
    pub(super) ctrl: ControllerWrap<'js>,
}

// impl<'js> Drop for ReadableStreamDefaultReader<'js> {
//     fn drop(&mut self) {
//         self.ctrl.release().ok();
//     }
// }

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultReader<'js> {
    #[qjs(constructor)]
    pub fn new(
        stream: Class<'js, ReadableStream<'js>>,
    ) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        Ok(ReadableStreamDefaultReader {
            ctrl: ControllerWrap::new(stream.borrow().ctrl.clone()),
        })
    }

    pub async fn read(&self, ctx: Ctx<'js>) -> rquickjs::Result<Chunk<'js>> {
        // Wait for new items
        if self.ctrl.borrow(&ctx)?.is_empty() && !self.ctrl.borrow(&ctx)?.is_done() {
            println!("WAITING");
            self.ctrl.wait(ctx.clone()).await?;
            println!("DONE WAITING");
        }

        if self.ctrl.borrow(&ctx)?.is_canceled() {
            throw!(ctx, "Canceled")
        } else if let Some(err) = self.ctrl.borrow(&ctx)?.has_error() {
            match err {
                CaughtError::Error(err) => {
                    return Err(ctx
                        .clone()
                        .throw(rquickjs::String::from_str(ctx, &err.to_string())?.into_value()))
                }
                CaughtError::Exception(err) => {
                    return Err(ctx.throw(Value::from_exception(err.clone())))
                }
                CaughtError::Value(value) => return Err(ctx.throw(value.clone())),
            }
        } else if self.ctrl.borrow(&ctx)?.is_done() {
            println!("DONE");
            return Ok(Chunk {
                value: None,
                done: true,
            });
        }

        let ret = self.ctrl.borrow_mut(&ctx)?.pop();

        println!("VALUE {:?}", ret);

        Ok(Chunk {
            value: ret,
            done: false,
        })
    }

    pub async fn cancel(
        &self,
        ctx: Ctx<'js>,
        reason: Option<rquickjs::String<'js>>,
    ) -> rquickjs::Result<()> {
        self.ctrl.borrow_mut(&ctx)?.cancel(&ctx, reason)?;
        Ok(())
    }

    #[qjs(rename = "releaseLock")]
    pub fn release_lock(&mut self) -> rquickjs::Result<()> {
        self.ctrl.release()?;
        Ok(())
    }
}

#[derive(Trace)]
pub struct Chunk<'js> {
    pub value: Option<Value<'js>>,
    pub done: bool,
}

impl<'js> IntoJs<'js> for Chunk<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        obj.set(PredefinedAtom::Value, self.value)?;
        obj.set(PredefinedAtom::Done, self.done)?;

        Ok(obj.into_value())
    }
}
