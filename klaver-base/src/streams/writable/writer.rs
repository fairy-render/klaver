use std::rc::Rc;

use rquickjs::{ArrayBuffer, Class, Ctx, JsLifetime, String, class::Trace, prelude::Opt};

use crate::streams::writable::state::StreamData;

use super::{controller::WritableStreamDefaultController, underlying_sink::UnderlyingSink};

#[derive(Trace)]
#[rquickjs::class]
pub struct WritableStreamDefaultWriter<'js> {
    pub ctrl: Option<Rc<StreamData<'js>>>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultWriter<'js> {
    type Changed<'to> = WritableStreamDefaultWriter<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStreamDefaultWriter<'js> {
    #[qjs(get)]
    pub async fn ready(&self) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            return Ok(());
        };

        ctrl.ready().await?;

        Ok(())
    }

    pub async fn write(&self, ctx: Ctx<'js>, buffer: ArrayBuffer<'js>) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            todo!()
        };

        ctrl.push(ctx, buffer.into_value()).await?;

        Ok(())
    }

    pub fn release_lock(&mut self) -> rquickjs::Result<()> {
        if let Some(ctrl) = self.ctrl.take() {
            ctrl.unlock();
        }
        Ok(())
    }

    pub async fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            todo!()
        };

        ctrl.close(ctx.clone())?;
        ctrl.wait_done(ctx).await?;

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

        ctrl.abort(ctx, reason.0.clone())?;

        Ok(reason.0)
    }
}
