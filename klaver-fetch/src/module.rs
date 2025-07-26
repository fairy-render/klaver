use klaver_base::{Exportable, Registry};
use rquickjs::{
    class::JsClass,
    module::ModuleDef,
    prelude::{Async, Func},
};

use crate::{Headers, URLSearchParams, Url, fetch::fetch, request::Request};

pub struct FetchModule;

impl ModuleDef for FetchModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(Headers::NAME)?;
        decl.declare(Url::NAME)?;
        decl.declare(Request::NAME)?;
        decl.declare(URLSearchParams::NAME)?;
        decl.declare("fetch")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        Self::export(ctx, &Registry::get(ctx)?, exports)
    }
}

impl<'js> Exportable<'js> for FetchModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        Headers::export(ctx, registry, target)?;
        Url::export(ctx, registry, target)?;
        URLSearchParams::export(ctx, registry, target)?;
        Request::export(ctx, registry, target)?;

        target.set(ctx, "fetch", Func::from(Async(fetch)))?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for FetchModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        Self::export(&ctx, &Registry::get(&ctx)?, &ctx.globals())?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for FetchModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }
}
