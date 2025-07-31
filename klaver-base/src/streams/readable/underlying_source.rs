use async_trait::async_trait;
use klaver_util::{NativeAsyncIteratorInterface, NativeIteratorInterface};
use rquickjs::{
    CatchResultExt, CaughtError, Class, Ctx, FromJs, Function, IntoJs, Object, Value, class::Trace,
};
use std::{cell::RefCell, rc::Rc};

use super::controller::ReadableStreamDefaultController;

#[async_trait(?Send)]
pub trait NativeSource<'js>: Trace<'js> {
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

/// A Underlying source that wraps a async iterator
pub struct AsyncIteratorSource<T>(pub T);

impl<'js, T> Trace<'js> for AsyncIteratorSource<T>
where
    T: NativeAsyncIteratorInterface<'js>,
{
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.0.trace(tracer)
    }
}

#[async_trait(?Send)]
impl<'js, T> NativeSource<'js> for AsyncIteratorSource<T>
where
    T: NativeAsyncIteratorInterface<'js>,
{
    async fn start(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        if let Some(next) = self.0.next(&ctx).await? {
            ctrl.borrow_mut()
                .enqueue(ctx.clone(), next.into_js(&ctx)?)?;
        } else {
            ctrl.borrow_mut().close(ctx)?;
        }

        Ok(())
    }

    async fn pull(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        if let Some(next) = self.0.next(&ctx).await? {
            ctrl.borrow_mut()
                .enqueue(ctx.clone(), next.into_js(&ctx)?)?;
        } else {
            ctrl.borrow_mut().close(ctx)?;
        }

        Ok(())
    }
}

// Iterator
pub struct IteratorSource<T>(pub T);

impl<'js, T> Trace<'js> for IteratorSource<T>
where
    T: NativeIteratorInterface<'js>,
{
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.0.trace(tracer)
    }
}

#[async_trait(?Send)]
impl<'js, T> NativeSource<'js> for IteratorSource<T>
where
    T: NativeIteratorInterface<'js>,
{
    async fn start(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        if let Some(next) = self.0.next(&ctx)? {
            ctrl.borrow_mut()
                .enqueue(ctx.clone(), next.into_js(&ctx)?)?;
        } else {
            ctrl.borrow_mut().close(ctx)?;
        }

        Ok(())
    }

    async fn pull(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        if let Some(next) = self.0.next(&ctx)? {
            ctrl.borrow_mut()
                .enqueue(ctx.clone(), next.into_js(&ctx)?)?;
        } else {
            ctrl.borrow_mut().close(ctx)?;
        }

        Ok(())
    }
}

pub struct One<T>(Option<T>);

impl<T> One<T> {
    pub fn new(item: T) -> One<T> {
        One(Some(item))
    }
}

impl<'js, T: Trace<'js>> Trace<'js> for One<T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.0.trace(tracer)
    }
}

#[async_trait(?Send)]
impl<'js, T> NativeSource<'js> for One<T>
where
    T: Trace<'js> + IntoJs<'js>,
{
    async fn start(
        &mut self,
        _ctx: Ctx<'js>,
        __ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        Ok(())
    }

    async fn pull(
        &mut self,
        ctx: Ctx<'js>,

        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        if let Some(next) = self.0.take() {
            ctrl.borrow_mut()
                .enqueue(ctx.clone(), next.into_js(&ctx)?)?;
        } else {
            ctrl.borrow_mut().close(ctx)?;
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
            Self::Native(n) => n.borrow().trace(tracer),
        }
    }
}
