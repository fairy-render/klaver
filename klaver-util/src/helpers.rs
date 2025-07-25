use std::pin::Pin;

use futures::Stream;
use rquickjs::{Array, Ctx, FromJs, IntoJs, Type, Value, class::Trace, prelude::Opt};

use crate::array::ArrayExt;

pub struct Pair<K, V>(pub K, pub V);

impl<'js, K, V> FromJs<'js> for Pair<K, V>
where
    K: FromJs<'js>,
    V: FromJs<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let array = Array::from_js(ctx, value)?;

        let key = array.get(0)?;
        let value = array.get(1)?;

        Ok(Pair(key, value))
    }
}

impl<'js, K, V> IntoJs<'js> for Pair<K, V>
where
    K: IntoJs<'js>,
    V: IntoJs<'js>,
{
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let array = Array::new(ctx.clone())?;

        array.push(self.0)?;
        array.push(self.1)?;

        Ok(array.into_value())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Static<T>(pub T);

impl<T> std::ops::Deref for Static<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Static<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'js, T> Trace<'js> for Static<T> {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<T: Iterator> Iterator for Static<T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T: Stream> Stream for Static<T> {
    type Item = T::Item;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) }
            .as_mut()
            .poll_next(cx)
    }
}

pub fn is_plain_object<'js>(
    ctx: &Ctx<'js>,
    obj: &Value<'js>,
    strict: Opt<bool>,
) -> rquickjs::Result<bool> {
    if obj.is_null() || obj.is_undefined() {
        return Ok(false);
    }

    let Some(obj) = obj.as_object() else {
        return Ok(false);
    };

    let strict = strict.unwrap_or(true);

    let object_ctor = ctx.globals().get::<_, Value<'js>>("Object")?;
    let ctor = obj.get::<_, Value<'js>>("constructor")?;

    let is_instance = obj.is_instance_of(&object_ctor);
    let is_typeof = obj.type_of() == Type::Object;
    let is_ctor_undefined = ctor.is_undefined() || ctor.is_null();
    let is_ctor_object = ctor == object_ctor;
    let is_ctor_fn = ctor.type_of() == Type::Function || ctor.type_of() == Type::Constructor;

    Ok(if strict {
        (is_instance || is_typeof) && (is_ctor_undefined || is_ctor_object)
    } else {
        is_ctor_undefined || is_ctor_fn
    })
}
