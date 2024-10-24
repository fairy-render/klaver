use std::marker::PhantomData;

use rquickjs::{
    atom::PredefinedAtom, class::Trace, prelude::This, Ctx, FromJs, Function, Object, Value,
};

use crate::util::{is_iterator, ObjectExt};

pub struct JsIterator<'js, T> {
    iter: Object<'js>,
    ty: PhantomData<T>,
}

impl<'js, T> JsIterator<'js, T>
where
    T: FromJs<'js>,
{
    pub fn next(&self) -> rquickjs::Result<Option<T>> {
        let chunk = self
            .iter
            .get::<_, Function>(PredefinedAtom::Next)?
            .call::<_, JsIterChunk<T>>((This(self.iter.clone()),))?;

        Ok(chunk.value)
    }
}

impl<'js, T> Iterator for JsIterator<'js, T>
where
    T: FromJs<'js>,
{
    type Item = rquickjs::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Self::next(self).transpose()
    }
}

impl<'js, T> FromJs<'js> for JsIterator<'js, T> {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = if is_iterator(&value) {
            let obj = value.call_property::<_, _, Object>(
                ctx.clone(),
                PredefinedAtom::SymbolIterator,
                (),
            )?;
            obj
        } else if let Ok(object) = Object::from_js(ctx, value.clone()) {
            object
        } else {
            return Err(rquickjs::Error::new_from_js(value.type_name(), "iterator"));
        };

        if obj.get::<_, Function>(PredefinedAtom::Next).is_err() {
            return Err(rquickjs::Error::new_from_js_message(
                "object",
                "iterator",
                "Missing next function",
            ));
        }

        Ok(JsIterator {
            iter: obj,
            ty: PhantomData,
        })
    }
}

pub struct JsIterChunk<T> {
    pub value: Option<T>,
    pub done: bool,
}

impl<'js, T: Trace<'js>> Trace<'js> for JsIterChunk<T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.value.trace(tracer)
    }
}

impl<'js, T> FromJs<'js> for JsIterChunk<T>
where
    T: FromJs<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_value(value)?;

        Ok(JsIterChunk {
            value: obj.get(PredefinedAtom::Value)?,
            done: obj.get(PredefinedAtom::Done)?,
        })
    }
}
