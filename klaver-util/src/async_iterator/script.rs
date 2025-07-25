use rquickjs::{FromJs, Function, IntoJs, Object, Value, atom::PredefinedAtom, class::Trace};

use crate::{async_iterator::native::NativeAsyncIteratorInterface, func::FunctionExt};

#[derive(Trace)]
pub struct AsyncIter<'js> {
    target: Value<'js>,
    next: Function<'js>,
    returns: Option<Function<'js>>,
}

impl<'js> NativeAsyncIteratorInterface<'js> for AsyncIter<'js> {
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

impl<'js> FromJs<'js> for AsyncIter<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let mut next: Function = obj.get(PredefinedAtom::Next)?;
        let mut returns: Option<Function> = obj.get(PredefinedAtom::Return)?;

        next = next.bind(ctx, (obj.clone(),))?;

        if let Some(ret) = returns {
            returns = Some(ret.bind(ctx, (obj.clone(),))?);
        }

        Ok(AsyncIter {
            next,
            returns,
            target: value,
        })
    }
}

impl<'js> IntoJs<'js> for AsyncIter<'js> {
    fn into_js(self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.target)
    }
}

#[derive(Trace)]
pub struct AsyncIterable<'js> {
    object: Value<'js>,
    create: Function<'js>,
}

impl<'js> AsyncIterable<'js> {
    pub fn async_iterator(&self) -> rquickjs::Result<AsyncIter<'js>> {
        Ok(self.create.call(())?)
    }
}

impl<'js> FromJs<'js> for AsyncIterable<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let create: Function<'_> = obj.get(PredefinedAtom::SymbolAsyncIterator)?;

        Ok(AsyncIterable {
            object: value,
            create: create.bind(ctx, (obj,))?,
        })
    }
}
impl<'js> IntoJs<'js> for AsyncIterable<'js> {
    fn into_js(self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object)
    }
}
