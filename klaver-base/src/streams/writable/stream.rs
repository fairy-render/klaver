use rquickjs::{Class, Ctx, JsLifetime, String, class::Trace, prelude::Opt};

use super::{
    controller::WritableStreamDefaultController,
    underlying_sink::{JsUnderlyingSink, UnderlyingSink},
    writer::WritableStreamDefaultWriter,
};

#[derive(Trace)]
#[rquickjs::class]
pub struct WritableStream<'js> {
    sink: UnderlyingSink<'js>,
    ctrl: Class<'js, WritableStreamDefaultController<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStream<'js> {
    type Changed<'to> = WritableStream<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStream<'js> {
    #[qjs(constructor)]
    fn new(
        &self,
        ctx: Ctx<'js>,
        sink: JsUnderlyingSink<'js>,
    ) -> rquickjs::Result<WritableStream<'js>> {
        todo!()
    }

    async fn abort(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<String<'js>>,
    ) -> rquickjs::Result<Option<String<'js>>> {
        self.ctrl.borrow_mut().abort(ctx, reason.clone())?;
        Ok(reason.0)
    }

    async fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        self.ctrl.borrow_mut().close(ctx)
    }

    #[qjs(rename = "getWriter")]
    fn get_writer(&self) -> rquickjs::Result<WritableStreamDefaultWriter<'js>> {
        if self.ctrl.borrow().wait.locked {
            todo!()
        }

        self.ctrl.borrow_mut().lock();

        Ok(WritableStreamDefaultWriter {
            ctrl: Some(self.ctrl.clone()),
            sink: self.sink.clone(),
        })
    }

    fn locked(&self) -> rquickjs::Result<bool> {
        todo!()
    }
}

fn write<'js>(
    ctx: Ctx<'js>,
    sink: UnderlyingSink<'js>,
    ctrl: Class<'js, WritableStreamDefaultController<'js>>,
) -> rquickjs::Result<()> {
    ctx.clone().spawn(async move {
        //

        sink.start(ctrl.clone());

        loop {
            if ctrl.borrow().is_aborted() {
                sink.abort(ctrl.borrow().abort_reason());
                break;
            }

            if ctrl.borrow().is_closed() && ctrl.borrow().queue.borrow().is_empty() {
                sink.close(ctrl.clone()).await.ok();
            }

            let Some(chunk) = ctrl.borrow().queue.borrow_mut().pop_front() else {
                let notify = ctrl.borrow().wait.wait.clone();
                notify.notified().await;
                continue;
            };

            sink.write(chunk, ctrl.clone()).await;
        }
    });
    Ok(())
}
