use std::fs::FileType;

use futures::TryStreamExt;
use klaver::{throw, throw_if};
use klaver_shared::{buffer::Buffer, iter::AsyncIter, Static};
use rquickjs::{ArrayBuffer, Ctx, IntoJs, Object};

#[rquickjs::function]
pub async fn read_file<'js>(ctx: Ctx<'js>, path: String) -> rquickjs::Result<ArrayBuffer<'js>> {
    let bytes = throw_if!(ctx, tokio::fs::read(path).await);
    ArrayBuffer::new(ctx, bytes)
}

#[rquickjs::function]
pub async fn write_file<'js>(
    ctx: Ctx<'js>,
    path: String,
    content: Buffer<'js>,
) -> rquickjs::Result<()> {
    let Some(content) = content.as_raw() else {
        throw!(ctx, "Buuffer is detached")
    };
    throw_if!(ctx, tokio::fs::write(path, content.slice()).await);

    Ok(())
}

pub struct JsDirEntry {
    path: String,
    ty: String,
}

impl<'js> IntoJs<'js> for JsDirEntry {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        obj.set("path", self.path)?;
        obj.set("type", self.ty)?;

        obj.into_js(ctx)
    }
}

#[rquickjs::function]
pub async fn read_dir<'js>(ctx: Ctx<'js>, path: String) -> rquickjs::Result<rquickjs::Value<'js>> {
    let read_dir = throw_if!(ctx, tokio::fs::read_dir(path).await);

    let stream = tokio_stream::wrappers::ReadDirStream::new(read_dir).and_then(|item| async move {
        Result::<_, std::io::Error>::Ok(JsDirEntry {
            path: item.path().display().to_string(),
            ty: "".to_string(),
        })
    });

    AsyncIter::new(Static(stream)).into_js(&ctx)
}
