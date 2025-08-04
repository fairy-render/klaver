use crate::{
    blob::{Blob, BlobInit},
    streams::ReadableStream,
};
use rquickjs::{
    ArrayBuffer, Ctx, JsLifetime, String,
    class::{Class, Trace},
    prelude::Opt,
};

#[rquickjs::class]
#[derive(Trace)]
pub struct File<'js> {
    blob: Blob<'js>,
    #[qjs(get)]
    file_name: String<'js>,
}

unsafe impl<'js> JsLifetime<'js> for File<'js> {
    type Changed<'to> = File<'to>;
}

impl<'js> File<'js> {
    pub fn init(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let file_prototype = Class::<File>::prototype(ctx)?;
        let blob_prototype = Class::<Blob>::prototype(ctx)?;

        let Some(file_proto) = file_prototype else {
            return Ok(());
        };

        file_proto.set_prototype(blob_prototype.as_ref())?;

        Ok(())
    }
}

#[rquickjs::methods]
impl<'js> File<'js> {
    #[qjs(constructor)]
    fn new(
        ctx: Ctx<'js>,
        init: Vec<BlobInit<'js>>,
        file_name: String<'js>,
    ) -> rquickjs::Result<File<'js>> {
        let blob = Blob::new(ctx, init, Opt(None))?;

        Ok(File { blob, file_name })
    }

    #[qjs(rename = "arrayBuffer")]
    async fn array_buffer(&self, ctx: Ctx<'js>) -> rquickjs::Result<ArrayBuffer<'js>> {
        self.blob.array_buffer(ctx).await
    }

    async fn bytes(&self) -> rquickjs::Result<rquickjs::TypedArray<'js, u8>> {
        self.blob.bytes().await
    }

    pub async fn text(&self, ctx: Ctx<'js>) -> rquickjs::Result<std::string::String> {
        self.blob.text(ctx).await
    }

    #[qjs(get, enumerable)]
    pub fn size(&self) -> usize {
        self.blob.size()
    }

    pub fn stream(&self, ctx: Ctx<'js>) -> rquickjs::Result<ReadableStream<'js>> {
        self.blob.stream(ctx, None)
    }
}
