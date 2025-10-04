use rquickjs::{Ctx, FromJs, Function, Value, function::Args, prelude::IntoArgs};

pub trait FunctionExt<'js> {
    fn call_async<A: IntoArgs<'js>, R: FromJs<'js>>(
        &self,
        args: A,
    ) -> impl Future<Output = rquickjs::Result<R>>;

    fn bind<A: IntoArgs<'js>>(&self, ctx: &Ctx<'js>, args: A) -> rquickjs::Result<Function<'js>>;
}

impl<'js> FunctionExt<'js> for Function<'js> {
    async fn call_async<A: IntoArgs<'js>, R: FromJs<'js>>(&self, args: A) -> rquickjs::Result<R> {
        let mut value = self.call::<_, Value<'js>>(args)?;

        if let Some(promise) = value.as_promise() {
            value = promise.clone().into_future::<Value>().await?;
        }

        value.get()
    }

    fn bind<A: IntoArgs<'js>>(&self, ctx: &Ctx<'js>, args: A) -> rquickjs::Result<Function<'js>> {
        let mut a = Args::new(ctx.clone(), args.num_args());
        args.into_args(&mut a)?;
        a.this(self.clone())?;
        self.get::<_, Function>("bind")?.call_arg::<Function>(a)
    }
}
