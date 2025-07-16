use crate::streams::data::StreamData;
use rquickjs::{Class, Ctx, JsLifetime, Value, class::Trace};
use rquickjs_util::throw;

#[rquickjs::class]
#[derive(Trace)]
pub struct WritableStreamDefaultController<'js> {
    pub data: Class<'js, StreamData<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultController<'js> {
    type Changed<'to> = WritableStreamDefaultController<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStreamDefaultController<'js> {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        throw!(
            ctx,
            "WritableStreamDefaultController cannot be constructed manully"
        )
    }

    fn error(&self, ctx: Ctx<'js>, error: Value<'js>) -> rquickjs::Result<()> {
        self.data.borrow_mut().fail(ctx, error)?;

        Ok(())
    }
}
