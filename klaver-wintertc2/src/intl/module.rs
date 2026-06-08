use klaver_core::Exportable;
use klaver_core::throw_if;
use rquickjs::Ctx;

use super::{
    datetime::DateTimeFormat,
    locale::JsLocale,
    numberformat::NumberFormat,
    provider::{DynProvider, ProviderTrait},
};

pub struct IntlModule;

impl IntlModule {
    pub fn set_provider<T>(ctx: &Ctx<'_>, provider: T) -> rquickjs::Result<()>
    where
        T: ProviderTrait + 'static,
    {
        throw_if!(ctx, ctx.store_userdata(DynProvider::new(provider)));
        Ok(())
    }
}

impl<'js> Exportable<'js> for IntlModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        DateTimeFormat::export(ctx, registry, target)?;
        NumberFormat::export(ctx, registry, target)?;
        JsLocale::export(ctx, registry, target)?;
        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for IntlModule {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            use klaver_core::Registry;
            use rquickjs::Object;

            #[cfg(feature = "intl-baked")]
            if !ctx.userdata::<DynProvider>().is_none() {
                ctx.store_userdata(DynProvider::new(crate::baked::Baked::new()))?;
            }

            let obj = Object::new(ctx.clone())?;

            Self::export(&ctx, &Registry::instance(&ctx)?, &obj)?;

            ctx.globals().set("Intl", obj)?;

            Ok(())
        }
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for IntlModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(IntlModule);
    }
}
