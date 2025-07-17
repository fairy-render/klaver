use rquickjs::{Class, Ctx, Function, class::Trace};

use crate::DynEvent;

use super::event::Event;

pub enum Listener<'js> {
    Js(Function<'js>),
    Native(Box<dyn NativeListener<'js> + 'js>),
}

impl<'js> Listener<'js> {
    pub fn call(&self, ctx: Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()> {
        match self {
            Self::Js(js) => js.call((event,)),
            Self::Native(native) => {
                native.on_event(ctx, event)?;
                Ok(())
            }
        }
    }
}

impl<'js> Trace<'js> for Listener<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Js(js) => js.trace(tracer),
            _ => {}
        }
    }
}

impl<'js> PartialEq<Function<'js>> for Listener<'js> {
    fn eq(&self, other: &Function<'js>) -> bool {
        match self {
            Self::Js(js) => js == other,
            _ => false,
        }
    }
}

pub trait NativeListener<'js> {
    fn on_event(&self, ctx: Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()>;
}

impl<'js> NativeListener<'js> for async_channel::Sender<DynEvent<'js>> {
    fn on_event(&self, ctx: Ctx<'js>, event: DynEvent<'js>) -> rquickjs::Result<()> {
        let this = self.clone();
        ctx.spawn(async move {
            this.send(event).await.ok();
        });

        Ok(())
    }
}
