use rquickjs::{Ctx, FromJs, Function, Object, class::Trace};

/// The `transformer` argument to the `TransformStream` constructor, per
/// <https://streams.spec.whatwg.org/#dictdef-transformer>. All hooks are optional: with no
/// `transform`, chunks are forwarded to the readable side unchanged (an identity transform).
#[derive(Debug, Default, Clone, Trace)]
pub struct JsTransformer<'js> {
    pub start: Option<Function<'js>>,
    pub transform: Option<Function<'js>>,
    pub flush: Option<Function<'js>>,
}

impl<'js> FromJs<'js> for JsTransformer<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(JsTransformer {
            start: obj.get("start")?,
            transform: obj.get("transform")?,
            flush: obj.get("flush")?,
        })
    }
}
