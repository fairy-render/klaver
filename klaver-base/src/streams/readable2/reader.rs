use rquickjs::{Class, JsLifetime, class::Trace};

use super::state::ReadableStreamData;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub struct ReadableStreamDefaultReader<'js> {
    pub data: Option<Class<'js, ReadableStreamData<'js>>>,
}
