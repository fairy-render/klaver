use std::{cell::RefCell, rc::Rc};

use klaver_task::AsyncState;
use klaver_util::throw;
use rquickjs::{
    Class, Ctx, JsLifetime, Value,
    class::Trace,
    prelude::{Opt, This},
};

use crate::streams::{
    queue_strategy::QueuingStrategy,
    readable::{
        NativeSource,
        reader::ReadableStreamDefaultReader,
        resource::ReadableStreamResource,
        source::{JsUnderlyingSource, UnderlyingSource},
        state::ReadableStreamData,
    },
};

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStream<'js> {
    state: Class<'js, ReadableStreamData<'js>>,
}

impl<'js> ReadableStream<'js> {
    pub fn from_native<S: NativeSource<'js> + 'js>(
        ctx: &Ctx<'js>,
        source: S,
        strategy: Option<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        let strategy = match strategy {
            Some(ret) => ret,
            None => QueuingStrategy::create_default(ctx)?,
        };

        let data = ReadableStreamData::new(strategy);
        let state = Class::instance(ctx.clone(), data)?;

        let resource = ReadableStreamResource {
            data: state.clone(),
            source: UnderlyingSource::Native(Rc::new(RefCell::new(source))),
        };

        AsyncState::push(&ctx, resource)?;

        Ok(ReadableStream { state })
    }
}

#[rquickjs::methods]
impl<'js> ReadableStream<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        source: JsUnderlyingSource<'js>,
        strategy: Opt<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        let strategy = match strategy.0 {
            Some(ret) => ret,
            None => QueuingStrategy::create_default(&ctx)?,
        };

        let data = ReadableStreamData::new(strategy);
        let state = Class::instance(ctx.clone(), data)?;

        let resource = ReadableStreamResource {
            data: state.clone(),
            source: UnderlyingSource::Js(source),
        };

        AsyncState::push(&ctx, resource)?;

        Ok(ReadableStream { state })
    }

    #[qjs(rename = "getReader")]
    pub fn get_reader(&self, ctx: Ctx<'js>) -> rquickjs::Result<ReadableStreamDefaultReader<'js>> {
        if self.state.borrow().is_locked() {
            throw!(@type ctx, "Stream is locked")
        }

        self.state.borrow_mut().locked = true;

        Ok(ReadableStreamDefaultReader {
            data: Some(self.state.clone()),
        })
    }

    pub async fn cancel(
        This(this): This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        reason: Opt<Value<'js>>,
    ) -> rquickjs::Result<()> {
        let reader = Class::instance(ctx.clone(), this.borrow().get_reader(ctx.clone())?)?;

        ReadableStreamDefaultReader::cancel(This(reader), ctx, reason).await?;

        Ok(())
    }
}

create_export!(ReadableStream<'js>);
