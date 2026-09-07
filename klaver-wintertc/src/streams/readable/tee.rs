use async_trait::async_trait;
use rquickjs::{Class, Ctx, Value, class::Trace};

use super::{
    NativeSource, ReadableStreamDefaultController, ReadableStreamDefaultReader,
    state::ReadableStreamData,
};

/// The "driving" branch of a `tee()`: performs every actual read from the shared underlying
/// reader, enqueues each chunk into its own queue as usual, and additionally pushes a copy
/// directly into the sibling ("passive") branch's queue. Cancelling only actually cancels the
/// shared source once the sibling has also given up (or was already done) - so cancelling one
/// branch alone doesn't cut off the other.
///
/// Only one side ever calls `read()` on the shared reader (there's no possible race between the
/// two branches over who gets which chunk); the sibling's own `pull()` is a no-op ([`Passive`]),
/// relying entirely on chunks pushed here. A consequence (shared with real implementations):
/// if the passive branch is never read, its queue eventually fills to its high water mark,
/// which - since a `NativeSource`'s `pull()` is only invoked while its own queue has room -
/// eventually stops *this* branch from pulling too, per spec's intent that tee'd branches keep
/// each other's backpressure honest rather than let an ignored branch grow unboundedly.
pub struct Driving<'js> {
    reader: ReadableStreamDefaultReader<'js>,
    other: Class<'js, ReadableStreamData<'js>>,
}

impl<'js> Driving<'js> {
    pub fn new(
        reader: ReadableStreamDefaultReader<'js>,
        other: Class<'js, ReadableStreamData<'js>>,
    ) -> Self {
        Driving { reader, other }
    }
}

impl<'js> Trace<'js> for Driving<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.reader.trace(tracer);
        self.other.trace(tracer);
    }
}

#[async_trait(?Send)]
impl<'js> NativeSource<'js> for Driving<'js> {
    async fn start(
        &mut self,
        _ctx: Ctx<'js>,
        _ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    async fn pull(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        match self.reader.read_native(&ctx).await? {
            Some(chunk) => {
                ctrl.borrow_mut().enqueue(ctx.clone(), chunk.clone())?;

                let mut other = self.other.borrow_mut();
                if other.is_running() {
                    other.push(&ctx, chunk)?;
                }
            }
            None => {
                ctrl.borrow_mut().close(ctx.clone()).ok();

                let mut other = self.other.borrow_mut();
                if other.is_running() {
                    other.close(&ctx).ok();
                }
            }
        }

        Ok(())
    }

    async fn cancel(&mut self, ctx: Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        if !self.other.borrow().is_running() {
            self.reader.cancel_native(&ctx, reason).await?;
        }
        Ok(())
    }
}

/// The "passive" branch of a `tee()` - see [`Driving`]. Never reads anything itself; entirely
/// fed by chunks the driving branch pushes in directly.
pub struct Passive;

impl<'js> Trace<'js> for Passive {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[async_trait(?Send)]
impl<'js> NativeSource<'js> for Passive {
    async fn start(
        &mut self,
        _ctx: Ctx<'js>,
        _ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    async fn pull(
        &mut self,
        _ctx: Ctx<'js>,
        _ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }
}
