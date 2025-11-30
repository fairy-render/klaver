use std::pin::Pin;

use futures::{Stream, TryStreamExt, stream::BoxStream};
use klaver_base::Exportable;
use klaver_util::{StringRef, sync::AsyncLock, throw, throw_if};
use pin_project_lite::pin_project;
use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, Object, String, Value,
    atom::PredefinedAtom,
    class::{JsClass, Trace},
    prelude::This,
};
use vfs::boxed::BoxVPath;

use crate::file::File;

#[rquickjs::class]
pub struct FileSystemEntry {
    pub path: BoxVPath,
}

unsafe impl<'js> JsLifetime<'js> for FileSystemEntry {
    type Changed<'to> = FileSystemEntry;
}

impl<'js> Trace<'js> for FileSystemEntry {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

// impl<'js> klaver_util::AsyncIterableProtocol<'js> for FileSystemEntry {
//     type Iterator = klaver_util::StreamAsyncIterator<ListMap>;

//     fn create_stream(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
//         todo!()
//     }
// }

#[rquickjs::methods]
impl FileSystemEntry {
    #[qjs(constructor)]
    fn new(ctx: Ctx<'_>) -> rquickjs::Result<FileSystemEntry> {
        throw!(ctx, "FileSystemEntry cannot be instantiated directly")
    }

    #[qjs(get, rename = "fileName")]
    fn file_name(&self) -> rquickjs::Result<Option<std::string::String>> {
        Ok(self.path.file_name().map(|m| m.to_string()))
    }

    #[qjs(get)]
    fn extension(&self) -> rquickjs::Result<Option<std::string::String>> {
        Ok(self.path.extension().map(|m| m.to_string()))
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    fn to_string(&self) -> rquickjs::Result<std::string::String> {
        Ok(self.path.to_string())
    }

    fn resolve<'js>(
        &self,
        ctx: Ctx<'js>,
        path: StringRef<'js>,
    ) -> rquickjs::Result<FileSystemEntry> {
        Ok(FileSystemEntry {
            path: throw_if!(ctx, self.path.resolve(path.as_str())),
        })
    }

    #[qjs(rename = "listDir")]
    async fn list_dir<'js>(
        ctx: Ctx<'js>,
        this: This<Class<'js, FileSystemEntry>>,
    ) -> rquickjs::Result<Value<'js>> {
        let future = this.borrow().path.read_dir();
        let stream = throw_if!(ctx, future.await);

        let stream =
            klaver_util::StreamAsyncIterator::new(stream.map_ok(|path| FileSystemEntry { path }));

        let iterator = klaver_util::NativeAsyncIterator::new(stream);
        let iterator_class = Class::instance(ctx.clone(), iterator)?;

        Ok(iterator_class.into_value())
    }

    async fn metadata<'js>(
        ctx: Ctx<'js>,
        this: This<Class<'js, FileSystemEntry>>,
    ) -> rquickjs::Result<Value<'js>> {
        let metadata = throw_if!(ctx, this.borrow().path.metadata().await);

        let object = Object::new(ctx.clone())?;

        object.set("size", metadata.size)?;
        object.set("type", if metadata.is_dir() { "dir" } else { "file" })?;

        Ok(object.into_value())
    }

    async fn open<'js>(
        this: This<Class<'js, FileSystemEntry>>,
        ctx: Ctx<'js>,
        options: OpenOptions,
    ) -> rquickjs::Result<File<'js>> {
        let future = this.borrow().path.open(options.inner);
        let inner = throw_if!(ctx, future.await);

        let file_name = String::from_str(
            ctx.clone(),
            this.borrow().path.file_name().unwrap_or("unnamed"),
        )?;

        let mime = this
            .borrow()
            .path
            .extension()
            .map(|m| mime_guess::from_ext(m).first_or_octet_stream())
            .unwrap_or_else(|| mime_guess::mime::APPLICATION_OCTET_STREAM);

        let mime = String::from_str(ctx, &mime.to_string())?;

        Ok(File {
            inner: AsyncLock::new(inner),
            file_name,
            mime,
        })
    }
}

impl<'js> Exportable<'js> for FileSystemEntry {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        target.set(
            ctx,
            FileSystemEntry::NAME,
            Class::<FileSystemEntry>::create_constructor(ctx),
        )?;
        Ok(())
    }
}

pub struct OpenOptions {
    inner: vfs::OpenOptions,
}

impl<'js> FromJs<'js> for OpenOptions {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object = value.get()?;

        Ok(OpenOptions {
            inner: vfs::OpenOptions {
                read: obj.get::<_, Option<bool>>("read")?.unwrap_or_default(),
                write: obj.get::<_, Option<bool>>("write")?.unwrap_or_default(),
                truncate: obj.get::<_, Option<bool>>("truncate")?.unwrap_or_default(),
                create: obj.get::<_, Option<bool>>("create")?.unwrap_or_default(),
                append: obj.get::<_, Option<bool>>("append")?.unwrap_or_default(),
            },
        })
    }
}

pin_project! {
    pub struct ListMap {
        #[pin]
        stream: BoxStream<'static, Result<BoxVPath, vfs::Error>>,
    }
}

impl Stream for ListMap {
    type Item = Result<FileSystemEntry, vfs::Error>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project()
            .stream
            .poll_next(cx)
            .map_ok(|path| FileSystemEntry { path })
    }
}
