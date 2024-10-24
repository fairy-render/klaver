mod digest;
mod random;

use klaver_shared::Buffer;
use rquickjs::{
    prelude::{Async, Func},
    Ctx, Object,
};

pub use self::{digest::*, random::*};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    decl.declare("crypto")?;
    Ok(())
}

pub fn evaluate<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    let object = Object::new(ctx.clone())?;

    let subtle = Object::new(ctx.clone())?;

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

    object.set("subtle", subtle)?;

    object.set("getRandomValues", Func::new(random_values))?;
    object.set("randomUUID", Func::new(random_uuid))?;

    exports.export("crypto", object)?;

    Ok(())
}
