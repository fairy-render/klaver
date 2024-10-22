use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use futures::{future::LocalBoxFuture, Stream, StreamExt, TryStream, TryStreamExt};
use rquickjs::{
    atom::PredefinedAtom,
    class::{JsClass, Trace},
    function::{Async, Func, MutFn, This},
    Class, Ctx, Exception, IntoJs, Object, Symbol, Value,
};

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

pub struct Iter<T> {
    i: T,
}

impl<T> Iter<T> {
    pub fn new(iter: T) -> Iter<T> {
        Iter { i: iter }
    }
}

impl<'js, T> IntoJs<'js> for Iter<T>
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
    type Iter: Iterator<Item = Self::Item>;
    fn entries(&mut self) -> Iter<Self::Iter>;

    fn add_iterable_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx.clone()).unwrap();

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
        let proto = Class::<Self>::prototype(ctx.clone()).unwrap();

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
