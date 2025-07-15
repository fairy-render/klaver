use rquickjs::{Class, Ctx, FromJs, Function, Object, String, Value, class::Trace};

use super::controller::WritableStreamDefaultController;

#[derive(Trace, Debug, Clone)]
pub enum UnderlyingSink<'js> {
    Quick(JsUnderlyingSink<'js>),
}

impl<'js> UnderlyingSink<'js> {
    pub async fn start(
        &self,
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
        }
        Ok(())
    }
    pub async fn abort(&self, reason: Option<String<'js>>) -> rquickjs::Result<()> {
        match self {
            Self::Quick(quick) => {
                if let Some(abort) = &quick.abort {
                    abort.call::<_, ()>((reason,))?;
                }
            }
        }
        Ok(())
    }

    pub async fn close(
        &self,
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
        }
        Ok(())
    }

    pub async fn write(
        &self,
        chunk: Value<'js>,
        ctrl: Class<'js, WritableStreamDefaultController<'js>>,
    ) -> rquickjs::Result<()> {
        match self {
            Self::Quick(quick) => {
                if let Some(write) = &quick.write {
                    write.call::<_, ()>((chunk, ctrl))?;
                }
            }
        }
        Ok(())
    }
}

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
