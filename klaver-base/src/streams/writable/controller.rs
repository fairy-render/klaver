use event_listener::Event;
use rquickjs::{Class, Ctx, JsLifetime, String, Value, class::Trace, prelude::Opt};
use rquickjs_util::throw;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::streams::{queue_strategy::QueuingStrategy, writable::state::StreamData};

#[rquickjs::class]
#[derive(Trace)]
pub struct WritableStreamDefaultController<'js> {
    pub data: Class<'js, StreamData<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for WritableStreamDefaultController<'js> {
    type Changed<'to> = WritableStreamDefaultController<'to>;
}

#[rquickjs::methods]
impl<'js> WritableStreamDefaultController<'js> {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'js>) -> rquickjs::Result<Self> {
        throw!(
            ctx,
            "WritableStreamDefaultController cannot be constructed manully"
        )
    }

    fn error(&self, error: Value<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}
