use rquickjs::{
    ArrayBuffer, Class, Ctx, FromJs, JsLifetime, Object, String, class::Trace, prelude::Opt,
};
use rquickjs_util::{Buffer, StringRef, throw, throw_if};

use crate::streams::{ReadableStream, readable::One};

#[derive(Debug, JsLifetime)]
#[rquickjs::class]
pub struct Blob<'js> {
    buffer: ArrayBuffer<'js>,
    #[qjs(rename = "type", get)]
    pub ty: Option<String<'js>>,
}

impl<'js> Trace<'js> for Blob<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.buffer.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> Blob<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        inits: Vec<BlobInit<'js>>,
        options: Opt<BlobOptions<'js>>,
    ) -> rquickjs::Result<Blob<'js>> {
        let mut data = Vec::<u8>::new();

        for init in inits {
            init.extend(&ctx, &mut data)?;
        }

        Ok(Blob {
            buffer: ArrayBuffer::new(ctx, data)?,
            ty: options.0.and_then(|m| m.ty),
        })
    }

    #[qjs(rename = "arrayBuffer")]
    pub async fn array_buffer(&self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        Ok(self.buffer.clone())
    }

    pub async fn bytes(&self) -> rquickjs::Result<rquickjs::TypedArray<'js, u8>> {
        rquickjs::TypedArray::from_arraybuffer(self.buffer.clone())
    }

    pub async fn text(&self, ctx: Ctx<'js>) -> rquickjs::Result<std::string::String> {
        let Some(bytes) = self.buffer.as_bytes() else {
            throw!(@type ctx, "Buffer is detached")
        };
        Ok(throw_if!(ctx, str::from_utf8(bytes).map(|m| m.to_string())))
    }

    pub fn stream(&self, ctx: Ctx<'js>) -> rquickjs::Result<ReadableStream<'js>> {
        ReadableStream::from_native(ctx, One::new(Buffer::ArrayBuffer(self.buffer.clone())))
    }

    #[qjs(get, enumerable)]
    pub fn size(&self) -> usize {
        self.buffer.len()
    }
}

pub struct BlobOptions<'js> {
    ty: Option<String<'js>>,
}

impl<'js> FromJs<'js> for BlobOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(BlobOptions {
            ty: obj.get("type")?,
        })
    }
}

pub enum BlobInit<'js> {
    Blob(Class<'js, Blob<'js>>),
    String(StringRef<'js>),
    Buffer(Buffer<'js>),
}

impl<'js> BlobInit<'js> {
    pub fn extend(&self, ctx: &Ctx<'js>, output: &mut Vec<u8>) -> rquickjs::Result<()> {
        match self {
            BlobInit::String(s) => output.extend_from_slice(s.as_bytes()),
            BlobInit::Buffer(b) => {
                if let Some(raw) = b.as_raw() {
                    output.extend_from_slice(raw.slice());
                }
            }
            BlobInit::Blob(b) => {
                todo!()
            }
        };

        Ok(())
    }
}

impl<'js> FromJs<'js> for BlobInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(blob) = Class::<Blob<'js>>::from_js(ctx, value.clone()) {
            Ok(Self::Blob(blob))
        } else if let Ok(buffer) = Buffer::from_js(ctx, value.clone()) {
            Ok(Self::Buffer(buffer))
        } else if let Ok(string) = StringRef::from_js(ctx, value) {
            Ok(Self::String(string))
        } else {
            Err(rquickjs::Error::new_from_js("value", "blobpart"))
        }
    }
}
