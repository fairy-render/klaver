use std::borrow::Cow;

use klaver_core::value::StringRef;
use klaver_core::{Exportable, Registry};
use klaver_modules::{Global, GlobalInfo};
use rquickjs::{
    Ctx,
    prelude::{Async, Func},
};

use super::{File, FileSystem, FileSystemEntry};

pub struct FsModule;

impl<'js> Exportable<'js> for FsModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        FileSystem::export(ctx, registry, target)?;
        FileSystemEntry::export(ctx, registry, target)?;
        File::export(ctx, registry, target)?;

        target.set(
            ctx,
            "open",
            Func::new(Async(|ctx: Ctx<'js>, path: StringRef<'js>| async move {
                FileSystem::from_path(ctx, "", std::path::Path::new(path.as_str())).await
            })),
        )?;

        Ok(())
    }
}

impl Global for FsModule {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: rquickjs::Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            Self::export(&ctx, &Registry::instance(&ctx)?, &ctx.globals())?;

            Ok(())
        }
    }
}

impl GlobalInfo for FsModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(Cow::Borrowed(include_str!("../../types/fs.d.ts")))
    }
}
