use std::collections::VecDeque;

use extensions::Extensions;
use rquickjs::{class::Trace, Value};
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
impl<'js> Base<'js> {}

impl<'js> Trace<'js> for Base<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.uncaugth_exceptions.trace(tracer)
    }
}
