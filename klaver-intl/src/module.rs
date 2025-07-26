use klaver_base::Exportable;
use klaver_util::throw_if;
use rquickjs::Ctx;

use crate::{
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
        registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
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
            use klaver_base::Registry;
            use rquickjs::Object;

            #[cfg(feature = "compiled")]
            if !ctx.userdata::<DynProvider>().is_none() {
                ctx.store_userdata(crate::baked::Baked::new());
            }

            let obj = Object::new(ctx.clone())?;

            Self::export(&ctx, &Registry::get(&ctx)?, &obj)?;

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
