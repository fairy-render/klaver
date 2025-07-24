use rquickjs::{IntoJs, Object, atom::PredefinedAtom};

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
