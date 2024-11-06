use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use rquickjs::{
    class::Trace, function::Opt, module::ModuleDef, ArrayBuffer, Ctx, FromJs, IntoJs, Object,
};
use rquickjs_modules::module_info;
use rquickjs_util::{
    async_iterator::{AsyncIter, AsyncIterable},
    buffer::Buffer,
    throw, throw_if, Static,
};
use tokio::fs::OpenOptions;

use crate::file::JsFile;

// pub type Module = js_fs;

module_info!("@klaver/fs" @types: include_str!("../module.d.ts") => Module);

pub struct JsDirEntry {
    path: String,
    ty: &'static str,
}

impl<'js> IntoJs<'js> for JsDirEntry {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        obj.set("path", self.path)?;
        obj.set("type", self.ty)?;

        obj.into_js(ctx)
    }
}

pub struct Module;

impl ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("read")?;
        decl.declare("write")?;
        decl.declare("resolve")?;
        decl.declare("readDir")?;
        decl.declare("open")?;

        Ok(())
    }

    fn evaluate<'js>(
        _ctx: &Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export("read", js_read)?;
        exports.export("write", js_write)?;
        exports.export("resolve", js_resolve)?;
        exports.export("readDir", js_read_dir)?;
        exports.export("open", js_open_file)?;

        Ok(())
    }
}

pub struct OpenFlags(tokio::fs::OpenOptions);

impl<'js> FromJs<'js> for OpenFlags {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let string = String::from_js(ctx, value)?;

        let mut options = OpenOptions::new();

        for char in string.chars() {
            match char {
                'r' => options.read(true),
                'w' => options.write(true),
                't' => options.truncate(true),
                'a' => options.append(true),
                'c' => options.create_new(true),
                _ => {
                    continue;
                }
            };
        }

        Ok(OpenFlags(options))
    }
}

#[rquickjs::function(rename = "open")]
pub async fn open_file<'js>(
    ctx: Ctx<'js>,
    path: String,
    flag: Opt<OpenFlags>,
) -> rquickjs::Result<JsFile> {
    let file = throw_if!(
        ctx,
        flag.0
            .map(|m| m.0)
            .unwrap_or_else(|| {
                let mut opts = tokio::fs::OpenOptions::new();
                opts.read(true);
                opts
            })
            .open(path)
            .await
    );

    Ok(JsFile::new(file))
}

#[rquickjs::function]
pub async fn read<'js>(ctx: Ctx<'js>, path: String) -> rquickjs::Result<ArrayBuffer<'js>> {
    let bytes = throw_if!(ctx, tokio::fs::read(path).await);
    ArrayBuffer::new(ctx, bytes)
}

#[rquickjs::function]
pub async fn resolve(ctx: Ctx<'_>, path: String) -> rquickjs::Result<String> {
    let path = throw_if!(
        ctx,
        tokio::fs::canonicalize(path)
            .await
            .map(|m| m.display().to_string())
    );

    Ok(path)
}

#[rquickjs::function]
pub async fn write<'js>(ctx: Ctx<'js>, path: String, content: Buffer<'js>) -> rquickjs::Result<()> {
    let Some(content) = content.as_raw() else {
        throw!(ctx, "Buffer is detached")
    };
    throw_if!(ctx, tokio::fs::write(path, content.slice()).await);

    Ok(())
}

#[rquickjs::function(rename = "readDir")]
pub async fn read_dir<'js>(ctx: Ctx<'js>, path: String) -> rquickjs::Result<rquickjs::Value<'js>> {
    let read_dir = throw_if!(ctx, tokio::fs::read_dir(path).await);

    let stream = tokio_stream::wrappers::ReadDirStream::new(read_dir)
        .and_then(|item| async move {
            let ty = item.file_type().await?;

            let ty = if ty.is_dir() {
                "dir"
            } else if ty.is_file() {
                "file"
            } else {
                "symlink"
            };

            Result::<_, std::io::Error>::Ok(JsDirEntry {
                path: item.path().display().to_string(),
                ty,
            })
        })
        .boxed();

    AsyncIter::new(Static(stream)).into_js(&ctx)
}

#[rquickjs::class]
struct ReadDir {
    stream: Option<tokio_stream::wrappers::ReadDirStream>,
}

impl<'js> Trace<'js> for ReadDir {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> AsyncIterable<'js> for ReadDir {
    type Item = JsDirEntry;

    type Error = std::io::Error;

    type Stream = Static<BoxStream<'static, Result<Self::Item, Self::Error>>>;

    fn stream(&mut self, _ctx: &Ctx<'js>) -> rquickjs::Result<AsyncIter<Self::Stream>> {
        let Some(stream) = self.stream.take() else {
            panic!("stream already consumed")
        };

        Ok(AsyncIter::new(Static(
            stream
                .and_then(|item| async move {
                    let ty = item.file_type().await?;

                    let ty = if ty.is_dir() {
                        "dir"
                    } else if ty.is_file() {
                        "file"
                    } else {
                        "symlink"
                    };

                    Result::<_, std::io::Error>::Ok(JsDirEntry {
                        path: item.path().display().to_string(),
                        ty,
                    })
                })
                .boxed(),
        )))
    }
}
