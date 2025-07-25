use rquickjs::{FromJs, Function, IntoJs, Object, Value, atom::PredefinedAtom, class::Trace};

use super::native::NativeIteratorInterface;
use crate::func::FunctionExt;

#[derive(Trace)]
pub struct Iter<'js> {
    target: Value<'js>,
    next: Function<'js>,
    returns: Option<Function<'js>>,
}

impl<'js> NativeIteratorInterface<'js> for Iter<'js> {
    type Item = Value<'js>;

    fn next(&self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        self.next.call(())
    }

    fn returns(&self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        if let Some(returns) = &self.returns {
            returns.call::<_, ()>(())?;
        }

        Ok(())
    }
}

impl<'js> FromJs<'js> for Iter<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let mut next: Function = obj.get(PredefinedAtom::Next)?;
        let mut returns: Option<Function> = obj.get(PredefinedAtom::Return)?;

        next = next.bind(ctx, (obj.clone(),))?;

        if let Some(ret) = returns {
            returns = Some(ret.bind(ctx, (obj.clone(),))?);
        }

        Ok(Iter {
            next,
            returns,
            target: value,
        })
    }
}

impl<'js> IntoJs<'js> for Iter<'js> {
    fn into_js(self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.target)
    }
}

#[derive(Trace)]
pub struct Iterable<'js> {
    object: Value<'js>,
    create: Function<'js>,
}

impl<'js> Iterable<'js> {
    pub fn iterator(&self) -> rquickjs::Result<Iter<'js>> {
        Ok(self.create.call(())?)
    }
}

impl<'js> FromJs<'js> for Iterable<'js> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        let create: Function<'_> = obj.get(PredefinedAtom::SymbolIterator)?;

        Ok(Iterable {
            object: value,
            create: create.bind(ctx, (obj,))?,
        })
    }
}

impl<'js> IntoJs<'js> for Iterable<'js> {
    fn into_js(self, _ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(self.object)
    }
}
