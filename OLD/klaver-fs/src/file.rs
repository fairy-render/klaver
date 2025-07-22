use rquickjs::{class::Trace, Ctx, IntoJs, Value};
use rquickjs_util::{async_iterator::AsyncIter, buffer::Buffer, throw, throw_if, Static};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
#[derive(Trace)]
pub struct JsFile {
    pub file: Option<Static<tokio::fs::File>>,
}

impl JsFile {
    pub fn new(file: tokio::fs::File) -> JsFile {
        JsFile {
            file: Some(Static(file)),
        }
    }

    fn file(&mut self, ctx: &Ctx<'_>) -> rquickjs::Result<&mut tokio::fs::File> {
        match &mut self.file {
            Some(file) => Ok(&mut file.0),
            None => throw!(ctx, "File closed"),
        }
    }
}

#[rquickjs::methods]
impl JsFile {
    #[qjs(rename = "readLines")]
    pub async fn read_lines<'js>(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let file = throw_if!(ctx, self.file(&ctx)?.try_clone().await);
        let lines = tokio::io::BufReader::new(file).lines();

        let stream = tokio_stream::wrappers::LinesStream::new(lines);

        AsyncIter::new(Static(stream)).into_js(&ctx)
    }

    pub async fn write<'js>(&mut self, ctx: Ctx<'js>, buffer: Buffer<'js>) -> rquickjs::Result<()> {
        let Some(buffer) = buffer.as_raw() else {
            throw!(ctx, "Buffer detached")
        };

        throw_if!(ctx, self.file(&ctx)?.write_all(buffer.slice()).await);

        Ok(())
    }

    pub async fn read<'js>(
        &mut self,
        ctx: Ctx<'js>,
        buffer: Buffer<'js>,
    ) -> rquickjs::Result<usize> {
        let Some(mut buffer) = buffer.as_raw() else {
            throw!(ctx, "Buffer detached")
        };

        let len = throw_if!(
            ctx,
            self.file(&ctx)?.read(unsafe { buffer.slice_mut() }).await
        );

        Ok(len)
    }

    pub async fn flush(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        throw_if!(ctx, self.file(&ctx)?.flush().await);
        Ok(())
    }

    pub async fn close(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        throw_if!(ctx, self.file(&ctx)?.shutdown().await);
        self.file = None;
        Ok(())
    }
}
