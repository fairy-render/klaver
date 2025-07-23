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
