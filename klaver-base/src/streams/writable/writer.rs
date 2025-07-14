use std::rc::Rc;

use async_notify::Notify;
use rquickjs::{ArrayBuffer, Class, Ctx, JsLifetime, String, class::Trace};

use super::{controller::WritableStreamDefaultController, underlying_sink::UnderlyingSink};

#[derive(Trace)]
#[rquickjs::class]
pub struct WritableStreamDefaultWriter<'js> {
    pub ctrl: Option<Class<'js, WritableStreamDefaultController<'js>>>,
    pub sink: UnderlyingSink<'js>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultWriter<'js> {
    type Changed<'to> = WritableStreamDefaultWriter<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStreamDefaultWriter<'js> {
    async fn ready(&self) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            return Ok(());
        };

        ctrl.borrow().ready().await?;

        Ok(())
    }

    async fn write(&self, ctx: Ctx<'js>, buffer: ArrayBuffer<'js>) -> rquickjs::Result<()> {
        let Some(ctrl) = self.ctrl.as_ref() else {
            todo!()
        };

        ctrl.borrow().write(buffer.into_value()).await?;

        Ok(())
    }

    fn release_lock(&mut self) -> rquickjs::Result<()> {
        if let Some(mut ctrl) = self.ctrl.take() {
            ctrl.borrow_mut().unlock();
        }
        Ok(())
    }

    fn close(&self) -> rquickjs::Result<()> {
        Ok(())
    }

    fn abort(&self, reason: Option<String<'js>>) -> rquickjs::Result<Option<String<'js>>> {
        todo!()
    }
}
