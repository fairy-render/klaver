use klaver_util::{Buffer, Inheritable, StringRef, SuperClass, throw, throw_if};
use rquickjs::{
    ArrayBuffer, Class, Ctx, FromJs, JsLifetime, Object, String,
    class::{JsClass, Trace},
    prelude::Opt,
};

use crate::{
    Clonable, Registry, SerializationContext, StructuredClone, Tag, TransferData,
    export::{ExportTarget, Exportable},
    register,
    streams::{ReadableStream, readable::One},
};

#[derive(Debug, JsLifetime)]
#[rquickjs::class]
pub struct Blob<'js> {
    pub buffer: ArrayBuffer<'js>,
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
    pub async fn array_buffer(&self, _ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
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
    pub fn extend(&self, _ctx: &Ctx<'js>, output: &mut Vec<u8>) -> rquickjs::Result<()> {
        match self {
            BlobInit::String(s) => output.extend_from_slice(s.as_bytes()),
            BlobInit::Buffer(b) => {
                if let Some(raw) = b.as_raw() {
                    output.extend_from_slice(raw.slice());
                }
            }
            BlobInit::Blob(b) => {
                let blob = b.borrow();
                let Some(bytes) = blob.buffer.as_bytes() else {
                    todo!("Detached buffer")
                };
                output.extend_from_slice(bytes);
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

// Inheritance

impl<'js, T> Inheritable<'js, T> for Blob<'js> where T: JsClass<'js> {}

impl<'js> SuperClass<'js> for Blob<'js> {}

// Structured Cloning;

pub struct BlobCloner;

impl StructuredClone for BlobCloner {
    type Item<'js> = Class<'js, Blob<'js>>;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: crate::TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        match obj {
            TransferData::Bytes(bytes) => {
                let buffer = ArrayBuffer::new(ctx.ctx().clone(), bytes)?;
                let blob = Blob { buffer, ty: None };
                Class::instance(ctx.ctx().clone(), blob)
            }
            _ => {
                throw!(@type ctx, "Expected bytes")
            }
        }
    }

    fn to_transfer_object<'js>(
        _ctx: &mut SerializationContext<'js, '_>,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<crate::TransferData> {
        Ok(TransferData::Bytes(
            value.borrow().buffer.as_slice()?.to_vec(),
        ))
    }
}

impl<'js> Clonable for Blob<'js> {
    type Cloner = BlobCloner;
}

// Export

impl<'js> Exportable<'js> for Blob<'js> {
    fn export<T>(ctx: &Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: ExportTarget<'js>,
    {
        register::<Blob>(ctx, registry)?;
        target.set(ctx, Blob::NAME, Class::<Blob>::create_constructor(ctx)?)?;
        Ok(())
    }
}
