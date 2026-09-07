use futures::FutureExt;
use klaver_core::{throw, value::iterable::IteratorResult};
use rquickjs::{
    Class, Ctx, JsLifetime, Value,
    class::Trace,
    prelude::{Opt, This},
};

use crate::streams::readable::state::StreamState;

use super::state::ReadableStreamData;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStreamDefaultReader<'js> {
    pub data: Option<Class<'js, ReadableStreamData<'js>>>,
}

impl<'js> ReadableStreamDefaultReader<'js> {
    pub async fn read_native(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Value<'js>>> {
        let Some(data) = &self.data else {
            throw!(@type ctx, "Lock released");
        };

        if data.borrow().is_failed() || data.borrow().is_cancled() {
            if let Some(data) = data.borrow().reason.clone() {
                return Err(ctx.throw(data));
            }
            throw!(@type ctx, "Stream was cancled")
        }

        // Not data in the queue, so we'll wait for a state change
        if data.borrow().queue.is_empty() && !data.borrow().is_closed() {
            loop {
                let state = data.borrow().state.subscribe();
                let queue = data.borrow().queue.subscribe();

                futures::select! {
                    _ = state.fuse() => {
                        // A state change means some kind of errors happended
                        break;
                    }
                    _ = queue.fuse() => {
                        if !data.borrow().queue.is_empty() {
                            break;
                        }
                    }
                }
            }

            if data.borrow().is_failed() || data.borrow().is_cancled() {
                if let Some(data) = data.borrow().reason.clone() {
                    return Err(ctx.throw(data));
                }
                throw!(@type ctx, "Stream was cancled")
            }
        }

        Ok(data.borrow_mut().queue.pop())
    }

    /// The implementation behind the JS-facing `cancel()`, usable directly on a plain
    /// (non-`Class`-wrapped) reader value - e.g. by `pipeTo`, which needs to cancel the source
    /// on a writable-side failure without round-tripping through a JS `Class` handle.
    pub async fn cancel_native(
        &self,
        ctx: &Ctx<'js>,
        reason: Option<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let Some(data) = &self.data else {
            throw!(@type ctx, "Lock released");
        };

        if data.borrow().is_failed() || data.borrow().is_cancled() {
            throw!(@type ctx, "Stream already canceled");
        }

        data.borrow_mut().cancel(ctx, reason)?;

        loop {
            if !data.borrow().resource_active.get() {
                break;
            }

            let listener = data.borrow().resource_active.subscribe();
            listener.await;
        }

        Ok(())
    }
}

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultReader<'js> {
    #[qjs(get)]
    pub async fn closed(This(this): This<Class<'js, Self>>, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let Some(data) = this.borrow().data.clone() else {
            throw!(@type ctx, "Lock released");
        };

        loop {
            if !data.borrow().is_locked() {
                throw!(@type ctx, "Lock released")
            }

            let state = data.borrow().state.get();
            match state {
                StreamState::Aborted | StreamState::Failed => {
                    if let Some(err) = data.borrow().reason.clone() {
                        return Err(ctx.throw(err));
                    } else {
                        throw!(@type ctx, "Stream was canceled")
                    }
                }
                StreamState::Closed => {
                    if *data.borrow().resource_active {
                        let listener = data.borrow().resource_active.subscribe();
                        listener.await;
                    }

                    return Ok(());
                }
                StreamState::Running => {
                    let listener = data.borrow().state.subscribe();
                    let lock = data.borrow().locked.subscribe();

                    futures::future::select(listener, lock).await;
                }
            }
        }
    }

    pub async fn cancel(
        This(this): This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        reason: Opt<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let data = this.borrow().data.clone();
        ReadableStreamDefaultReader { data }
            .cancel_native(&ctx, reason.0)
            .await
    }

    pub async fn read(
        This(this): This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<IteratorResult<Value<'js>>> {
        let data = this.borrow().data.clone();
        match (ReadableStreamDefaultReader { data })
            .read_native(&ctx)
            .await?
        {
            Some(value) => Ok(IteratorResult::Value(value)),
            None => Ok(IteratorResult::Done),
        }
    }

    #[qjs(rename = "releaseLock")]
    pub fn release_lock(&mut self) {
        let Some(data) = self.data.take() else {
            return;
        };

        if data.borrow().is_locked() {
            data.borrow_mut().locked.set(false);
        }
    }
}

klaver_core::create_export!(ReadableStreamDefaultReader<'js>);
