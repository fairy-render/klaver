use rquickjs::{Ctx, Function, IntoJs, String, function::Args};

use crate::value::StringRef;

pub trait StringExt<'js> {
    fn starts_with<K: IntoJs<'js>>(&self, ctx: Ctx<'js>, prefix: K) -> rquickjs::Result<bool>;
    fn length(&self, ctx: Ctx<'js>) -> rquickjs::Result<usize>;
    fn to_lowercase(&self, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>>;
    fn str_ref(&self) -> rquickjs::Result<StringRef<'js>>;
}

impl<'js> StringExt<'js> for rquickjs::String<'js> {
    fn starts_with<K: IntoJs<'js>>(&self, ctx: Ctx<'js>, prefix: K) -> rquickjs::Result<bool> {
        // `call_property` (via `ObjectExt for Value`) coerces `self` through `Object::from_js`,
        // which fails for primitive JS strings (they aren't objects). Route the call through
        // JS itself, which autoboxes primitives for property/method lookup, like `length` below.
        ctx.eval::<Function, _>("(a, b) => a.startsWith(b)")?
            .call((self.clone(), prefix))
    }

    fn length(&self, ctx: Ctx<'js>) -> rquickjs::Result<usize> {
        ctx.eval::<Function, _>("(a) => a.length")?
            .call((self.clone(),))
    }

    fn to_lowercase(&self, ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        ctx.eval::<Function, _>("(a) => a.toLowerCase()")?
            .call((self.clone(),))
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
