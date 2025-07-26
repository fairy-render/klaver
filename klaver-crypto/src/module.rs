use klaver_base::{Exportable, Registry};
use klaver_util::Buffer;
use rquickjs::{
    Ctx, Object,
    module::ModuleDef,
    prelude::{Async, Func},
};

use crate::digest::{Algo, Digest};

pub struct CryptoModule;

impl ModuleDef for CryptoModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("randomUUID")?;
        decl.declare("randomValues")?;
        decl.declare("subtle")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        Self::export(ctx, &Registry::get(ctx)?, exports)?;
        Ok(())
    }
}

impl<'js> Exportable<'js> for CryptoModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_base::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_base::ExportTarget<'js>,
    {
        let subtle = Object::new(ctx.clone())?;

        Digest::export(ctx, registry, &subtle)?;

        subtle.set(
            "digest",
            Func::new(Async(
                |ctx: Ctx<'js>, algo: Algo, buffer: Buffer<'js>| async move {
                    let mut digest = Digest::new(algo)?;
                    digest.update(ctx.clone(), buffer)?;
                    digest.digest(ctx)
                },
            )),
        )?;

        target.set(ctx, "randomUUID", Func::new(crate::random::random_uuid))?;
        target.set(ctx, "randomValues", Func::new(crate::random::random_values))?;

        target.set(ctx, "subtle", subtle)?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for CryptoModule {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: rquickjs::Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move {
            let obj = Object::new(ctx.clone())?;

            Self::export(&ctx, &Registry::get(&ctx)?, &obj)?;

            ctx.globals().set("crypto", obj)?;

            Ok(())
        }
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for CryptoModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(CryptoModule {});
    }
}
