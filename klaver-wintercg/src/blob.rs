use klaver::{shared::Buffer, throw_if};
use rquickjs::{class::Trace, ArrayBuffer, Class, Ctx, FromJs};

pub enum BlobInit<'js> {
    String(String),
    Buffer(Buffer<'js>),
    Blob(Class<'js, Blob<'js>>),
}

impl<'js> BlobInit<'js> {
    pub fn byte_len(&self) -> usize {
        match self {
            Self::String(bs) => bs.len(),
            Self::Buffer(bs) => bs.len(),
            Self::Blob(bs) => bs.borrow().size(),
        }
    }

    pub fn as_array_buffer(&self, ctx: &Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        let bytes = match self {
            BlobInit::String(s) => ArrayBuffer::new_copy(ctx.clone(), s.as_bytes())?,
            BlobInit::Buffer(b) => b.array_buffer()?,
            BlobInit::Blob(b) => b.borrow().as_array_buffer(ctx)?,
        };

        Ok(bytes)
    }

    pub fn extend(&self, ctx: &Ctx<'js>, output: &mut Vec<u8>) -> rquickjs::Result<()> {
        match self {
            BlobInit::String(s) => output.extend_from_slice(s.as_bytes()),
            BlobInit::Buffer(b) => {
                if let Some(raw) = b.as_raw() {
                    output.extend_from_slice(raw.slice());
                }
            }
            BlobInit::Blob(b) => {
                b.borrow_mut().extend(ctx, output)?;
            }
        };

        Ok(())
    }
}

impl<'js> Trace<'js> for BlobInit<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::Blob(b) => b.trace(tracer),
            Self::Buffer(b) => b.trace(tracer),
            _ => {}
        }
    }
}

impl<'js> FromJs<'js> for BlobInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(blob) = Class::<Blob<'js>>::from_js(ctx, value.clone()) {
            Ok(Self::Blob(blob))
        } else if let Ok(buffer) = Buffer::from_js(ctx, value.clone()) {
            Ok(Self::Buffer(buffer))
        } else if let Ok(string) = String::from_js(ctx, value) {
            Ok(Self::String(string))
        } else {
            Err(rquickjs::Error::new_from_js("value", "blobpart"))
        }
    }
}

// TODO: Implement type
#[derive(Trace)]
#[rquickjs::class]
pub struct Blob<'js> {
    i: Vec<BlobInit<'js>>,
}

impl<'js> Blob<'js> {
    fn as_array_buffer(&self, ctx: &Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        let mut output: Vec<u8> = Vec::default();
        self.extend(ctx, &mut output)?;
        ArrayBuffer::new(ctx.clone(), output)
    }

    fn extend(&self, ctx: &Ctx<'js>, bytes: &mut Vec<u8>) -> rquickjs::Result<()> {
        for item in &self.i {
            item.extend(ctx, bytes)?;
        }
        Ok(())
    }

    fn to_bytes(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Vec<u8>> {
        let mut output = Vec::with_capacity(self.size());
        self.extend(ctx, &mut output)?;
        Ok(output)
    }
}

// TODO: Implement stream and slice
#[rquickjs::methods]
impl<'js> Blob<'js> {
    // TODO: Handle all iterables
    #[qjs(constructor)]
    pub fn new(init: Vec<BlobInit<'js>>) -> rquickjs::Result<Blob<'js>> {
        Ok(Blob { i: init })
    }

    #[qjs(rename = "arrayBuffer")]
    pub async fn array_buffer(&self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        self.as_array_buffer(&ctx)
    }

    pub async fn bytes(&self, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::TypedArray<'js, u8>> {
        let array = self.to_bytes(&ctx)?;
        rquickjs::TypedArray::new(ctx, array)
    }

    pub async fn text(&self, ctx: Ctx<'js>) -> rquickjs::Result<String> {
        let array = self.to_bytes(&ctx)?;
        Ok(throw_if!(ctx, String::from_utf8(array)))
    }

    #[qjs(get, enumerable)]
    pub fn size(&self) -> usize {
        self.i.iter().fold(0, |p, c| p + c.byte_len())
    }
}
