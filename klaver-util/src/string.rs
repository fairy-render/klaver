use rquickjs::{Ctx, Function, IntoJs, String, function::Args};

use crate::{ObjectExt, StringRef};

pub trait StringExt<'js> {
    fn starts_with<K: IntoJs<'js>>(&self, ctx: Ctx<'js>, prefix: K) -> rquickjs::Result<bool>;
    fn length(&self, ctx: Ctx<'js>) -> rquickjs::Result<usize>;
    fn to_lowercase(&self, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>>;
    fn str_ref(&self) -> rquickjs::Result<StringRef<'js>>;
}

impl<'js> StringExt<'js> for rquickjs::String<'js> {
    fn starts_with<K: IntoJs<'js>>(&self, _ctx: Ctx<'js>, prefix: K) -> rquickjs::Result<bool> {
        self.call_property("startsWith", (prefix,))
    }

    fn length(&self, ctx: Ctx<'js>) -> rquickjs::Result<usize> {
        ctx.eval::<Function, _>("(a) => a.length")?
            .call((self.clone(),))
    }

    fn to_lowercase(&self, _ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        self.call_property("toLowerCase", ())
    }

    fn str_ref(&self) -> rquickjs::Result<StringRef<'js>> {
        StringRef::from_string(self.clone())
    }
}

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
