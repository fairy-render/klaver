use std::{cell::RefCell, rc::Rc};

use rquickjs::{
    Class, Ctx, IntoJs, JsLifetime, Object, Symbol, Value,
    atom::PredefinedAtom,
    class::{JsClass, Readable, Trace},
    prelude::{Func, Opt, This},
};

use super::IteratorResult;

pub trait NativeIteratorInterface<'js>: Trace<'js> {
    type Item: IntoJs<'js>;

    fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>>;

    fn returns(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()>;
}

trait DynNativeInteratorInterface<'js>: Trace<'js> {
    fn next<'a>(&'a self, ctx: &'a Ctx<'js>) -> rquickjs::Result<Option<Value<'js>>>;
    fn returns<'a>(&'a self, ctx: &'a Ctx<'js>) -> rquickjs::Result<()>;
}

pub struct NativeIterator<'js> {
    inner: Box<dyn DynNativeInteratorInterface<'js> + 'js>,
}

impl<'js> NativeIterator<'js> {
    pub fn new<T>(iterator: T) -> NativeIterator<'js>
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

        NativeIterator {
            inner: Box::new(Impl(iterator)),
        }
    }
}

impl<'js> Trace<'js> for NativeIterator<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.inner.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for NativeIterator<'js> {
    type Changed<'to> = NativeIterator<'to>;
}

impl<'js> JsClass<'js> for NativeIterator<'js> {
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

impl<'js> IntoJs<'js> for NativeIterator<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

pub struct NativeIter<T>(RefCell<T>);

impl<T> NativeIter<T> {
    pub fn new(item: T) -> NativeIter<T> {
        NativeIter(RefCell::new(item))
    }
}

impl<'js, T: Trace<'js>> Trace<'js> for NativeIter<T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.0.borrow().trace(tracer);
    }
}

impl<'js, T: Trace<'js>> NativeIteratorInterface<'js> for NativeIter<T>
where
    T: Iterator,
    T::Item: IntoJs<'js>,
{
    type Item = T::Item;

    fn next(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        Ok(self.0.borrow_mut().next())
    }

    fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}
