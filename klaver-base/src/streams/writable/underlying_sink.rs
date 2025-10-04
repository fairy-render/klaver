use std::rc::Rc;

use async_trait::async_trait;
use rquickjs::{Class, Ctx, FromJs, Function, Object, Value, class::Trace};

use super::controller::WritableStreamDefaultController;

#[derive(Clone)]
pub enum UnderlyingSink<'js> {
    Quick(JsUnderlyingSink<'js>),
    Native(Rc<dyn NativeSink<'js>>),
}

impl<'js> Trace<'js> for UnderlyingSink<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Quick(m) => m.trace(tracer),
            Self::Native(m) => m.trace(tracer),
        }
    }
}

impl<'js> UnderlyingSink<'js> {
    pub async fn start(
        &self,
        ctx: &Ctx<'js>,
        ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Quick(quick) => {
                if let Some(start) = &quick.start {
                    let value = start.call::<_, Value<'js>>((ctrl,))?;
                    if let Some(promise) = value.as_promise() {
                        promise.clone().into_future::<Value<'js>>().await?;
                    }
                }
            }
            Self::Native(native) => native.start(ctx, ctrl).await?,
        }
        Ok(())
    }
    pub async fn abort(&self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()> {
        match self {
            Self::Quick(quick) => {
                if let Some(abort) = &quick.abort {
                    abort.call::<_, ()>((reason,))?;
                }
            }
            Self::Native(native) => native.abort(ctx, reason).await?,
        }
        Ok(())
    }

    pub async fn close(
        &self,
        ctx: &Ctx<'js>,
        ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Quick(quick) => {
                if let Some(close) = &quick.close {
                    let value = close.call::<_, Value<'js>>((ctrl,))?;
                    if let Some(promise) = value.as_promise() {
                        promise.clone().into_future::<Value<'js>>().await?;
                    }
                }
            }
            Self::Native(native) => native.close(ctx, ctrl).await?,
        }
        Ok(())
    }

    pub async fn write(
        &self,
        ctx: &Ctx<'js>,
        chunk: Value<'js>,
        ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Quick(quick) => {
                if let Some(write) = &quick.write {
                    write.call::<_, ()>((chunk, ctrl))?;
                }
            }
            Self::Native(native) => native.write(ctx, chunk, ctrl).await?,
        }
        Ok(())
    }
}

#[async_trait(?Send)]
pub trait NativeSink<'js>: Trace<'js> {
    async fn start(
        &self,
        ctx: &Ctx<'js>,
        ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()>;

    async fn write(
        &self,
        ctx: &Ctx<'js>,
        chunk: Value<'js>,
        ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()>;

    async fn close(
        &self,
        ctx: &Ctx<'js>,
        ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()>;

    async fn abort(&self, ctx: &Ctx<'js>, reason: Option<Value<'js>>) -> rquickjs::Result<()>;
}

// pub trait DynNativeSink<'js>: Trace<'js> {
//     fn start(
//         &self,
//         ctx: &Ctx<'js>,
//         ctrl: Class<'js, WritableStreamDefaultController<'js>>,
//     ) -> LocalBoxFuture<'js, rquickjs::Result<()>>;

//     fn write(
//         &self,
//         ctx: &Ctx<'js>,
//         chunk: Value<'js>,
//         ctrl: Class<'js, WritableStreamDefaultController<'js>>,
//     ) -> LocalBoxFuture<'js, rquickjs::Result<()>>;

//     fn close(
//         &self,
//         ctx: &Ctx<'js>,
//         ctrl: Class<'js, WritableStreamDefaultController<'js>>,
//     ) -> LocalBoxFuture<'js, rquickjs::Result<()>>;

//     fn abort(
//         &self,
//         ctx: &Ctx<'js>,
//         reason: Option<Value<'js>>,
//     ) -> LocalBoxFuture<'js, rquickjs::Result<()>>;
// }

#[derive(Debug, Clone, Trace)]
pub struct JsUnderlyingSink<'js> {
    start: Option<Function<'js>>,
    write: Option<Function<'js>>,
    close: Option<Function<'js>>,
    abort: Option<Function<'js>>,
}

impl<'js> FromJs<'js> for JsUnderlyingSink<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(JsUnderlyingSink {
            start: obj.get("start")?,
            write: obj.get("write")?,
            close: obj.get("close")?,
            abort: obj.get("abort")?,
        })
    }
}
