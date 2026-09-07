use std::{
    future::Future,
    time::{SystemTime, UNIX_EPOCH},
};

use klaver_core::{
    Inheritable, Subclass, SuperClass, throw, throw_if,
    value::{
        Buffer, StringRef,
        structured_clone::{
            self, Clonable, Registry, SerializationContext, StructuredClone, Tag, TransferData,
        },
    },
};
use rquickjs::{
    ArrayBuffer, Class, Ctx, FromJs, JsLifetime, Object, String, TypedArray, Value,
    class::{JsClass, Trace},
    object::Accessor,
    prelude::{Async, Func, Opt, This},
};

#[cfg(feature = "streams")]
use crate::streams::{QueuingStrategy, ReadableStream, readable::One};

/// Whole milliseconds since the Unix epoch, matching `Date.now()` (used as the default
/// `File.lastModified`, which per spec is an integer millisecond timestamp).
fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

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
}

/// Every `Blob`-family class (`Blob` itself, and `File`) implements this so `Blob`'s behavior -
/// `size`, `type`, `arrayBuffer()`, `bytes()`, `text()`, `stream()`, `slice()` - works correctly
/// regardless of the concrete Rust type behind the JS object.
///
/// This indirection exists because rquickjs classes aren't really JS-prototype-polymorphic at
/// the Rust binding layer: a native method bound via `#[rquickjs::methods] impl Blob` expects
/// `this` to literally *be* a `Class<'js, Blob>`, so calling it on a `Class<'js, File>` (even
/// though `File.prototype`'s prototype chain includes `Blob.prototype`) would fail to unwrap
/// `this`. Instead, each subtype implements [`NativeBlob::blob`] to expose its embedded [`Blob`]
/// data, and `add_blob_prototype`/`add_blob_prototype_to` bind every Blob-family method/accessor
/// generically per concrete subtype (parameterized on `Self`, not hardcoded to `Blob`) - the
/// same trick `events::event::NativeEvent` uses for `Event`/`MessageEvent`.
pub trait NativeBlob<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    /// The shared `Blob` data embedded in this concrete type.
    fn blob(&self) -> &Blob<'js>;

    fn size(this: This<Class<'js, Self>>) -> usize {
        this.borrow().blob().buffer.len()
    }

    fn ty(this: This<Class<'js, Self>>, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        match &this.borrow().blob().ty {
            Some(ty) => Ok(ty.clone()),
            None => String::from_str(ctx, ""),
        }
    }

    fn array_buffer(
        this: This<Class<'js, Self>>,
    ) -> impl Future<Output = rquickjs::Result<ArrayBuffer<'js>>> {
        async move { Ok(this.borrow().blob().buffer.clone()) }
    }

    fn bytes(
        this: This<Class<'js, Self>>,
    ) -> impl Future<Output = rquickjs::Result<TypedArray<'js, u8>>> {
        async move { TypedArray::from_arraybuffer(this.borrow().blob().buffer.clone()) }
    }

    fn text(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<std::string::String>> {
        async move {
            let this = this.borrow();
            let Some(bytes) = this.blob().buffer.as_bytes() else {
                throw!(@type ctx, "Buffer is detached")
            };
            Ok(throw_if!(ctx, str::from_utf8(bytes).map(|m| m.to_string())))
        }
    }

    fn stream(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        strategy: Option<QueuingStrategy<'js>>,
    ) -> rquickjs::Result<ReadableStream<'js>> {
        ReadableStream::from_native(
            &ctx,
            One::new(Buffer::ArrayBuffer(this.borrow().blob().buffer.clone())),
            strategy,
        )
    }

    fn slice(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        start: Opt<i64>,
        end: Opt<i64>,
        content_type: Opt<String<'js>>,
    ) -> rquickjs::Result<Blob<'js>> {
        let this = this.borrow();
        let Some(bytes) = this.blob().buffer.as_bytes() else {
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

    fn add_blob_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx)?.expect("Blob.prototype");
        Self::add_blob_prototype_to(&proto)
    }

    fn add_blob_prototype_to(proto: &Object<'js>) -> rquickjs::Result<()> {
        // No "already installed" guard here: see the equivalent comment on
        // `NativeEvent::add_event_prototype_to` - each subtype needs its own copies. Both
        // accessors must be `.configurable()` for the same reason documented there.
        proto.prop(
            "size",
            Accessor::new_get(Self::size).enumerable().configurable(),
        )?;
        proto.prop(
            "type",
            Accessor::new_get(Self::ty).enumerable().configurable(),
        )?;
        proto.set("arrayBuffer", Func::from(Async(Self::array_buffer)))?;
        proto.set("bytes", Func::from(Async(Self::bytes)))?;
        proto.set("text", Func::from(Async(Self::text)))?;
        proto.set("stream", Func::new(Self::stream))?;
        proto.set("slice", Func::new(Self::slice))?;

        Ok(())
    }
}

impl<'js> NativeBlob<'js> for Blob<'js> {
    fn blob(&self) -> &Blob<'js> {
        self
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
    File(Class<'js, File<'js>>),
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
            BlobInit::File(f) => {
                let file = f.borrow();
                let Some(bytes) = file.base.buffer.as_bytes() else {
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
        if let Ok(file) = Class::<File<'js>>::from_js(ctx, value.clone()) {
            Ok(Self::File(file))
        } else if let Ok(blob) = Class::<Blob<'js>>::from_js(ctx, value.clone()) {
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

impl<'js> SuperClass<'js> for Blob<'js> {}

impl<'js, T> Inheritable<'js, T> for Blob<'js>
where
    T: JsClass<'js> + NativeBlob<'js>,
{
    fn additional_override(_ctx: &Ctx<'js>, proto: &Object<'js>) -> rquickjs::Result<()> {
        T::add_blob_prototype_to(proto)
    }
}

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
        Blob::add_blob_prototype(ctx)?;
        Ok(())
    }
}

// File, per https://w3c.github.io/FileAPI/#file-section - a `Blob` with a `name` and
// `lastModified` timestamp attached.

#[derive(Debug, JsLifetime)]
#[rquickjs::class]
pub struct File<'js> {
    pub base: Blob<'js>,
    pub name: String<'js>,
    pub last_modified: f64,
}

impl<'js> Trace<'js> for File<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.base.trace(tracer);
        self.name.trace(tracer);
    }
}

impl<'js> NativeBlob<'js> for File<'js> {
    fn blob(&self) -> &Blob<'js> {
        &self.base
    }
}

pub struct FileOptions<'js> {
    ty: Option<String<'js>>,
    last_modified: Option<f64>,
}

impl<'js> FromJs<'js> for FileOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(FileOptions {
            ty: obj.get("type")?,
            last_modified: obj.get("lastModified")?,
        })
    }
}

impl<'js> File<'js> {
    /// Builds a `File` natively (e.g. from a parsed `multipart/form-data` field or a
    /// `FormData.append(name, blob, filename)` call), bypassing the JS-facing constructor.
    /// `last_modified` defaults to "now" per the spec's "create a new `File` object" step.
    pub fn new_native(
        buffer: ArrayBuffer<'js>,
        ty: Option<String<'js>>,
        name: String<'js>,
        last_modified: Option<f64>,
    ) -> File<'js> {
        File {
            base: Blob { buffer, ty },
            name,
            last_modified: last_modified.unwrap_or_else(now_ms),
        }
    }
}

#[rquickjs::methods]
impl<'js> File<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        inits: Opt<Vec<BlobInit<'js>>>,
        name: String<'js>,
        options: Opt<FileOptions<'js>>,
    ) -> rquickjs::Result<File<'js>> {
        let mut data = Vec::<u8>::new();

        for init in inits.0.into_iter().flatten() {
            init.extend(&ctx, &mut data)?;
        }

        let options = options.0;

        Ok(File {
            base: Blob {
                buffer: ArrayBuffer::new(ctx, data)?,
                ty: options.as_ref().and_then(|o| o.ty.clone()),
            },
            name,
            last_modified: options
                .and_then(|o| o.last_modified)
                .unwrap_or_else(now_ms),
        })
    }

    #[qjs(get, enumerable)]
    pub fn name(&self) -> String<'js> {
        self.name.clone()
    }

    #[qjs(rename = "lastModified", get, enumerable)]
    pub fn last_modified(&self) -> f64 {
        self.last_modified
    }
}

// Inheritance

impl<'js> SuperClass<'js> for File<'js> {}

impl<'js> Subclass<'js, Blob<'js>> for File<'js> {}

// Export

impl<'js> klaver_core::Exportable<'js> for File<'js> {
    fn export<T>(ctx: &Ctx<'js>, _registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(ctx, File::NAME, Class::<File>::create_constructor(ctx)?)?;
        // Sets `File.prototype`'s `__proto__` to `Blob.prototype` (so `instanceof Blob` holds)
        // and binds the `NativeBlob` methods/accessors onto `File.prototype` itself (see the
        // comment on `NativeBlob` for why the latter is needed too).
        File::inherit(ctx)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klaver_core::value::FunctionExt;
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Function};

    /// Runs `body` as the contents of an `async` function, with global `Blob` and `File`
    /// constructors available. `body` is expected to throw on failure (e.g. via a plain
    /// `if (...) throw ...`).
    fn run(body: &str) {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();
            let ctx = AsyncContext::full(&rt).await.unwrap();

            ctx.async_with(async |ctx| {
                ctx.globals()
                    .set("Blob", Class::<Blob>::create_constructor(&ctx)?)?;
                Blob::add_blob_prototype(&ctx)?;

                ctx.globals()
                    .set("File", Class::<File>::create_constructor(&ctx)?)?;
                File::inherit(&ctx)?;

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

    /// Regression test: `Blob`/`File`'s prototype is cached per `Runtime` (rquickjs shares class
    /// prototype objects across every `Context` built on the same `Runtime`), but two separate
    /// `Context`s each independently run their own global registration, which used to try to
    /// redefine the (already-installed, non-configurable) `size`/`type` accessors a second time
    /// and throw. This is what `klaver_vm::Vm::create_context()` does in practice.
    #[test]
    fn add_blob_prototype_is_idempotent_across_contexts_on_the_same_runtime() {
        futures::executor::block_on(async move {
            let rt = AsyncRuntime::new().unwrap();

            let ctx1 = AsyncContext::full(&rt).await.unwrap();
            ctx1.async_with(async |ctx| {
                ctx.globals()
                    .set("Blob", Class::<Blob>::create_constructor(&ctx)?)?;
                Blob::add_blob_prototype(&ctx)?;
                ctx.globals()
                    .set("File", Class::<File>::create_constructor(&ctx)?)?;
                File::inherit(&ctx)?;
                rquickjs::Result::Ok(())
            })
            .await
            .unwrap();

            // Second `Context` on the *same* `Runtime` - this used to throw.
            let ctx2 = AsyncContext::full(&rt).await.unwrap();
            ctx2.async_with(async |ctx| {
                ctx.globals()
                    .set("Blob", Class::<Blob>::create_constructor(&ctx)?)?;
                Blob::add_blob_prototype(&ctx)?;
                ctx.globals()
                    .set("File", Class::<File>::create_constructor(&ctx)?)?;
                File::inherit(&ctx)?;

                let test_fn: Function = ctx.eval(
                    r#"(async () => {
                        const blob = new Blob(["hi"], { type: "text/plain" });
                        if (blob.size !== 2) throw new Error(`size was ${blob.size}`);
                        if (blob.type !== "text/plain") throw new Error(`type was ${blob.type}`);
                    })"#,
                )?;

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
    fn blob_part_can_be_an_array_buffer() {
        run(r#"
            const buf = new Uint8Array([104, 105]).buffer; // "hi"
            const blob = new Blob([buf]);
            const text = await blob.text();
            if (text !== "hi") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn file_is_a_blob_with_name_and_last_modified() {
        run(r#"
            const file = new File(["hello"], "greeting.txt", { type: "text/plain", lastModified: 123 });

            if (!(file instanceof Blob)) throw new Error("expected File to be a Blob");
            if (!(file instanceof File)) throw new Error("expected File to be a File");

            if (file.name !== "greeting.txt") throw new Error(`name was ${file.name}`);
            if (file.lastModified !== 123) throw new Error(`lastModified was ${file.lastModified}`);
            if (file.size !== 5) throw new Error(`size was ${file.size}`);
            if (file.type !== "text/plain") throw new Error(`type was ${file.type}`);

            const text = await file.text();
            if (text !== "hello") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn file_defaults_last_modified_to_now() {
        run(r#"
            const before = Date.now();
            const file = new File(["hi"], "a.txt");
            const after = Date.now();

            if (file.lastModified < before || file.lastModified > after) {
                throw new Error(`lastModified ${file.lastModified} not within [${before}, ${after}]`);
            }
        "#);
    }

    #[test]
    fn file_part_can_be_another_file() {
        run(r#"
            const a = new File(["foo"], "a.txt");
            const b = new Blob([a, "bar"]);
            const text = await b.text();
            if (text !== "foobar") throw new Error(`text was ${text}`);
        "#);
    }

    #[test]
    fn blob_slice_of_a_file_returns_a_plain_blob() {
        run(r#"
            const file = new File(["hello world"], "a.txt");
            const sliced = file.slice(0, 5);
            if (sliced instanceof File) throw new Error("slice() should not return a File");
            if (!(sliced instanceof Blob)) throw new Error("slice() should return a Blob");

            const text = await sliced.text();
            if (text !== "hello") throw new Error(`text was ${text}`);
        "#);
    }
}
