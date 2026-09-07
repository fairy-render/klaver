use klaver_core::{throw, value::Buffer};
use rquickjs::{
    Class, Ctx, JsLifetime, Value,
    class::Trace,
    prelude::{Opt, This},
};

use crate::streams::readable::state::StreamState;

use super::{reader::ReadableStreamDefaultReader, state::ReadableStreamData};

/// `ReadableStreamBYOBReader`, per <https://streams.spec.whatwg.org/#readablestreambyobreader>.
///
/// Simplification: a real BYOB reader fills the caller-supplied `view` in place (zero-copy,
/// coordinated with the source via `controller.byobRequest`). This implementation instead reads
/// one chunk the same way a default reader would and hands it back as-is - correct byte data,
/// wrong buffer identity (`value` is not `view`, and `view` itself is left untouched/detached).
/// Good enough for code that only cares about the bytes it gets back, not for code that
/// specifically depends on `view`'s buffer being reused.
#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStreamBYOBReader<'js> {
    pub data: Option<Class<'js, ReadableStreamData<'js>>>,
}

#[rquickjs::methods]
impl<'js> ReadableStreamBYOBReader<'js> {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        throw!(
            ctx,
            "ReadableStreamBYOBReader cannot be constructed manually"
        )
    }

    #[qjs(get)]
    pub async fn closed(This(this): This<Class<'js, Self>>, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        let data = this.borrow().data.clone();
        Self::closed_native(data, &ctx).await
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
        view: Buffer<'js>,
    ) -> rquickjs::Result<ReadableStreamBYOBResult<'js>> {
        // `view` itself isn't filled - see the type-level doc comment - but we still validate it
        // was actually given an ArrayBufferView, matching spec's argument validation.
        let _ = view;

        let data = this.borrow().data.clone();
        match (ReadableStreamDefaultReader { data })
            .read_native(&ctx)
            .await?
        {
            Some(value) => Ok(ReadableStreamBYOBResult {
                value: Some(value),
                done: false,
            }),
            None => Ok(ReadableStreamBYOBResult {
                value: None,
                done: true,
            }),
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

impl<'js> ReadableStreamBYOBReader<'js> {
    /// Shared with `ReadableStreamDefaultReader::closed` - the algorithm doesn't depend on the
    /// reader's mode, only on the underlying stream's state.
    pub(crate) async fn closed_native(
        data: Option<Class<'js, ReadableStreamData<'js>>>,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<()> {
        let Some(data) = data else {
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
}

/// The `{ value, done }` shape `read()` resolves with - a plain struct (rather than reusing
/// `IteratorResult`) since `value` is `undefined` (not simply absent) when `done`.
pub struct ReadableStreamBYOBResult<'js> {
    value: Option<Value<'js>>,
    done: bool,
}

impl<'js> rquickjs::IntoJs<'js> for ReadableStreamBYOBResult<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let obj = rquickjs::Object::new(ctx.clone())?;
        obj.set(
            "value",
            self.value
                .unwrap_or_else(|| Value::new_undefined(ctx.clone())),
        )?;
        obj.set("done", self.done)?;
        Ok(obj.into_value())
    }
}

klaver_core::create_export!(ReadableStreamBYOBReader<'js>);
