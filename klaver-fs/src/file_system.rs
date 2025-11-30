use std::path::Path;

use klaver_base::Exportable;
use klaver_util::{throw, throw_if};
use rquickjs::{
    Class, Ctx, JsLifetime, String,
    class::{JsClass, Trace},
};
use vfs::{VFS, VPathExt, boxed::BoxVFS};

use crate::file_system_entry::FileSystemEntry;

#[rquickjs::class]
pub struct FileSystem<'js> {
    #[qjs(get)]
    name: String<'js>,
    #[qjs(get)]
    root: Class<'js, FileSystemEntry>,
}

unsafe impl<'js> JsLifetime<'js> for FileSystem<'js> {
    type Changed<'to> = FileSystem<'to>;
}

impl<'js> Trace<'js> for FileSystem<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.name.trace(tracer);
        self.root.trace(tracer);
    }
}

impl<'js> FileSystem<'js> {
    pub fn new(ctx: Ctx<'js>, name: &str, fs: &BoxVFS) -> rquickjs::Result<FileSystem<'js>> {
        let path = throw_if!(ctx, fs.path("."));
        let name = String::from_str(ctx.clone(), name)?;

        let root = Class::instance(ctx.clone(), FileSystemEntry { path })?;

        Ok(FileSystem { name, root })
    }

    pub async fn from_path(
        ctx: Ctx<'js>,
        name: &str,
        path: &Path,
    ) -> rquickjs::Result<FileSystem<'js>> {
        let fs = throw_if!(ctx, vfs_tokio::FS::new(path.to_path_buf()).await);

        let path = throw_if!(ctx, fs.path(".")).boxed();

        let name = String::from_str(ctx.clone(), name)?;
        let root = Class::instance(ctx.clone(), FileSystemEntry { path })?;

        Ok(FileSystem { name, root })
    }
}

#[rquickjs::methods]
impl<'js> FileSystem<'js> {
    #[qjs(constructor)]
    fn ctor(ctx: Ctx<'js>) -> rquickjs::Result<FileSystem<'js>> {
        throw!(@type ctx, "FileSystem cannot be instantiated directly")
    }
}

impl<'js> Exportable<'js> for FileSystem<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        target.set(
            ctx,
            FileSystem::NAME,
            Class::<FileSystem>::create_constructor(ctx),
        )?;
        Ok(())
    }
}
