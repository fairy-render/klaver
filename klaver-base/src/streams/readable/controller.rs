use klaver_util::throw;
use rquickjs::{Class, Ctx, JsLifetime, Value, class::Trace};

use crate::streams::readable::state::ReadableStreamData;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStreamDefaultController<'js> {
    pub data: Class<'js, ReadableStreamData<'js>>,
    pub enqueued: bool,
}

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultController<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultController<'js>> {
        throw!(ctx, "ReadableStreamDefaultController cannot be constructed")
    }

    pub fn enqueue(&mut self, ctx: Ctx<'js>, data: Value<'js>) -> rquickjs::Result<()> {
        self.enqueued = true;

        let mut state = self.data.borrow_mut();

        if state.is_cancled() || state.is_failed() || state.is_closed() {
            throw!(@type ctx, "Stream is closed")
        }

        state.push(&ctx, data)
    }

    pub fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let mut state = self.data.borrow_mut();

        if state.is_cancled() || state.is_failed() || state.is_closed() {
            throw!(@type ctx, "Stream is closed")
        }

        state.close(&ctx)
    }

    pub fn error(&self, ctx: Ctx<'js>, value: Value<'js>) -> rquickjs::Result<()> {
        self.data.borrow_mut().fail(&ctx, Some(value))
    }
}

create_export!(ReadableStreamDefaultController<'js>);
