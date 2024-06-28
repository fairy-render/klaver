use std::collections::VecDeque;

use extensions::Extensions;
use rquickjs::{class::Trace, Ctx, Value};
use slotmap::SlotMap;
use tokio::sync::oneshot;

use crate::config::Config;

#[derive(Default)]
#[rquickjs::class]
pub struct Base<'js> {
    pub timeouts: SlotMap<slotmap::DefaultKey, oneshot::Sender<()>>,
    pub uncaugth_exceptions: VecDeque<Value<'js>>,
    pub config: Config,
    pub extensions: Extensions,
}

#[rquickjs::methods]
impl<'js> Base<'js> {
    pub fn uncaught(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        if let Some(next) = self.uncaugth_exceptions.pop_front() {
            Err(ctx.throw(next))
        } else {
            Ok(())
        }
    }
}

impl<'js> Trace<'js> for Base<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.uncaugth_exceptions.trace(tracer)
    }
}
