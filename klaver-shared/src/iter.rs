use std::marker::PhantomData;

use futures::{future::BoxFuture, Stream, StreamExt, TryStream, TryStreamExt};
use rquickjs::{
    atom::PredefinedAtom,
    class::{JsClass, Trace},
    function::{Async, Func, MutFn, This},
    Class, Ctx, Exception, IntoJs, Object, Value,
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

impl<T> Iter<T> {}

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

        Ok(obj.into_value())
    }
}

pub trait Iterable<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    type Item: IntoJs<'js>;
    type Iter: Iterator<Item = Self::Item>;
    fn entries(&self) -> Iter<Self::Iter>;

    fn add_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
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
        this.borrow().entries().into_js(&ctx)
    }
}

struct StreamContainer<T>(T);

impl<T> StreamContainer<T> {
    async fn next<'js>(&mut self, ctx: Ctx<'js>) -> Option<rquickjs::Result<Value<'js>>> {
        todo!()
    }
}

impl<T> DynamicStream for StreamContainer<T>
where
    T: TryStream + Unpin + Send,
    T::Error: std::error::Error,
    for<'js> T::Ok: IntoJs<'js>,
{
    fn next<'a, 'js: 'a>(
        &'a mut self,
        ctx: Ctx<'js>,
    ) -> BoxFuture<'a, rquickjs::Result<Option<Value<'js>>>> {
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

trait DynamicStream {
    fn next<'a, 'js: 'a>(
        &'a mut self,
        ctx: Ctx<'js>,
    ) -> BoxFuture<'a, rquickjs::Result<Option<Value<'js>>>>;
}

pub struct AsyncIter<T> {
    i: T,
}

impl<T> AsyncIter<T> {}

impl<'js, T> IntoJs<'js> for AsyncIter<T>
where
    T: TryStream + Send + Unpin + 'static,
    T::Error: std::error::Error,
    for<'a> T::Ok: IntoJs<'a>,
{
    fn into_js(mut self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(Class::instance(
            ctx.clone(),
            AsyncIterator {
                stream: Box::new(StreamContainer(self.i)),
            },
        )?
        .into_value())
    }
}

#[rquickjs::class]
struct AsyncIterator {
    stream: Box<dyn DynamicStream>,
}

impl<'js> Trace<'js> for AsyncIterator {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl AsyncIterator {
    #[qjs(rename = PredefinedAtom::Next)]
    pub async fn next<'js>(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let next = self.stream.next(ctx.clone()).await?;
        IterResult::new(next).into_js(&ctx)
    }
}
