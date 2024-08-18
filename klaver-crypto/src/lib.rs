// use buffer::TypedArray;
use digest::Digest;
use klaver::{module_info, throw};
use klaver_shared::buffer::TypedArray;
use rand::RngCore;
use rquickjs::{function::Func, Class, Ctx};

// mod buffer;
mod digest;

pub struct Crypto;

impl rquickjs::module::ModuleDef for Crypto {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("randomUUID")?;
        decl.declare("getRandomValues")?;
        decl.declare("Digest")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::prelude::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        exports.export("randomUUID", Func::new(random_uuid))?;
        exports.export("getRandomValues", Func::new(random_values))?;

        Class::<Digest>::register(ctx)?;
        exports.export("Digest", Class::<Digest>::create_constructor(ctx)?)?;
        Ok(())
    }
}

pub fn random_uuid() -> rquickjs::Result<String> {
    let id = uuid::Uuid::new_v4();
    Ok(id.hyphenated().to_string())
}

pub fn random_values<'js>(ctx: Ctx<'js>, buffer: TypedArray<'js>) -> rquickjs::Result<()> {
    let Some(mut raw) = buffer.as_raw() else {
        throw!(ctx, "TypedArray is detached")
    };

    rand::thread_rng().fill_bytes(raw.slice_mut());

    Ok(())
}

module_info!("@klaver/crypto" => Crypto);
