use rquickjs::{FromJs, Function, Object, Value, atom::PredefinedAtom, class::Trace};

use crate::{async_iteator::native::NativeAsyncIteratorInterface, func::FunctionExt};

#[derive(Trace)]
pub struct AsyncIterator<'js> {
    target: Value<'js>,
    next: Function<'js>,
    returns: Option<Function<'js>>,
}

impl<'js> NativeAsyncIteratorInterface<'js> for AsyncIterator<'js> {
    type Item = Value<'js>;

    async fn next(&self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        self.next.call_async(()).await
    }

    async fn returns(&self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        if let Some(returns) = &self.returns {
            returns.call_async::<_, ()>(()).await?;
        }

        Ok(())
    }
}

impl<'js> FromJs<'js> for AsyncIterator<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let mut next: Function = obj.get(PredefinedAtom::Next)?;
        let mut returns: Option<Function> = obj.get(PredefinedAtom::Return)?;

        next = next.bind(ctx, (obj.clone(),))?;

        if let Some(ret) = returns {
            returns = Some(ret.bind(ctx, (obj.clone(),))?);
        }

        Ok(AsyncIterator {
            next,
            returns,
            target: value,
        })
    }
}

#[derive(Trace)]
pub struct AsyncIteratable<'js> {
    object: Value<'js>,
    create: Function<'js>,
}

impl<'js> AsyncIteratable<'js> {
    pub fn create(&self) -> rquickjs::Result<AsyncIterator<'js>> {
        Ok(self.create.call(())?)
    }
}

impl<'js> FromJs<'js> for AsyncIteratable<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let create: Function<'_> = obj.get(PredefinedAtom::SymbolAsyncIterator)?;

        Ok(AsyncIteratable {
            object: value,
            create: create.bind(ctx, (obj,))?,
        })
    }
}
