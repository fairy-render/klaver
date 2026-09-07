use klaver_core::{throw, value::Buffer};
use rquickjs::{Class, Ctx, FromJs, JsLifetime, Value, class::Trace};

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

        // `ReadableByteStreamController.enqueue()` (this same controller, reused for byte
        // streams - see the module-level note) requires an ArrayBufferView per spec.
        if state.is_byte_stream && Buffer::from_js(&ctx, data.clone()).is_err() {
            throw!(@type ctx, "Can only enqueue an ArrayBufferView on a byte stream")
        }

        state.push(&ctx, data)
    }

    /// Always `null`: this implementation doesn't support the zero-copy `byobRequest`
    /// pull-directly-into-a-caller-buffer path. A BYOB reader still works (see
    /// `ReadableStreamBYOBReader`), just via an internal copy rather than filling the exact
    /// buffer the source was asked to write into.
    #[qjs(get, rename = "byobRequest")]
    pub fn byob_request(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(Value::new_null(ctx))
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

    #[qjs(get, rename = "desiredSize")]
    pub fn desired_size(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let data = self.data.borrow();
        let size = if data.is_failed() || data.is_cancled() {
            None
        } else {
            Some(data.queue.desired_size())
        };
        crate::streams::desired_size_value(&ctx, size)
    }
}

klaver_core::create_export!(ReadableStreamDefaultController<'js>);
