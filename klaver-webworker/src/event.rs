use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt, channel::mpsc, future::LocalBoxFuture};
use klaver_base::{DynEvent, Emitter, Event, EventList, IntoDynEvent, NativeEvent};
use klaver_runner::{Func, Runner, Shutdown, Workers};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Class, Ctx, FromJs, Function, JsLifetime, Module,
    String, Value, class::Trace, prelude::Opt,
};
use rquickjs_util::{RuntimeError, StringRef, Subclass, Val};

#[derive(Debug, Trace, JsLifetime)]
#[rquickjs::class]
pub struct MessageEvent<'js> {
    ty: String<'js>,
    #[qjs(get)]
    data: Option<Value<'js>>,
}

pub struct MessageEventOptions<'js> {
    data: Option<Value<'js>>,
}

impl<'js> FromJs<'js> for MessageEventOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        todo!()
    }
}

#[rquickjs::methods]
impl<'js> MessageEvent<'js> {
    pub fn new(
        ty: String<'js>,
        ops: Opt<MessageEventOptions<'js>>,
    ) -> rquickjs::Result<MessageEvent<'js>> {
        Ok(MessageEvent { data: None, ty })
    }
}

impl<'js> NativeEvent<'js> for MessageEvent<'js> {
    fn ty(
        this: rquickjs::prelude::This<Class<'js, Self>>,
        _ctx: Ctx<'js>,
    ) -> rquickjs::Result<String<'js>> {
        Ok(this.borrow().ty.clone())
    }
}

impl<'js> Subclass<'js, Event<'js>> for MessageEvent<'js> {}

impl<'js> IntoDynEvent<'js> for MessageEvent<'js> {
    fn into_dynevent(self, ctx: &Ctx<'js>) -> rquickjs::Result<klaver_base::DynEvent<'js>> {
        let event = Class::instance(ctx.clone(), self)?.into_value();
        DynEvent::from_js(ctx, event)
    }
}
