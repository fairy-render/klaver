use klaver_util::{Buffer, throw, throw_if};
use rquickjs::{ArrayBuffer, Class, Ctx, JsLifetime, String, class::Trace, prelude::This};
use vfs::{VFileExt, boxed::BoxVFile};

#[rquickjs::class]
pub struct File<'js> {
    pub inner: BoxVFile,
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

        let ret = throw_if!(ctx, this.borrow_mut().inner.read(&mut buffer).await);
        buffer.resize(ret, 0);

        ArrayBuffer::new(ctx, buffer)
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

        throw_if!(ctx, this.borrow_mut().inner.write_all(slice).await);

        Ok(())
    }
}
