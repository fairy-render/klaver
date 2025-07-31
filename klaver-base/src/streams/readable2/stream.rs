use rquickjs::{Class, JsLifetime, class::Trace};

use crate::streams::readable2::state::ReadableStreamData;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStream<'js> {
    state: Class<'js, ReadableStreamData<'js>>,
}

impl<'js> ReadableStream<'js> {
    pub fn get_reader(&self) -> rquickjs::Result<()> {
        Ok(())
    }
}
