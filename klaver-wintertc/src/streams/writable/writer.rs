use klaver_core::throw;
use rquickjs::{Class, Ctx, JsLifetime, Promise, Value, class::Trace, prelude::Opt};

use crate::streams::data::{StreamData, WaitDone, WaitWriteReady};

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
            "WritableStreamDefaultWriter cannot be constructed manually"
        )
    }

    #[qjs(get)]
    pub async fn ready(&self) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            return Ok(());
        };

        WaitWriteReady::new(ctrl.clone()).await?;

        Ok(())
    }

    /// How much more (by strategy-defined size units) can be written before the stream is
    /// considered full: `null` if errored, `0` if not writable, else `highWaterMark - size`.
    #[qjs(get, rename = "desiredSize")]
    pub fn desired_size(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            throw!(@type ctx, "This writable stream writer has been released")
        };

        let data = ctrl.borrow();
        let size = if data.is_failed() || data.is_aborted() {
            None
        } else if !data.is_running() {
            Some(0.0)
        } else {
            Some(data.queue.desired_size())
        };
        crate::streams::desired_size_value(&ctx, size)
    }

    /// Resolves once the stream finishes closing; rejects if it errors or is aborted instead.
    #[qjs(get)]
    pub async fn closed(&self) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            return Ok(());
        };

        WaitDone::new(ctrl.clone()).await?;

        Ok(())
    }

    pub fn write(&self, ctx: Ctx<'js>, buffer: Value<'js>) -> rquickjs::Result<Promise<'js>> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            throw!(@type ctx, "This writable stream writer has been released")
        };

        // Per spec: writing to a stream that isn't in the writable state returns a *rejected*
        // promise (with the stream's error/abort reason if there is one), rather than throwing.
        if !ctrl.borrow().is_running() {
            let (promise, _, reject) = Promise::new(&ctx)?;
            let reason = ctrl.borrow().state_error_value(&ctx)?;
            reject.call::<_, ()>((reason,))?;
            return Ok(promise);
        }

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
            throw!(@type ctx, "This writable stream writer has been released")
        };

        ctrl.borrow_mut().close(&ctx)?;

        WaitDone::new(ctrl.clone()).await?;

        Ok(())
    }

    pub fn abort(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<Value<'js>>,
    ) -> rquickjs::Result<Option<Value<'js>>> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            throw!(@type ctx, "This writable stream writer has been released")
        };

        ctrl.borrow_mut().abort(&ctx, reason.0.clone())?;

        Ok(reason.0)
    }
}

klaver_core::create_export!(WritableStreamDefaultWriter<'js>);
