use std::rc::Rc;

use rquickjs::{
    ArrayBuffer, Class, Ctx, JsLifetime, Promise, String, Value, class::Trace, prelude::Opt, qjs,
};
use rquickjs_util::throw;

use crate::streams::writable::state::{StreamData, WaitDone, WaitReady};

#[derive(Trace)]
#[rquickjs::class]
pub struct WritableStreamDefaultWriter<'js> {
    pub ctrl: Option<Class<'js, StreamData<'js>>>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultWriter<'js> {
    type Changed<'to> = WritableStreamDefaultWriter<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStreamDefaultWriter<'js> {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        throw!(
            ctx,
            "WritableStreamDefaultWriter cannot be constructed manully"
        )
    }

    #[qjs(get)]
    pub async fn ready(&self) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            return Ok(());
        };

        WaitReady::new(ctrl.clone()).await?;

        Ok(())
    }

    pub fn write(&self, ctx: Ctx<'js>, buffer: Value<'js>) -> rquickjs::Result<Promise<'js>> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            todo!()
        };

        let (promise, _, _) = ctrl.borrow_mut().push(ctx.clone(), buffer)?;

        Ok(promise)
    }

    #[qjs(rename = "releaseLock")]
    pub fn release_lock(&mut self) -> rquickjs::Result<()> {
        if let Some(ctrl) = self.ctrl.take() {
            ctrl.borrow_mut().unlock();
        }
        Ok(())
    }

    pub async fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            todo!()
        };

        ctrl.borrow_mut().close(ctx.clone())?;

        WaitDone::new(ctrl.clone()).await?;

        Ok(())
    }

    pub fn abort(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<String<'js>>,
    ) -> rquickjs::Result<Option<String<'js>>> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            todo!()
        };

        ctrl.borrow_mut().abort(ctx, reason.0.clone())?;

        Ok(reason.0)
    }
}
