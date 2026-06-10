mod b64;
mod encoding;

use klaver_core::{ExportTarget, Exportable};

pub use self::{
    b64::{atob, btoa},
    encoding::{TextDecoder, TextEncoder},
};
use rquickjs::prelude::Func;

pub struct EncodingModule;

impl<'js> Exportable<'js> for EncodingModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: ExportTarget<'js>,
    {
        TextDecoder::export(ctx, registry, target)?;
        TextEncoder::export(ctx, registry, target)?;

        target.set(ctx, "atob", Func::new(atob))?;
        target.set(ctx, "btoa", Func::new(btoa))?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for EncodingModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        Self::export(
            &ctx,
            &klaver_core::Registry::instance(&ctx)?,
            &ctx.globals(),
        )?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for EncodingModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(std::borrow::Cow::Borrowed(include_str!(
            "../../types/encoding.d.ts"
        )))
    }
}
