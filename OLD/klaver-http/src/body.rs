use klaver_shared::buffer::Buffer;
use rquickjs::{class::Trace, FromJs};

#[derive(Trace)]
pub enum BodyInit<'js> {
    Buffer(Buffer<'js>),
    String(String),
}

impl<'js> BodyInit<'js> {
    pub fn to_vec(self) -> Vec<u8> {
        match self {
            BodyInit::Buffer(buffer) => buffer
                .as_raw()
                .map(|m| m.slice().to_vec())
                .unwrap_or_default(),
            BodyInit::String(str) => str.into_bytes(),
        }
    }
}

impl<'js> FromJs<'js> for BodyInit<'js> {
    fn from_js(
        ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        let body = if Buffer::is(ctx, &value)? {
            BodyInit::Buffer(Buffer::from_js(ctx, value)?)
        } else if value.is_string() {
            let str = value
                .try_into_string()
                .map_err(|_| rquickjs::Error::new_from_js("value", "string"))?;
            BodyInit::String(str.to_string()?)
        } else {
            return Err(rquickjs::Error::new_from_js("value", "string or buffer"));
        };

        Ok(body)
    }
}
