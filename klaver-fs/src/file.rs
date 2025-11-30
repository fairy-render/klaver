use klaver_base::Exportable;
use klaver_util::{Buffer, sync::AsyncLock, throw, throw_if};
use rquickjs::{
    ArrayBuffer, Class, Ctx, JsLifetime, String,
    class::{JsClass, Trace},
    prelude::This,
};
use vfs::{SeekFrom, VFileExt, boxed::BoxVFile};

#[rquickjs::class]
pub struct File<'js> {
    pub inner: AsyncLock<BoxVFile>,
    #[qjs(get, rename = "fileName")]
    pub file_name: String<'js>,
    #[qjs(get, rename = "type")]
    pub mime: String<'js>,
}

unsafe impl<'js> JsLifetime<'js> for File<'js> {
    type Changed<'to> = File<'to>;
}

impl<'js> Trace<'js> for File<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.file_name.trace(tracer);
        self.mime.trace(tracer);
    }
}

#[rquickjs::methods]
impl<'js> File<'js> {
    async fn read(
        this: This<Class<'js, File<'js>>>,
        ctx: Ctx<'js>,
        len: usize,
    ) -> rquickjs::Result<ArrayBuffer<'js>> {
        let mut buffer = Vec::with_capacity(len);
        buffer.resize(len, 0);

        let ret = throw_if!(
            ctx,
            this.borrow().inner.write().await.read(&mut buffer).await
        );
        buffer.resize(ret, 0);

        ArrayBuffer::new(ctx, buffer)
    }

    #[qjs(rename = "arrayBuffer")]
    async fn array_buffer(&self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        let mut file = self.inner.write().await;
        throw_if!(ctx, file.seek(SeekFrom::Start(0)).await);

        let mut vec = Vec::new();

        throw_if!(ctx, file.read_to_end(&mut vec).await);

        ArrayBuffer::new(ctx, vec)
    }

    async fn write(
        this: This<Class<'js, File<'js>>>,
        ctx: Ctx<'js>,
        buffer: Buffer<'js>,
    ) -> rquickjs::Result<()> {
        let array = buffer.array_buffer()?;
        let Some(slice) = array.as_bytes() else {
            throw!(ctx, "Buffer is detached")
        };

        throw_if!(
            ctx,
            this.borrow().inner.write().await.write_all(slice).await
        );

        Ok(())
    }
}

impl<'js> Exportable<'js> for File<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        target.set(ctx, File::NAME, Class::<File>::create_constructor(ctx))?;
        Ok(())
    }
}
