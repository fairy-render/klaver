use rquickjs::{ArrayBuffer, JsLifetime, class::Trace};

#[derive(Debug, JsLifetime)]
pub struct Blob<'js> {
    buffer: ArrayBuffer<'js>,
}
