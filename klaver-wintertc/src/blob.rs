use klaver_core::{
    Inheritable, SuperClass, throw, throw_if,
    value::{
        Buffer, StringRef,
        structured_clone::{
            self, Clonable, Registry, SerializationContext, StructuredClone, Tag, TransferData,
        },
    },
};
use rquickjs::{
    ArrayBuffer, Class, Ctx, FromJs, JsLifetime, Object, String,
    class::{JsClass, Trace},
    prelude::Opt,
};

#[cfg(feature = "streams")]
use crate::streams::{QueuingStrategy, ReadableStream, readable::One};

#[derive(Debug, JsLifetime)]
#[rquickjs::class]
pub struct Blob<'js> {
    pub buffer: ArrayBuffer<'js>,
    pub ty: Option<String<'js>>,
}

impl<'js> Trace<'js> for Blob<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.buffer.trace(tracer);
        self.ty.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> Blob<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        inits: Opt<Vec<BlobInit<'js>>>,
        options: Opt<BlobOptions<'js>>,
    ) -> rquickjs::Result<Blob<'js>> {
        let mut data = Vec::<u8>::new();

        for init in inits.0.into_iter().flatten() {
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

    pub fn stream(
        &self,
        ctx: Ctx<'js>,
        strategy: Option<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        ReadableStream::from_native(
            &ctx,
            One::new(Buffer::ArrayBuffer(self.buffer.clone())),
            strategy,
        )
    }

    #[qjs(get, enumerable)]
    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    #[qjs(rename = "type", get, enumerable)]
    pub fn ty(&self, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        match &self.ty {
            Some(ty) => Ok(ty.clone()),
            None => String::from_str(ctx, ""),
        }
    }

    pub fn slice(
        &self,
        ctx: Ctx<'js>,
        start: Opt<i64>,
        end: Opt<i64>,
        content_type: Opt<String<'js>>,
    ) -> rquickjs::Result<Blob<'js>> {
        let Some(bytes) = self.buffer.as_bytes() else {
            throw!(@type ctx, "Buffer is detached")
        };

        let size = bytes.len() as i64;

        let clamp = |idx: i64| -> usize {
            let idx = if idx < 0 {
                (size + idx).max(0)
            } else {
                idx.min(size)
            };
            idx as usize
        };

        let start = start.0.map(clamp).unwrap_or(0);
        let end = end.0.map(clamp).unwrap_or(size as usize).max(start);

        let slice = bytes[start..end].to_vec();

        Ok(Blob {
            buffer: ArrayBuffer::new(ctx, slice)?,
            ty: content_type.0,
        })
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
        obj: TransferData,
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
    ) -> rquickjs::Result<TransferData> {
        Ok(TransferData::Bytes(
            value.borrow().buffer.as_slice()?.to_vec(),
        ))
    }
}

impl<'js> Clonable for Blob<'js> {
    type Cloner = BlobCloner;
}

// Export

impl<'js> klaver_core::Exportable<'js> for Blob<'js> {
    fn export<T>(ctx: &Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        structured_clone::register::<Blob>(ctx, registry)?;
        target.set(ctx, Blob::NAME, Class::<Blob>::create_constructor(ctx)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klaver_core::value::FunctionExt;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function};

    /// Runs `body` as the contents of an `async` function, with a global `Blob` constructor
    /// available. `body` is expected to throw on failure (e.g. via a plain `if (...) throw ...`).
    fn run(body: &str) {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                ctx.globals()
                    .set("Blob", Class::<Blob>::create_constructor(&ctx)?)?;

                let test_fn: Function = ctx.eval(format!("(async () => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call_async::<_, ()>(()).await.catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();
        });
    }

    #[test]
    fn size_and_type() {
        run(r#"
            const blob = new Blob(["hello ", "world"], { type: "text/plain" });
            if (blob.size !== 11) throw new Error(`size was ${blob.size}`);
            if (blob.type !== "text/plain") throw new Error(`type was ${blob.type}`);
        "#);
    }

    #[test]
    fn default_type_is_empty_string() {
        run(r#"
            const blob = new Blob(["hi"]);
            if (blob.type !== "") throw new Error(`type was ${JSON.stringify(blob.type)}`);
        "#);
    }

    #[test]
    fn empty_blob() {
        run(r#"
            const blob = new Blob();
            if (blob.size !== 0) throw new Error(`size was ${blob.size}`);
        "#);
    }

    #[test]
    fn text() {
        run(r#"
            const blob = new Blob(["hello ", "world"]);
            const text = await blob.text();
            if (text !== "hello world") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn array_buffer_and_bytes() {
        run(r#"
            const blob = new Blob(["abc"]);

            const buf = await blob.arrayBuffer();
            if (buf.byteLength !== 3) throw new Error(`byteLength was ${buf.byteLength}`);

            const bytes = await blob.bytes();
            if (bytes.length !== 3) throw new Error(`bytes length was ${bytes.length}`);
            if (bytes[0] !== 97 || bytes[1] !== 98 || bytes[2] !== 99) {
                throw new Error(`bytes were ${bytes}`);
            }
        "#);
    }

    #[test]
    fn slice_with_positive_indices() {
        run(r#"
            const blob = new Blob(["hello world"]);
            const sliced = blob.slice(0, 5, "text/plain");
            if (sliced.size !== 5) throw new Error(`size was ${sliced.size}`);
            if (sliced.type !== "text/plain") throw new Error(`type was ${sliced.type}`);

            const text = await sliced.text();
            if (text !== "hello") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn slice_with_negative_indices() {
        run(r#"
            const blob = new Blob(["hello world"]);
            const text = await blob.slice(-5).text();
            if (text !== "world") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn slice_clamps_out_of_range_indices() {
        run(r#"
            const blob = new Blob(["hello"]);
            const text = await blob.slice(-100, 100).text();
            if (text !== "hello") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn blob_part_can_be_another_blob() {
        run(r#"
            const a = new Blob(["foo"]);
            const b = new Blob([a, "bar"]);
            const text = await b.text();
            if (text !== "foobar") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn blob_part_can_be_an_array_buffer() {
        run(r#"
            const buf = new Uint8Array([104, 105]).buffer; // "hi"
            const blob = new Blob([buf]);
            const text = await blob.text();
            if (text !== "hi") throw new Error(`text was ${text}`);
        "#);
    }
}
