use async_trait::async_trait;
use futures::future::{BoxFuture, LocalBoxFuture};
use klaver::shared::iter::DynamicStream;
use rquickjs::{
    class::Trace, CatchResultExt, CaughtError, Class, Ctx, FromJs, Function, Object, Value,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

use super::controller::ReadableStreamDefaultController;

#[async_trait(?Send)]
pub trait NativeSource<'js> {
    async fn start(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()>;

    async fn pull(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()>;
}

pub struct StreamSource<T>(pub T);

#[async_trait(?Send)]
impl<'js, T> NativeSource<'js> for StreamSource<T>
where
    T: DynamicStream<'js>,
{
    async fn start(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        if let Some(next) = self.0.next(ctx).await? {
            ctrl.borrow_mut().enqueue(next)?;
        } else {
            ctrl.borrow_mut().close()?;
        }

        Ok(())
    }

    async fn pull(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        if let Some(next) = self.0.next(ctx).await? {
            ctrl.borrow_mut().enqueue(next)?;
        } else {
            ctrl.borrow_mut().close()?;
        }

        Ok(())
    }
}

#[derive(Debug, Trace, Clone)]
pub struct JsUnderlyingSource<'js> {
    start: Option<Function<'js>>,
    pull: Option<Function<'js>>,
    cancel: Option<Function<'js>>,
}

impl<'js> JsUnderlyingSource<'js> {
    pub async fn start(
        &self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> Result<(), CaughtError<'js>> {
        let Some(start) = &self.start else {
            return Ok(());
        };

        let called = start.call::<_, Value<'js>>((ctrl,)).catch(&ctx)?;

        if let Some(promise) = called.into_promise() {
            promise.into_future::<()>().await.catch(&ctx)?;
        }

        Ok(())
    }

    pub async fn pull(
        &self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> Result<bool, CaughtError<'js>> {
        let Some(pull) = &self.pull else {
            return Ok(false);
        };

        let called = pull.call::<_, Value<'js>>((ctrl,)).catch(&ctx)?;

        if let Some(promise) = called.into_promise() {
            promise.into_future::<()>().await.catch(&ctx)?;
        }

        Ok(true)
    }

    pub async fn cancel(
        &self,
        ctx: Ctx<'js>,
        ctrl: Option<rquickjs::String<'js>>,
    ) -> Result<(), CaughtError<'js>> {
        let Some(cancel) = &self.cancel else {
            return Ok(());
        };

        let called = cancel.call::<_, Value<'js>>((ctrl,)).catch(&ctx)?;

        if let Some(promise) = called.into_promise() {
            promise.into_future::<()>().await.catch(&ctx)?;
        }

        Ok(())
    }
}

impl<'js> FromJs<'js> for JsUnderlyingSource<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(JsUnderlyingSource {
            start: obj.get("start")?,
            pull: obj.get("pull")?,
            cancel: obj.get("cancel")?,
        })
    }
}

#[derive(Clone)]
pub enum UnderlyingSource<'js> {
    Native(Rc<RefCell<dyn NativeSource<'js> + 'js>>),
    Js(JsUnderlyingSource<'js>),
}

impl<'js> UnderlyingSource<'js> {
    pub async fn start(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> Result<(), CaughtError<'js>> {
        match self {
            Self::Js(i) => i.start(ctx, ctrl).await,
            Self::Native(n) => n.borrow_mut().start(ctx.clone(), ctrl).await.catch(&ctx),
        }
    }

    pub async fn pull(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> Result<bool, CaughtError<'js>> {
        match self {
            Self::Js(i) => i.pull(ctx, ctrl).await,
            Self::Native(n) => n
                .borrow_mut()
                .pull(ctx.clone(), ctrl)
                .await
                .catch(&ctx)
                .map(|_| true),
        }
    }

    pub async fn cancel(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Option<rquickjs::String<'js>>,
    ) -> Result<(), CaughtError<'js>> {
        match self {
            Self::Js(i) => i.cancel(ctx, ctrl).await,
            _ => {
                todo!("Native")
            }
        }
    }
}

impl<'js> Trace<'js> for UnderlyingSource<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Js(js) => js.trace(tracer),
            _ => {}
        }
    }
}
