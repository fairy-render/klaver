use futures::FutureExt;
use klaver_runtime::{Resource, ResourceId};
use rquickjs::Class;

use crate::streams::readable::{
    controller::ReadableStreamDefaultController,
    source::UnderlyingSource,
    state::{ReadableStreamData, StreamState},
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
            todo!()
        }

        let mut should_pull = true;

        loop {
            // Break if the stream is closed and the queue is empty
            if self.data.borrow().is_closed() && self.data.borrow().queue.is_empty() {
                // self.data.borrow_mut().state.set(StreamState::Done);
                break;
            } else if self.data.borrow().is_cancled() {
                if let Err(err) = self
                    .source
                    .cancel(ctx.ctx().clone(), self.data.borrow().reason.clone())
                    .await
                {}

                break;
            } else if !self.data.borrow().is_running() {
                // todo!("Not running")
                break;
            }

            if self.data.borrow().queue.is_full() {
                let state = self.data.borrow().state.subscribe();
                let queue = self.data.borrow().queue.subscribe();

                futures::select! {
                    _ = state.fuse() => {
                        continue;
                    }
                    _ = queue.fuse() => {
                        if self.data.borrow().queue.is_full() {
                            continue
                        }
                    }
                }
            }

            if should_pull {
                ctrl.borrow_mut().enqueued = false;
                if let Err(err) = self.source.pull(ctx.ctx().clone(), ctrl.clone()).await {
                    todo!()
                }

                if !ctrl.borrow().enqueued {
                    should_pull = false;
                }
            }
        }

        self.data.borrow_mut().resource_active.set(false);

        Ok(())
    }
}
