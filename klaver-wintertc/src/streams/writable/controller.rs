use crate::{
    abort_controller::{AbortController, AbortSignal},
    streams::data::StreamData,
};
use klaver_core::throw;
use rquickjs::{Class, Ctx, JsLifetime, Value, class::Trace, prelude::Opt};

#[rquickjs::class]
#[derive(Trace)]
pub struct WritableStreamDefaultController<'js> {
    pub data: Class<'js, StreamData<'js>>,
    /// Backs `.signal`: aborted when the stream is aborted, so a sink's in-flight
    /// `write()`/`close()` can observe cancellation.
    abort_controller: AbortController<'js>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultController<'js> {
    type Changed<'to> = WritableStreamDefaultController<'to>;
}

impl<'js> WritableStreamDefaultController<'js> {
    pub fn new_with(
        ctx: &Ctx<'js>,
        data: Class<'js, StreamData<'js>>,
    ) -> rquickjs::Result<WritableStreamDefaultController<'js>> {
        Ok(WritableStreamDefaultController {
            data,
            abort_controller: AbortController::new(ctx.clone())?,
        })
    }

    /// Fires `.signal`'s abort, per spec, so a sink mid-`write()`/`close()` observes it.
    /// Called by the resource loop once it sees the stream has been aborted.
    pub fn signal_abort(&self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        self.abort_controller.abort(ctx.clone(), Opt(reason))
    }
}

#[rquickjs::methods]
impl<'js> WritableStreamDefaultController<'js> {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        throw!(
            ctx,
            "WritableStreamDefaultController cannot be constructed manually"
        )
    }

    #[qjs(get)]
    pub fn signal(&self) -> Class<'js, AbortSignal<'js>> {
        self.abort_controller.signal_handle()
    }

    fn error(&self, ctx: Ctx<'js>, error: Value<'js>) -> rquickjs::Result<()> {
        self.data.borrow_mut().fail(&ctx, error)?;

        Ok(())
    }
}

klaver_core::create_export!(WritableStreamDefaultController<'js>);
