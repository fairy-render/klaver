use crate::streams::data::{StreamData, WaitDone, WaitReadReady, WaitWriteReady};
use klaver_util::throw;
use rquickjs::{
    Class, Ctx, FromJs, IntoJs, JsLifetime, Object, String, Value, atom::PredefinedAtom,
    class::Trace, prelude::Opt,
};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStreamDefaultReader<'js> {
    pub data: Option<Class<'js, StreamData<'js>>>,
}

impl<'js> ReadableStreamDefaultReader<'js> {}

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultReader<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        throw!(ctx, "ReadableStreamDefaultReader cannot be constructed")
    }

    #[qjs(get)]
    pub async fn closed(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let Some(data) = &self.data else {
            throw!(@type ctx, "This reader does not hold a lock to the stream")
        };

        WaitDone::new(data.clone()).await?;

        Ok(())
    }

    pub fn cancel(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<String<'js>>,
    ) -> rquickjs::Result<Option<String<'js>>> {
        let Some(data) = &self.data else {
            throw!(@type ctx, "This reader does not hold a lock to the stream")
        };

        data.borrow_mut().abort(&ctx, reason.0.clone())?;

        Ok(reason.0)
    }

    pub async fn read(&self, ctx: Ctx<'js>) -> rquickjs::Result<Chunk<'js>> {
        let Some(data) = &self.data else {
            throw!(@type ctx, "This reader does not hold a lock to the stream")
        };

        if !data.borrow().is_running() {
            throw!(@type ctx, "Stream is closed")
        }

        WaitReadReady::new(data.clone()).await?;

        println!("{:?}", &*data.borrow());

        if let Some(entry) = data.borrow_mut().pop() {
            Ok(Chunk {
                value: entry.chunk.into(),
                done: false,
            })
        } else {
            Ok(Chunk {
                value: None,
                done: true,
            })
        }
    }

    #[qjs(rename = "releaseLock")]
    pub fn release_lock(&mut self) -> rquickjs::Result<()> {
        if let Some(ctrl) = self.data.take() {
            ctrl.borrow_mut().unlock();
        }
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

impl<'js> FromJs<'js> for Chunk<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_value(value)?;

        Ok(Chunk {
            value: obj.get(PredefinedAtom::Value)?,
            done: obj.get(PredefinedAtom::Done)?,
        })
    }
}

create_export!(ReadableStreamDefaultReader<'js>);
