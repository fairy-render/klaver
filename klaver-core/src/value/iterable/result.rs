use rquickjs::{FromJs, IntoJs, Object, atom::PredefinedAtom};

pub enum IteratorResult<T> {
    Value(T),
    Done,
}

impl<'js, T> IntoJs<'js> for IteratorResult<T>
where
    T: IntoJs<'js>,
{
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        match self {
            Self::Value(value) => {
                obj.set(PredefinedAtom::Value, value)?;
            }
            Self::Done => {
                obj.set(PredefinedAtom::Done, true)?;
            }
        }

        Ok(obj.into_value())
    }
}

impl<'js, T> FromJs<'js> for IteratorResult<T>
where
    T: FromJs<'js>,
{
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;
        if obj.get::<_, bool>(PredefinedAtom::Done)? {
            Ok(Self::Done)
        } else {
            let val = obj.get(PredefinedAtom::Value)?;
            let item = T::from_js(ctx, val)?;
            Ok(Self::Value(item))
        }
    }
}
