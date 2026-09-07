use klaver_core::throw;
use rquickjs::{Class, Ctx, JsLifetime, Value, class::Trace};

use crate::streams::readable::ReadableStreamData;

/// `TransformStreamDefaultController`, per
/// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller>. Operates directly on the
/// readable side's shared state - enqueuing/erroring/terminating here is exactly the same as the
/// readable side's own `ReadableStreamDefaultController` would do, just under the spec's
/// transform-stream-facing names (`terminate` instead of `close`).
#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct TransformStreamDefaultController<'js> {
    pub readable: Class<'js, ReadableStreamData<'js>>,
}

#[rquickjs::methods]
impl<'js> TransformStreamDefaultController<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        throw!(
            ctx,
            "TransformStreamDefaultController cannot be constructed manually"
        )
    }

    #[qjs(get, rename = "desiredSize")]
    pub fn desired_size(&self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let data = self.readable.borrow();
        let size = if data.is_failed() || data.is_cancled() {
            None
        } else {
            Some(data.queue.desired_size())
        };
        crate::streams::desired_size_value(&ctx, size)
    }

    pub fn enqueue(&self, ctx: Ctx<'js>, chunk: Value<'js>) -> rquickjs::Result<()> {
        let mut data = self.readable.borrow_mut();

        if data.is_cancled() || data.is_failed() || data.is_closed() {
            throw!(@type ctx, "Readable side of the TransformStream is closed")
        }

        data.push(&ctx, chunk)
    }

    pub fn error(&self, ctx: Ctx<'js>, reason: Value<'js>) -> rquickjs::Result<()> {
        let mut data = self.readable.borrow_mut();
        if data.is_cancled() || data.is_failed() || data.is_closed() {
            return Ok(());
        }
        data.fail(&ctx, Some(reason))
    }

    pub fn terminate(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let mut data = self.readable.borrow_mut();
        if data.is_cancled() || data.is_failed() || data.is_closed() {
            return Ok(());
        }
        data.close(&ctx)
    }
}

klaver_core::create_export!(TransformStreamDefaultController<'js>);
