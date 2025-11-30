use std::borrow::Cow;

use klaver_base::{Exportable, Registry};
use klaver_modules::{Global, GlobalInfo, module_info};
use klaver_util::StringRef;
use rquickjs::{
    Ctx,
    class::JsClass,
    module::ModuleDef,
    prelude::{Async, Func},
};

use crate::{File, FileSystem, FileSystemEntry};

pub struct FsModule;

impl<'js> Exportable<'js> for FsModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        FileSystem::export(ctx, registry, target)?;
        FileSystemEntry::export(ctx, registry, target)?;
        File::export(ctx, registry, target)?;

        Ok(())
    }
}

impl ModuleDef for FsModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(FileSystem::NAME)?;
        decl.declare(FileSystemEntry::NAME)?;
        decl.declare(File::NAME)?;
        decl.declare("open")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        Self::export(&ctx, &Registry::instance(&ctx)?, exports)?;

        exports.export(
            "open",
            Func::new(Async(|ctx: Ctx<'js>, path: StringRef<'js>| async move {
                FileSystem::from_path(ctx, "", std::path::Path::new(path.as_str())).await
            })),
        )?;

        Ok(())
    }
}

module_info!("@klaver/fs" @types: include_str!("../module.d.ts") => FsModule);

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
        Some(Cow::Borrowed(include_str!("../global.d.ts")))
    }
}
