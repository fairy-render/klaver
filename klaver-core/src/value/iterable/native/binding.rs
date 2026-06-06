use rquickjs::{
    Class, Ctx, IntoJs, JsLifetime, Object, Symbol, Value,
    atom::PredefinedAtom,
    class::{JsClass, Readable, Trace},
    prelude::{Func, Opt, This},
};

use crate::value::iterable::FromNativeIter;

use super::NativeIteratorInterface;

use super::super::result::IteratorResult;

trait DynNativeInteratorInterface<'js>: Trace<'js> {
    fn next<'a>(&'a self, ctx: &'a Ctx<'js>) -> rquickjs::Result<Option<Value<'js>>>;
    fn returns<'a>(&'a self, ctx: &'a Ctx<'js>) -> rquickjs::Result<()>;
}

pub struct JsNativeIterator<'js> {
    inner: Box<dyn DynNativeInteratorInterface<'js> + 'js>,
}

impl<'js> JsNativeIterator<'js> {
    pub fn from_iter<I: Iterator>(iter: I) -> JsNativeIterator<'js>
    where
        I: Trace<'js> + 'js,
        I::Item: IntoJs<'js>,
    {
        let iter = FromNativeIter::new(iter);
        Self::new(iter)
    }

    pub fn new<T>(iterator: T) -> JsNativeIterator<'js>
    where
        T: NativeIteratorInterface<'js> + 'js,
        T::Item: IntoJs<'js>,
    {
        struct Impl<T>(T);

        impl<'js, T: Trace<'js>> Trace<'js> for Impl<T> {
            fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
                self.0.trace(tracer);
            }
        }

        impl<'js, T> DynNativeInteratorInterface<'js> for Impl<T>
        where
            T: NativeIteratorInterface<'js>,
            T::Item: IntoJs<'js>,
        {
            fn next<'a>(&'a self, ctx: &'a Ctx<'js>) -> rquickjs::Result<Option<Value<'js>>> {
                match self.0.next(ctx)? {
                    Some(ret) => Ok(Some(ret.into_js(ctx)?)),
                    None => Ok(None),
                }
            }

            fn returns<'a>(&'a self, ctx: &'a Ctx<'js>) -> rquickjs::Result<()> {
                self.0.returns(ctx)
            }
        }

        JsNativeIterator {
            inner: Box::new(Impl(iterator)),
        }
    }
}

impl<'js> Trace<'js> for JsNativeIterator<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.inner.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for JsNativeIterator<'js> {
    type Changed<'to> = JsNativeIterator<'to>;
}

impl<'js> JsClass<'js> for JsNativeIterator<'js> {
    const NAME: &'static str = "NatveIterator";

    type Mutable = Readable;

    fn constructor(
        _ctx: &Ctx<'js>,
    ) -> rquickjs::Result<Option<rquickjs::function::Constructor<'js>>> {
        Ok(None)
    }

    fn prototype(ctx: &Ctx<'js>) -> rquickjs::Result<Option<rquickjs::Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        proto.set(
            Symbol::iterator(ctx.clone()),
            Func::new(|this: This<Class<'js, Self>>| rquickjs::Result::Ok(this.0)),
        )?;

        proto.set(
            PredefinedAtom::Next,
            Func::new(|ctx: Ctx<'js>, This(this): This<Class<'js, Self>>| {
                //
                match this.borrow().inner.next(&ctx)? {
                    Some(next) => rquickjs::Result::Ok(IteratorResult::Value(next)),
                    None => Ok(IteratorResult::Done),
                }
            }),
        )?;

        proto.set(
            PredefinedAtom::Return,
            Func::new(
                |ctx: Ctx<'js>, This(this): This<Class<'js, Self>>, _: Opt<Value<'js>>| {
                    this.borrow().inner.returns(&ctx)?;
                    rquickjs::Result::Ok(IteratorResult::<Value<'js>>::Done)
                },
            ),
        )?;

        Ok(Some(proto))
    }
}

impl<'js> IntoJs<'js> for JsNativeIterator<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}
