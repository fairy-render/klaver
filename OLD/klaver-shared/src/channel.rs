use futures::{
    future::{BoxFuture, LocalBoxFuture},
    Future,
};
use rquickjs::{class::Trace, Ctx, Exception, FromJs, Value};

pub trait Sender<'js>: Trace<'js> {
    fn send<'a>(
        &'a self,
        ctx: Ctx<'js>,
        message: Value<'js>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<()>>
    where
        'js: 'a;
}

struct SenderBox<T> {
    sender: tokio::sync::mpsc::Sender<T>,
}

impl<'js, T> Trace<'js> for SenderBox<T> {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js, T> Sender<'js> for SenderBox<T>
where
    T: FromJs<'js>,
{
    fn send<'a>(
        &'a self,
        ctx: Ctx<'js>,
        message: Value<'js>,
    ) -> LocalBoxFuture<'a, rquickjs::Result<()>>
    where
        'js: 'a,
    {
        Box::pin(async move {
            let value = T::from_js(&ctx, message)?;
            self.sender.send(value).await.ok();

            Ok(())
        })
    }
}

#[rquickjs::class]
pub struct JsSender<'js> {
    sender: Option<Box<dyn Sender<'js> + 'js>>,
}

impl<'js> Trace<'js> for JsSender<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        if let Some(sender) = self.sender.as_ref() {
            sender.trace(tracer)
        }
    }
}

impl<'js> JsSender<'js> {
    pub fn new<T>(sender: tokio::sync::mpsc::Sender<T>) -> JsSender<'js>
    where
        T: FromJs<'js> + 'static,
    {
        JsSender {
            sender: Some(Box::new(SenderBox { sender })),
        }
    }
}

#[rquickjs::methods]
impl<'js> JsSender<'js> {
    pub async fn send(&self, ctx: Ctx<'js>, message: Value<'js>) -> rquickjs::Result<()> {
        let Some(sender) = self.sender.as_ref() else {
            return Err(ctx.throw(Value::from_exception(Exception::from_message(
                ctx.clone(),
                "channel is closed",
            )?)));
        };

        sender.send(ctx, message).await
    }

    fn close(&mut self) -> rquickjs::Result<()> {
        if let Some(sender) = self.sender.take() {
            drop(sender);
        }

        Ok(())
    }
}
