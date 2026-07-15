use std::path::Path;

use klaver_core::Exportable;
use klaver_core::{throw, throw_if};
use rquickjs::{
    Class, Ctx, JsLifetime, String,
    class::{JsClass, Trace},
};
use vfs::boxed::{BoxVPath, LocalBoxVPath};
use vfs::{VFS, VPathExt, boxed::BoxVFS};

use super::file_system_entry::FileSystemEntry;

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
    pub fn new(
        ctx: Ctx<'js>,
        name: &str,
        path: LocalBoxVPath,
    ) -> rquickjs::Result<FileSystem<'js>> {
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
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            FileSystem::NAME,
            Class::<FileSystem>::create_constructor(ctx),
        )?;
        Ok(())
    }
}
