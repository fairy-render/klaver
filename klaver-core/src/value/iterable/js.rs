use rquickjs::{FromJs, Function, IntoJs, Object, Value, atom::PredefinedAtom, class::Trace};

use crate::value::{
    extensions::FunctionExt,
    iterable::{IteratorResult, NativeIteratorIter},
};

use super::native::NativeIteratorInterface;

#[derive(Trace)]
pub struct JsIterator<'js> {
    target: Value<'js>,
    next: Function<'js>,
    returns: Option<Function<'js>>,
}

impl<'js> NativeIteratorInterface<'js> for JsIterator<'js> {
    type Item = Value<'js>;

    fn next(&self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let result = self.next.call::<_, IteratorResult<Value<'js>>>(())?;
        match result {
            IteratorResult::Value(value) => Ok(Some(value)),
            IteratorResult::Done => Ok(None),
        }
    }

    fn returns(&self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        if let Some(returns) = &self.returns {
            returns.call::<_, ()>(())?;
        }

        Ok(())
    }
}

impl<'js> FromJs<'js> for JsIterator<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let mut next: Function = obj.get(PredefinedAtom::Next)?;
        let mut returns: Option<Function> = obj.get(PredefinedAtom::Return)?;

        next = next.bind(ctx, (obj.clone(),))?;

        if let Some(ret) = returns {
            returns = Some(ret.bind(ctx, (obj.clone(),))?);
        }

        Ok(JsIterator {
            next,
            returns,
            target: value,
        })
    }
}

impl<'js> IntoIterator for JsIterator<'js> {
    type IntoIter = NativeIteratorIter<'js, Self>;

    type Item = rquickjs::Result<Value<'js>>;

    fn into_iter(self) -> Self::IntoIter {
        NativeIteratorIter::new(self.target.ctx().clone(), self)
    }
}

impl<'js> IntoJs<'js> for JsIterator<'js> {
    fn into_js(self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.target)
    }
}

#[derive(Trace)]
pub struct JsIterable<'js> {
    object: Value<'js>,
    create: Function<'js>,
}

impl<'js> JsIterable<'js> {
    pub fn iterator(&self) -> rquickjs::Result<JsIterator<'js>> {
        Ok(self.create.call(())?)
    }
}

impl<'js> FromJs<'js> for JsIterable<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let create: Function<'_> = obj.get(PredefinedAtom::SymbolIterator)?;

        Ok(JsIterable {
            object: value,
            create: create.bind(ctx, (obj,))?,
        })
    }
}

impl<'js> IntoJs<'js> for JsIterable<'js> {
    fn into_js(self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object)
    }
}
