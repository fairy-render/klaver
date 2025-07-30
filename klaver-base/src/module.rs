use rquickjs::{Ctx, class::JsClass, module::ModuleDef, prelude::Func};

pub struct BaseModule;

use crate::{
    Console, EventTarget, Exportable, Registry, abort_controller::AbortController,
    abort_signal::AbortSignal, blob::Blob, dom_exception::DOMException, events::Event, file::File,
    serialize, structured_clone,
};

impl ModuleDef for BaseModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        declare!(
            decl,
            AbortController,
            AbortSignal,
            EventTarget,
            Event,
            DOMException,
            Blob,
            File,
            Console
        );

        crate::streams::declare(decl)?;
        crate::encoding::declare(decl)?;
        crate::message::declare(decl)?;

        decl.declare("structuredClone")?;
        decl.declare("serialize")?;

        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        let registry = &Registry::instance(ctx)?;

        Self::export(ctx, registry, exports)
    }
}

impl<'js> Exportable<'js> for BaseModule {
    fn export<T>(ctx: &Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::ExportTarget<'js>,
    {
        export!(
            ctx,
            registry,
            target,
            AbortController,
            AbortSignal,
            DOMException,
            Blob,
            Console
        );
        crate::events::exports(ctx, registry, target)?;
        crate::encoding::export(ctx, registry, target)?;
        crate::streams::export(ctx, registry, target)?;
        crate::message::export(ctx, registry, target)?;

        target.set(ctx, "structuredClone", Func::from(structured_clone))?;
        target.set(ctx, "serialize", Func::from(serialize))?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for BaseModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        Self::export(&ctx, &Registry::instance(&ctx)?, &ctx.globals())?;
        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for BaseModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(BaseModule);
    }

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(std::borrow::Cow::Borrowed(include_str!(
            "../klaver.base.d.ts"
        )))
    }
}
