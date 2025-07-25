use futures::future::LocalBoxFuture;
use rquickjs::{
    Class, Ctx, IntoJs, JsLifetime, Object, Symbol, Value,
    atom::PredefinedAtom,
    class::{JsClass, Readable, Trace},
    prelude::{Async, Func, Opt, This},
};

use crate::iterator::IteratorResult;

pub trait NativeAsyncIteratorInterface<'js>: Trace<'js> {
    type Item: IntoJs<'js>;

    fn next(&self, ctx: &Ctx<'js>) -> impl Future<Output = rquickjs::Result<Option<Self::Item>>>;

    fn returns(&self, ctx: &Ctx<'js>) -> impl Future<Output = rquickjs::Result<()>>;
}

trait DynNativeAsyncInteratorInterface<'js>: Trace<'js> {
    fn next<'a>(
        &'a self,
        ctx: &'a Ctx<'js>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Option<Value<'js>>>>;
    fn returns<'a>(&'a self, ctx: &'a Ctx<'js>) -> LocalBoxFuture<'a, rquickjs::Result<()>>;
}

pub struct NativeAsyncIterator<'js> {
    inner: Box<dyn DynNativeAsyncInteratorInterface<'js> + 'js>,
}

impl<'js> NativeAsyncIterator<'js> {
    pub fn new<T>(iterator: T) -> NativeAsyncIterator<'js>
    where
        T: NativeAsyncIteratorInterface<'js> + 'js,
        T::Item: IntoJs<'js>,
    {
        struct Impl<T>(T);

        impl<'js, T: Trace<'js>> Trace<'js> for Impl<T> {
            fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
                self.0.trace(tracer);
            }
        }

        impl<'js, T> DynNativeAsyncInteratorInterface<'js> for Impl<T>
        where
            T: NativeAsyncIteratorInterface<'js>,
            T::Item: IntoJs<'js>,
        {
            fn next<'a>(
                &'a self,
                ctx: &'a Ctx<'js>,
            ) -> LocalBoxFuture<'a, rquickjs::Result<Option<Value<'js>>>> {
                Box::pin(async move {
                    match self.0.next(ctx).await? {
                        Some(ret) => Ok(Some(ret.into_js(ctx)?)),
                        None => Ok(None),
                    }
                })
            }

            fn returns<'a>(
                &'a self,
                ctx: &'a Ctx<'js>,
            ) -> LocalBoxFuture<'a, rquickjs::Result<()>> {
                Box::pin(async move { self.0.returns(ctx).await })
            }
        }

        NativeAsyncIterator {
            inner: Box::new(Impl(iterator)),
        }
    }
}

impl<'js> Trace<'js> for NativeAsyncIterator<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.inner.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for NativeAsyncIterator<'js> {
    type Changed<'to> = NativeAsyncIterator<'to>;
}

impl<'js> JsClass<'js> for NativeAsyncIterator<'js> {
    const NAME: &'static str = "NatveAsyncIterator";

    type Mutable = Readable;

    fn constructor(
        _ctx: &Ctx<'js>,
    ) -> rquickjs::Result<Option<rquickjs::function::Constructor<'js>>> {
        Ok(None)
    }

    fn prototype(ctx: &Ctx<'js>) -> rquickjs::Result<Option<rquickjs::Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        proto.set(
            Symbol::async_iterator(ctx.clone()),
            Func::new(|this: This<Class<'js, Self>>| rquickjs::Result::Ok(this.0)),
        )?;

        proto.set(
            PredefinedAtom::Next,
            Func::new(Async(
                |ctx: Ctx<'js>, This(this): This<Class<'js, Self>>| async move {
                    //
                    match this.borrow().inner.next(&ctx).await? {
                        Some(next) => rquickjs::Result::Ok(IteratorResult::Value(next)),
                        None => Ok(IteratorResult::Done),
                    }
                },
            )),
        )?;

        proto.set(
            PredefinedAtom::Return,
            Func::new(Async(
                |ctx: Ctx<'js>, This(this): This<Class<'js, Self>>, _: Opt<Value<'js>>| async move {
                    this.borrow().inner.returns(&ctx).await?;
                    rquickjs::Result::Ok(IteratorResult::<Value<'js>>::Done)
                },
            )),
        )?;

        Ok(Some(proto))
    }
}
