use std::convert::Infallible;

use futures::stream::BoxStream;
use klaver_shared::{
    date::Date,
    iter::{AsyncIter, AsyncIterable, Iter, Iterable},
    Static,
};
use rquickjs::{
    class::Trace, function::Func, CatchResultExt, Class, Context, Ctx, Function, Runtime,
};

#[derive(Trace)]
#[rquickjs::class]
pub struct Test {
    list: Option<Vec<String>>,
}

impl<'js> Iterable<'js> for Test {
    type Item = String;

    type Iter = std::vec::IntoIter<String>;

    fn entries(&mut self) -> klaver_shared::iter::Iter<Self::Iter> {
        Iter::new(self.list.take().unwrap_or_default().into_iter())
    }
}

impl<'js> AsyncIterable<'js> for Test {
    type Item = String;

    type Error = Infallible;

    type Stream = Static<BoxStream<'static, Result<Self::Item, Self::Error>>>;

    fn stream(
        &mut self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<klaver_shared::iter::AsyncIter<Self::Stream>> {
        let iter = self
            .list
            .take()
            .unwrap_or_default()
            .into_iter()
            .map(Result::<_, Infallible>::Ok);
        let stream = futures::stream::iter(iter);

        Ok(AsyncIter::new(Static(Box::pin(stream))))
    }
}

fn main() {
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();

    context
        .with(|ctx| {
            ctx.globals().set(
                "print",
                Func::new(|arg: String| {
                    println!("{arg}");
                    rquickjs::Result::Ok(())
                }),
            )?;

            Class::<Test>::register(&ctx)?;
            Test::add_iterable_prototype(&ctx)?;
            Test::add_async_iterable_prototype(&ctx)?;

            let fun = ctx.eval::<Function, _>(
                r#"(arg) => {
                for (const item of arg) {
                    print(item)
                }
            }"#,
            )?;

            fun.call::<_, ()>((Test {
                list: vec!["Hello".to_string(), "World".to_string()].into(),
            },))
                .catch(&ctx)
                .unwrap();

            rquickjs::Result::Ok(())
        })
        .unwrap();
}
