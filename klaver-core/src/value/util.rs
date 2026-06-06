#[cfg(feature = "async")]
use futures::Stream;
use rquickjs::{Ctx, Value, class::Trace, function::Opt};

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
    let is_typeof = obj.type_of() == rquickjs::Type::Object;
    let is_ctor_undefined = ctor.is_undefined() || ctor.is_null();
    let is_ctor_object = ctor == object_ctor;
    let is_ctor_fn =
        ctor.type_of() == rquickjs::Type::Function || ctor.type_of() == rquickjs::Type::Constructor;

    Ok(if strict {
        (is_instance || is_typeof) && (is_ctor_undefined || is_ctor_object)
    } else {
        is_ctor_undefined || is_ctor_fn
    })
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

#[cfg(feature = "async")]
impl<T: Stream> Stream for Static<T> {
    type Item = T::Item;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        unsafe { core::pin::Pin::new_unchecked(&mut self.get_unchecked_mut().0) }
            .as_mut()
            .poll_next(cx)
    }
}
