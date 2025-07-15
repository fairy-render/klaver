use std::rc::Rc;

use event_listener::listener;
use rquickjs::{Class, Ctx, JsLifetime, String, class::Trace, prelude::Opt};

use crate::streams::{queue_strategy::QueuingStrategy, writable::state::StreamData};

use super::{
    controller::WritableStreamDefaultController,
    underlying_sink::{JsUnderlyingSink, UnderlyingSink},
    writer::WritableStreamDefaultWriter,
};

#[derive(Trace)]
#[rquickjs::class]
pub struct WritableStream<'js> {
    state: Rc<StreamData<'js>>,
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
        let state = Rc::new(StreamData::new(QueuingStrategy::create_default(
            ctx.clone(),
        )?));
        let ctrl = Class::instance(
            ctx.clone(),
            WritableStreamDefaultController {
                data: state.clone(),
            },
        )?;

        write(ctx, UnderlyingSink::Quick(sink), ctrl, state.clone())?;

        Ok(WritableStream { state })
    }

    async fn abort(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<String<'js>>,
    ) -> rquickjs::Result<Option<String<'js>>> {
        let writer = self.get_writer(ctx.clone())?;

        let ret = writer.abort(ctx, reason);

        ret
    }

    #[qjs(rename = "getWriter")]
    fn get_writer(&self, ctx: Ctx<'js>) -> rquickjs::Result<WritableStreamDefaultWriter<'js>> {
        if self.state.is_locked() {
            todo!()
        }

        self.state.lock(ctx)?;

        Ok(WritableStreamDefaultWriter {
            ctrl: Some(self.state.clone()),
        })
    }

    async fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let writer = self.get_writer(ctx.clone())?;

        writer.close(ctx).await?;

        Ok(())
    }

    #[qjs(get)]
    fn locked(&self) -> rquickjs::Result<bool> {
        Ok(self.state.is_locked())
    }
}

macro_rules! throw_exception {
    ($ctx: expr, $data:expr, $expr: expr) => {
        if let Err(err) = $expr {
            if err.is_exception() {
                let failure = $ctx.catch();
                $data.fail($ctx, failure);
            }
            break;
        }
    };

    (@ret $ctx: expr, $data:expr, $expr: expr) => {
        if let Err(err) = $expr {
            if err.is_exception() {
                let failure = $ctx.catch();
                $data.fail($ctx, failure).ok();
            }
            return;
        }
    };
}

fn write<'js>(
    ctx: Ctx<'js>,
    sink: UnderlyingSink<'js>,
    ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    data: Rc<StreamData<'js>>,
) -> rquickjs::Result<()> {
    ctx.clone().spawn(async move {
        if let Err(err) = sink.start(ctrl.clone()).await {
            if err.is_exception() {
                let failure = ctx.catch();
                data.fail(ctx.clone(), failure).ok();
            }
            return;
        }

        loop {
            if data.is_aborted() {
                sink.abort(data.abort_reason()).await.ok();
                break;
            }

            if data.is_closed() && data.queue.borrow().is_empty() {
                sink.close(ctrl.clone()).await.ok();
                break;
            }

            if data.is_failed() {
                break;
            }

            let Some(chunk) = data.queue.borrow_mut().pop() else {
                let notify = data.wait.clone();
                listener!(notify => listener);
                listener.await;
                continue;
            };

            if let Err(err) = sink.write(chunk, ctrl.clone()).await {
                if err.is_exception() {
                    let failure = ctx.catch();
                    data.fail(ctx.clone(), failure).ok();
                }
                break;
            }
        }
    });
    Ok(())
}
