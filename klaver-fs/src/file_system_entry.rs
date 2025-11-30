use futures::TryStreamExt;
use klaver_util::{StringRef, throw, throw_if};
use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, Object, String, Value, class::Trace, prelude::This,
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

    async fn resolve<'js>(
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
            inner,
            file_name,
            mime,
        })
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
