use rquickjs::{Class, JsLifetime, class::Trace, prelude::Opt};

use crate::streams::{ReadableStream, WritableStream, transform::transformer::Transform};

#[derive(Trace, JsLifetime)]
pub struct TransformStream<'js> {
    pub readable: Class<'js, ReadableStream<'js>>,
    pub writable: Class<'js, WritableStream<'js>>,
}

impl<'js> TransformStream<'js> {
    pub fn new(transform: Opt<Transform<'js>>) {}
}
