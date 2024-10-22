use std::sync::Arc;

use rquickjs::{class::Trace, CatchResultExt, Class, Ctx, FromJs, Function, Object};

use super::controller::{ReadableStreamDefaultController, State};

macro_rules! catch {
    ($ctrl: expr, $ctx: expr, $ret: expr) => {
        match $ret.catch(&$ctx) {
            Ok(ret) => ret,
            Err(err) => {
                $ctrl.borrow_mut().state = State::Error(err);
                return Ok(());
            }
        }
    };
    ($ctrl: expr, $ctx: expr, $ret: expr, $return: expr) => {
        match $ret.catch(&$ctx) {
            Ok(ret) => ret,
            Err(err) => {
                $ctrl.borrow_mut().state = State::Error(err);
                return Ok($return);
            }
        }
    };
}

macro_rules! call {
    ($ctrl: expr, $ctx: expr, $func: expr) => {
        catch!(
            $ctrl,
            $ctx,
            $func.call::<_, rquickjs::Value>(($ctrl.clone(),))
        )
    };
    ($ctrl: expr, $ctx: expr, $func: expr, $return: expr) => {
        catch!(
            $ctrl,
            $ctx,
            $func.call::<_, rquickjs::Value>(($ctrl.clone(),)),
            $return
        )
    };
}

pub trait NativeSource {
    fn start(&mut self) -> ();
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
    ) -> rquickjs::Result<()> {
        let Some(start) = &self.start else {
            return Ok(());
        };

        let called = call!(ctrl, ctx, start);

        if let Some(promise) = called.into_promise() {
            catch!(ctrl, ctx, promise.into_future::<()>().await);
        }

        Ok(())
    }

    pub async fn pull(
        &self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<bool> {
        let Some(start) = &self.pull else {
            return Ok(false);
        };

        let called = call!(ctrl, ctx, start, true);

        if let Some(promise) = called.into_promise() {
            catch!(ctrl, ctx, promise.into_future::<()>().await, true);
        }

        Ok(true)
    }

    pub async fn cancel(
        &self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        let Some(start) = &self.cancel else {
            return Ok(());
        };

        let called = call!(ctrl, ctx, start);

        if let Some(promise) = called.into_promise() {
            catch!(ctrl, ctx, promise.into_future::<()>().await);
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
    Native(Arc<dyn NativeSource>),
    Js(JsUnderlyingSource<'js>),
}

impl<'js> UnderlyingSource<'js> {
    pub async fn start(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Js(i) => i.start(ctx, ctrl).await,
            _ => {
                todo!("Native")
            }
        }
    }

    pub async fn pull(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<bool> {
        match self {
            Self::Js(i) => i.pull(ctx, ctrl).await,
            _ => {
                todo!("Native")
            }
        }
    }

    pub async fn cancel(
        &mut self,
        ctx: Ctx<'js>,
        ctrl: Class<'js, ReadableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
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
