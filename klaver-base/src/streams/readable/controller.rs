use rquickjs::{Class, Ctx, JsLifetime, Value, class::Trace};
use rquickjs_util::throw;

use crate::streams::data::StreamData;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStreamDefaultController<'js> {
    pub data: Class<'js, StreamData<'js>>,
}

#[rquickjs::methods]
impl<'js> ReadableStreamDefaultController<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultController<'js>> {
        throw!(ctx, "ReadableStreamDefaultController cannot be constructed")
    }

    pub fn enqueue(&self, ctx: Ctx<'js>, data: Value<'js>) -> rquickjs::Result<()> {
        self.data.borrow_mut().push(ctx, data)?;
        Ok(())
    }

    pub fn close(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        self.data.borrow_mut().close(ctx)
    }

    pub fn error(&self, ctx: Ctx<'js>, value: Value<'js>) -> rquickjs::Result<()> {
        self.data.borrow_mut().fail(ctx, value)
    }
}
