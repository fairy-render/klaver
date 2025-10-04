use std::marker::PhantomData;

use rquickjs::{
    atom::PredefinedAtom,
    class::{JsClass, Trace},
    prelude::{Func, MutFn, This},
    Class, Ctx, FromJs, Function, IntoJs, Object, Value,
};

use crate::{
    util::{is_iterator, ObjectExt},
    Next,
};

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
            .call::<_, Next<T>>((This(self.iter.clone()),))?;

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

pub struct NativeIter<T> {
    i: T,
}

impl<T> NativeIter<T> {
    pub fn new(iter: T) -> NativeIter<T> {
        NativeIter { i: iter }
    }
}

impl<'js, T> IntoJs<'js> for NativeIter<T>
where
    T: Iterator + 'js,
    T::Item: IntoJs<'js>,
{
    fn into_js(mut self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        obj.set(
            PredefinedAtom::Next,
            Func::new(MutFn::new(move || {
                let next = self.i.next();
                rquickjs::Result::Ok(IterResult::new(next))
            })),
        )?;

        obj.set(
            PredefinedAtom::SymbolIterator,
            Func::new(|this: This<Object<'js>>| rquickjs::Result::Ok(this.0)),
        )?;

        Ok(obj.into_value())
    }
}

pub trait Iterable<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    type Item: IntoJs<'js>;
    type Iter: Iterator<Item = rquickjs::Result<Self::Item>>;

    fn entries(&mut self) -> rquickjs::Result<NativeIter<Self::Iter>>;

    fn add_iterable_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx)?.expect("Prototype");

        proto.set(
            PredefinedAtom::SymbolIterator,
            Func::new(Self::return_iterator),
        )?;

        Ok(())
    }

    fn return_iterator(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        this.borrow_mut().entries().into_js(&ctx)
    }
}

#[derive(Debug, Clone)]
pub struct IterResult<'js, T> {
    value: Option<T>,
    life: PhantomData<&'js ()>,
}

impl<'js, T> IterResult<'js, T> {
    pub fn new(value: Option<T>) -> IterResult<'js, T> {
        IterResult {
            value: value,
            life: PhantomData,
        }
    }
}

impl<'js, T: Trace<'js>> Trace<'js> for IterResult<'js, T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.value.trace(tracer)
    }
}

impl<'js, T: IntoJs<'js>> IntoJs<'js> for IterResult<'js, T> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        let done = self.value.is_none();

        obj.set(PredefinedAtom::Value, self.value)?;
        obj.set(PredefinedAtom::Done, done)?;

        Ok(obj.into_value())
    }
}
