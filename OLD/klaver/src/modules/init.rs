use rquickjs::Ctx;

pub trait Init {
    fn init<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<()>;
}

impl<T> Init for T
where
    for<'js> T: Fn(Ctx<'js>) -> rquickjs::Result<()>,
{
    fn init<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        (self)(ctx)
    }
}
