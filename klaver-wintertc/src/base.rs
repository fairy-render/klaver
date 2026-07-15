#[cfg(feature = "module")]
use klaver_core::Registry;
use klaver_core::{Exportable, value::structured_clone};
#[cfg(feature = "module")]
use rquickjs::Ctx;
use rquickjs::prelude::Func;

use crate::{
    abort_controller::{AbortController, AbortSignal},
    channel::ChannelModule,
    dom_exception::DOMException,
    encoding::EncodingModule,
    events::EventsModule,
};

pub struct BaseModule;

impl<'js> klaver_core::Exportable<'js> for BaseModule {
    fn export<T>(ctx: &Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        export!(
            ctx,
            registry,
            target,
            AbortController,
            AbortSignal,
            DOMException
        );

        EventsModule::export(ctx, registry, target)?;
        EncodingModule::export(ctx, registry, target)?;
        ChannelModule::export(ctx, registry, target)?;

        #[cfg(feature = "streams")]
        crate::streams::export(ctx, registry, target)?;
        #[cfg(feature = "streams")]
        crate::blob::Blob::export(ctx, registry, target)?;
        target.set(
            ctx,
            "structuredClone",
            Func::from(structured_clone::structured_clone),
        )?;
        target.set(ctx, "serialize", Func::from(structured_clone::serialize))?;

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
            "../types/base.d.ts"
        )))
    }
}
