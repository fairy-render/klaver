use rquickjs::{function::Args, Ctx, Function, IntoAtom, IntoJs};

use crate::util::ObjectExt;

pub fn concat<'js>(
    ctx: Ctx<'js>,
    first: rquickjs::String<'js>,
    second: rquickjs::String<'js>,
) -> rquickjs::Result<rquickjs::String<'js>> {
    ctx.eval::<Function, _>("(a,b) => a + b")?
        .call((first, second))
}

pub fn concat_many<'js>(
    ctx: Ctx<'js>,
    args: &[rquickjs::String<'js>],
) -> rquickjs::Result<rquickjs::String<'js>> {
    let mut a = Args::new(ctx.clone(), args.len());

    a.push_args(args.iter().map(|m| m.clone()))?;

    ctx.eval::<Function, _>("(...a) => a.join('')")?.call_arg(a)
}
