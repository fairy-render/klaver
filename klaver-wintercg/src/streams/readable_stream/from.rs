use rquickjs::{
    atom::PredefinedAtom, class::Trace, Class, Ctx, FromJs, Function, Object,
    Promise, Symbol, Value,
};
use rquickjs_util::{
    async_iterator::DynamicStream,
    util::{is_async_iterator, is_iterator, ObjectExt},
    Buffer,
};

use super::{underlying_source::StreamSource, Chunk, ReadableStream};

pub fn from<'js>(
    ctx: Ctx<'js>,
    value: Value<'js>,
) -> rquickjs::Result<Class<'js, ReadableStream<'js>>> {
    let read = if ReadableStream::is(&value) {
        Class::<ReadableStream>::from_js(&ctx, value)?
    } else if is_async_iterator(&ctx, &value) {
        println!("Async!");
        let symbol = Symbol::async_iterator(ctx.clone());
        let iter = Object::from_js(&ctx, value)?
            .get::<_, Function>(symbol)?
            .call::<_, Object>(())?;

        ReadableStream::from_native(ctx, StreamSource(AsyncIter { i: iter }))?
    } else if is_iterator(&value) {
        let iter =
            value.call_property::<_, _, Object>(ctx.clone(), PredefinedAtom::SymbolIterator, ())?;
        // let iter = Object::from_js(&ctx, value.clone())?
        //     .get::<_, Function>(PredefinedAtom::SymbolIterator)?
        //     .call::<_, Object>((This(value),))?;
        ReadableStream::from_native(ctx, StreamSource(Iter { i: iter }))?
    } else if Buffer::is(&ctx, &value)? {
        todo!()
    } else {
        todo!()
    };

    Ok(read)
}

#[derive(Trace)]
struct AsyncIter<'js> {
    i: Object<'js>,
}

impl<'js> AsyncIter<'js> {
    pub async fn pull(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Chunk<'js>> {
        let ret = self
            .i
            .get::<_, Function>(PredefinedAtom::Next)?
            .call::<_, Value>(())?;

        let ret = if ret.is_promise() {
            Promise::from_value(ret)?.into_future::<Chunk>().await?
        } else {
            Chunk::from_js(&ctx, ret)?
        };

        Ok(ret)
    }
}

impl<'js> DynamicStream<'js> for AsyncIter<'js> {
    fn next<'a>(
        &'a mut self,
        ctx: Ctx<'js>,
    ) -> futures::future::LocalBoxFuture<'a, rquickjs::Result<Option<Value<'js>>>>
    where
        'js: 'a,
    {
        Box::pin(async move { Ok(self.pull(&ctx).await?.value) })
    }
}

#[derive(Trace)]
struct Iter<'js> {
    i: Object<'js>,
}

impl<'js> Iter<'js> {
    pub async fn pull(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Chunk<'js>> {
        // let ret = self
        //     .i
        //     .get::<_, Function>(PredefinedAtom::Next)?
        //     .call::<_, Value>(())?;
        let ret = self
            .i
            .call_property::<_, _, Value>(ctx.clone(), PredefinedAtom::Next, ())?;

        let ret = if ret.is_promise() {
            Promise::from_value(ret)?.into_future::<Chunk>().await?
        } else {
            Chunk::from_js(&ctx, ret)?
        };

        Ok(ret)
    }
}

impl<'js> DynamicStream<'js> for Iter<'js> {
    fn next<'a>(
        &'a mut self,
        ctx: Ctx<'js>,
    ) -> futures::future::LocalBoxFuture<'a, rquickjs::Result<Option<Value<'js>>>>
    where
        'js: 'a,
    {
        Box::pin(async move { Ok(self.pull(&ctx).await?.value) })
    }
}
