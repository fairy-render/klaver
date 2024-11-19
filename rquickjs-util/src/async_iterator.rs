use std::{cell::RefCell, rc::Rc};

use futures::{future::LocalBoxFuture, Stream, TryStream, TryStreamExt};
use rquickjs::{
    atom::PredefinedAtom,
    class::{JsClass, Trace},
    prelude::{Func, This},
    Class, Ctx, Exception, IntoJs, JsLifetime, Symbol, Value,
};

use crate::iterator::IterResult;

pub struct StreamContainer<T>(pub T);

impl<'js, T: Trace<'js>> Trace<'js> for StreamContainer<T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.0.trace(tracer)
    }
}

impl<'js, T: Trace<'js>> DynamicStream<'js> for StreamContainer<T>
where
    T: TryStream + Unpin,
    T::Error: std::error::Error,
    T::Ok: IntoJs<'js>,
{
    fn next<'a>(
        &'a mut self,
        ctx: Ctx<'js>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Option<Value<'js>>>>
    where
        'js: 'a,
    {
        Box::pin(async move {
            let next = self.0.try_next().await;

            match next {
                Ok(Some(ret)) => {
                    <<T as TryStream>::Ok as IntoJs<'js>>::into_js(ret, &ctx).map(Some)
                }
                Ok(None) => Ok(None),
                Err(err) => Err(ctx.throw(Value::from_exception(Exception::from_message(
                    ctx.clone(),
                    &err.to_string(),
                )?))),
            }
        })
    }
}

pub trait DynamicStream<'js>: Trace<'js> {
    fn next<'a>(
        &'a mut self,
        ctx: Ctx<'js>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<Option<Value<'js>>>>
    where
        'js: 'a;
}

pub struct AsyncIter<T> {
    i: T,
}

impl<'js, T: Trace<'js>> Trace<'js> for AsyncIter<T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.i.trace(tracer)
    }
}

impl<T> AsyncIter<T> {
    pub fn new(stream: T) -> AsyncIter<T> {
        AsyncIter { i: stream }
    }
}

impl<'js, T> IntoJs<'js> for AsyncIter<T>
where
    T: TryStream + Trace<'js> + Unpin + 'js,
    T::Error: std::error::Error,
    T::Ok: IntoJs<'js>,
{
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let obj = Class::instance(
            ctx.clone(),
            AsyncIterator {
                stream: Rc::new(RefCell::new(Box::new(StreamContainer(self.i)))),
            },
        )?;

        let symbol = Symbol::async_iterator(ctx.clone());

        obj.set(
            symbol,
            Func::new(|this: This<Value<'js>>| Result::<_, rquickjs::Error>::Ok(this.0)),
        )?;

        obj.into_js(ctx)
    }
}

#[rquickjs::class]
pub struct AsyncIterator<'js> {
    stream: Rc<RefCell<Box<dyn DynamicStream<'js> + 'js>>>,
}

unsafe impl<'js> JsLifetime<'js> for AsyncIterator<'js> {
    type Changed<'to> = AsyncIterator<'to>;
}

impl<'js> AsyncIterator<'js> {
    pub fn new<T>(stream: T) -> AsyncIterator<'js>
    where
        T: DynamicStream<'js> + 'js,
    {
        AsyncIterator {
            stream: Rc::new(RefCell::new(Box::new(stream))),
        }
    }
}

impl<'js> Trace<'js> for AsyncIterator<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.stream.borrow().trace(tracer)
    }
}

#[rquickjs::methods]
impl<'js> AsyncIterator<'js> {
    #[qjs(rename = PredefinedAtom::Next)]
    pub async fn next(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let next = self.stream.borrow_mut().next(ctx.clone()).await?;
        IterResult::new(next).into_js(&ctx)
    }

    #[qjs(rename = PredefinedAtom::Return)]
    pub async fn returns(&self) -> rquickjs::Result<()> {
        Ok(())
    }
}

pub trait AsyncIterable<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    type Item: IntoJs<'js>;
    type Error: std::error::Error;

    type Stream: Stream<Item = Result<Self::Item, Self::Error>> + Trace<'js> + Unpin + 'js;

    fn stream(&mut self, ctx: &Ctx<'js>) -> rquickjs::Result<AsyncIter<Self::Stream>>;

    fn add_async_iterable_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx)?.expect("Prototype");

        let symbol = Symbol::async_iterator(ctx.clone());

        proto.set(symbol, Func::new(Self::return_iterator))?;

        Ok(())
    }

    fn return_iterator(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        this.borrow_mut().stream(&ctx)?.into_js(&ctx)
    }
}
