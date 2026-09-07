use klaver_core::value::Buffer;
use klaver_core::{Exportable, Registry};
use rquickjs::{
    Ctx, Object,
    module::ModuleDef,
    prelude::{Async, Func},
};

use super::digest::{Algo, Digest};

pub struct CryptoModule;

impl ModuleDef for CryptoModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("randomUUID")?;
        decl.declare("getRandomValues")?;
        decl.declare("subtle")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        Self::export(ctx, &Registry::instance(ctx)?, exports)?;
        Ok(())
    }
}

impl<'js> Exportable<'js> for CryptoModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
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

        target.set(ctx, "randomUUID", Func::new(super::random::random_uuid))?;
        target.set(
            ctx,
            "getRandomValues",
            Func::new(super::random::random_values),
        )?;

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

            Self::export(&ctx, &Registry::instance(&ctx)?, &obj)?;

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

    fn typings() -> Option<std::borrow::Cow<'static, str>> {
        Some(std::borrow::Cow::Borrowed(include_str!(
            "../../types/crypto.d.ts"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{CatchResultExt, Context, Runtime};

    /// Runs `body` as the contents of a plain function, with a global `crypto` object available.
    /// `body` is expected to throw on failure (e.g. via a plain `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                let crypto = Object::new(ctx.clone())?;
                CryptoModule::export(&ctx, &Registry::instance(&ctx)?, &crypto)?;
                ctx.globals().set("crypto", crypto)?;

                let test_fn: rquickjs::Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn get_random_values_is_registered_under_the_spec_name() {
        run(r#"
            if (typeof crypto.getRandomValues !== "function") {
                throw new Error("crypto.getRandomValues is not a function");
            }
            if (typeof crypto.randomValues !== "undefined") {
                throw new Error("crypto.randomValues should not exist");
            }

            const buf = new Uint8Array(16);
            crypto.getRandomValues(buf);
            if (buf.every((b) => b === 0)) throw new Error("buffer was not filled");
        "#);
    }
}
