use futures::FutureExt;
use klaver_runtime::{Resource, ResourceId};
use rquickjs::{CaughtError, Class};

use crate::streams::readable::{
    controller::ReadableStreamDefaultController, source::UnderlyingSource,
    state::ReadableStreamData,
};

pub struct ReadableStreamResourceId;

impl ResourceId for ReadableStreamResourceId {
    fn name() -> &'static str {
        "ReadableStreamWrap"
    }
}

pub struct ReadableStreamResource<'js> {
    pub data: Class<'js, ReadableStreamData<'js>>,
    pub source: UnderlyingSource<'js>,
}

/// Turns a source error into an event that fails the stream (matching the spec: an exception
/// thrown from `start`/`pull` errors the stream, the same as the source calling
/// `controller.error()` itself).
fn fail_from_source<'js>(
    ctx: &rquickjs::Ctx<'js>,
    data: &Class<'js, ReadableStreamData<'js>>,
    err: CaughtError<'js>,
) {
    let reason = match err {
        CaughtError::Error(e) => rquickjs::String::from_str(ctx.clone(), &e.to_string())
            .map(|s| s.into_value())
            .ok(),
        CaughtError::Exception(e) => Some(e.into_value()),
        CaughtError::Value(v) => Some(v),
    };
    // A stream that's already closed/failed/cancelled can't be failed again; ignore that case
    // (the source raced with the consumer, the consumer's outcome wins).
    data.borrow_mut().fail(ctx, reason).ok();
}

impl<'js> Resource<'js> for ReadableStreamResource<'js> {
    type Id = ReadableStreamResourceId;
    const INTERNAL: bool = true;
    const SCOPED: bool = true;

    async fn run(mut self, ctx: klaver_runtime::Context<'js>) -> rquickjs::Result<()> {
        let ctrl = Class::instance(
            ctx.ctx().clone(),
            ReadableStreamDefaultController {
                data: self.data.clone(),
                enqueued: false,
            },
        )?;

        if let Err(err) = self.source.start(ctx.ctx().clone(), ctrl.clone()).await {
            fail_from_source(ctx.ctx(), &self.data, err);
        }

        let mut should_pull = true;

        loop {
            // Break if the stream is closed and the queue is empty
            if self.data.borrow().is_closed() && self.data.borrow().queue.is_empty() {
                break;
            } else if self.data.borrow().is_cancled() {
                if let Err(err) = self
                    .source
                    .cancel(ctx.ctx().clone(), self.data.borrow().reason.clone())
                    .await
                {
                    // The stream is already cancelled either way; just report the source's
                    // cancellation error rather than silently dropping it.
                    eprintln!("Uncaught {err}");
                }

                break;
            } else if !self.data.borrow().is_running() {
                break;
            }

            if self.data.borrow().queue.is_full() || !should_pull {
                // Either there's no room to pull more right now, or the last `pull()` call
                // didn't produce anything (`should_pull` false) and we're waiting for some
                // reason to try again. Either way, wait for a real signal instead of spinning:
                // a queue change (a push *or* a pop) means something's different, so give the
                // source another chance next iteration.
                let state = self.data.borrow().state.subscribe();
                let queue = self.data.borrow().queue.subscribe();

                futures::select! {
                    _ = state.fuse() => {}
                    _ = queue.fuse() => {
                        should_pull = true;
                    }
                }
                continue;
            }

            ctrl.borrow_mut().enqueued = false;
            if let Err(err) = self.source.pull(ctx.ctx().clone(), ctrl.clone()).await {
                fail_from_source(ctx.ctx(), &self.data, err);
                continue;
            }

            if !ctrl.borrow().enqueued {
                should_pull = false;
            }
        }

        self.data.borrow_mut().resource_active.set(false);

        Ok(())
    }
}
