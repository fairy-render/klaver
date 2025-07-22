use event_listener::listener;
use futures::FutureExt;
use klaver_runner::{Shutdown, Workers};
use rquickjs::{CatchResultExt, Class, Ctx, JsLifetime, String, class::Trace, prelude::Opt};
use rquickjs_util::throw;

use crate::streams::{data::StreamData, queue_strategy::QueuingStrategy};

use super::{
    controller::WritableStreamDefaultController,
    underlying_sink::{JsUnderlyingSink, UnderlyingSink},
    writer::WritableStreamDefaultWriter,
};

#[derive(Trace)]
#[rquickjs::class]
pub struct WritableStream<'js> {
    state: Class<'js, StreamData<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStream<'js> {
    type Changed<'to> = WritableStream<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStream<'js> {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'js>, sink: JsUnderlyingSink<'js>) -> rquickjs::Result<WritableStream<'js>> {
        let state = StreamData::new(QueuingStrategy::create_default(ctx.clone())?);

        let state = Class::instance(ctx.clone(), state)?;

        let ctrl = Class::instance(
            ctx.clone(),
            WritableStreamDefaultController {
                data: state.clone(),
            },
        )?;

        let state_clone = state.clone();
        let worker = Workers::from_ctx(&ctx)?;
        worker.push(ctx.clone(), |ctx, shutdown| async move {
            write(
                ctx.clone(),
                UnderlyingSink::Quick(sink),
                ctrl,
                state_clone,
                shutdown,
            )
            .await
            .catch(&ctx)?;
            Ok(())
        });

        // write(
        //     ctx.clone(),
        //     UnderlyingSink::Quick(sink),
        //     ctrl,
        //     state.clone(),
        // )?;

        Ok(WritableStream { state })
    }

    async fn abort(
        &self,
        ctx: Ctx<'js>,
        reason: Opt<String<'js>>,
    ) -> rquickjs::Result<Option<String<'js>>> {
        if self.state.borrow().is_locked() {
            throw!(@type ctx, "The stream you are trying to abort is locked.")
        }
        let writer = self.get_writer(ctx.clone())?;

        let ret = writer.abort(ctx, reason);

        ret
    }

    #[qjs(rename = "getWriter")]
    pub fn get_writer(&self, ctx: Ctx<'js>) -> rquickjs::Result<WritableStreamDefaultWriter<'js>> {
        if self.state.borrow().is_locked() {
            throw!(@type ctx, "The stream you are trying to create a writer for is already locked to another writer")
        }

        self.state.borrow_mut().lock(ctx)?;

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
        Ok(self.state.borrow().is_locked())
    }
}

async fn write<'js>(
    ctx: Ctx<'js>,
    sink: UnderlyingSink<'js>,
    ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    data: Class<'js, StreamData<'js>>,
    mut shutdown: Shutdown,
) -> rquickjs::Result<()> {
    if shutdown.is_killed() {
        return Ok(());
    }
    if let Err(err) = sink.start(ctrl.clone()).await {
        if err.is_exception() {
            let failure = ctx.catch();
            data.borrow_mut().fail(ctx.clone(), failure).ok();
        }
        return Ok(());
    }

    loop {
        if data.borrow().is_aborted() {
            sink.abort(data.borrow().abort_reason()).await.ok();
            break;
        }

        if data.borrow().is_closed() && data.borrow().queue.is_empty() {
            sink.close(ctrl.clone()).await.ok();
            data.borrow_mut().finished().ok();
            break;
        }

        if data.borrow().is_failed() {
            break;
        }

        let Some(chunk) = data.borrow_mut().pop() else {
            let notify = data.borrow().wait.clone();
            listener!(notify => listener);
            futures::select! {
                _ = listener.fuse() => {
                    continue
                }
                _ = shutdown => {

                }
            };
            continue;
        };

        if shutdown.is_killed() {
            return Ok(());
        }

        if let Err(err) = sink.write(chunk.chunk, ctrl.clone()).await {
            if err.is_exception() {
                let failure = ctx.catch();
                chunk.reject.call::<_, ()>((failure.clone(),)).ok();
                data.borrow_mut().fail(ctx.clone(), failure).ok();
            }
            break;
        }

        chunk.resolve.call::<_, ()>(()).ok();
    }
    Ok(())
}

create_export!(WritableStream<'js>);
