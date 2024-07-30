use futures::{future::BoxFuture, FutureExt};
use gerning::{
    arguments::Arguments,
    service::AsyncService,
};
use klaver::throw_if;
use klaver_shared::val::Val;
use rquickjs::{class::Trace, function::Rest, Ctx, FromJs};
use vaerdi::Value;

#[rquickjs::class]
pub struct Service {
    service: Box<dyn DynamicAsyncService<(), Value>>,
}

impl Service {
    pub fn new<T: AsyncService<(), Value>>(service: T) -> Service
    where
        T: AsyncService<(), Value> + 'static,
        for<'a> T::Set<'a>: Send,
        for<'a> T::Get<'a>: Send,
        for<'a> T::Call<'a>: Send,
    {
        Service {
            service: Box::new(BoxedDynamicAsyncService(service)),
        }
    }
}

impl<'js> Trace<'js> for Service {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Service {
    pub async fn call(
        &mut self,
        ctx: Ctx<'_>,
        name: String,
        args: Rest<Val>,
    ) -> rquickjs::Result<Val> {
        let arguments = Arguments::new(args.0.into_iter().map(|m| m.0).collect());
        let ret = throw_if!(ctx, self.service.call(&mut (), &name, arguments).await);
        Ok(Val(ret))
    }

    pub async fn set(&mut self, ctx: Ctx<'_>, name: String, value: Val) -> rquickjs::Result<()> {
        throw_if!(ctx, self.service.set_value(&name, value.0).await);
        Ok(())
    }

    pub async fn get(&mut self, ctx: Ctx<'_>, name: String) -> rquickjs::Result<Option<Val>> {
        let val = throw_if!(ctx, self.service.get_value(&name).await);
        Ok(val.map(Val))
    }
}

#[rquickjs::class]
pub struct Signature {
    inner: gerning::signature::Signature<Value>,
}

impl<'js> Trace<'js> for Signature {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl Signature {}

pub trait DynamicAsyncService<C, V: gerning::Value> {
    fn set_value<'a>(
        &'a self,
        name: &'a str,
        value: V,
    ) -> BoxFuture<'a, Result<(), gerning::Error<V>>>;
    fn get_value<'a>(
        &'a self,
        name: &'a str,
    ) -> BoxFuture<'a, Result<Option<V>, gerning::Error<V>>>;
    fn call<'a>(
        &'a self,
        ctx: &'a mut C,
        name: &'a str,
        args: Arguments<V>,
    ) -> BoxFuture<'a, Result<V, gerning::Error<V>>>;
}

pub struct BoxedDynamicAsyncService<T>(T);

impl<T, C: 'static, V> DynamicAsyncService<C, V> for BoxedDynamicAsyncService<T>
where
    V: gerning::Value + 'static,
    T: AsyncService<C, V>,
    for<'a> T::Set<'a>: Send,
    for<'a> T::Get<'a>: Send,
    for<'a> T::Call<'a>: Send,
{
    fn set_value<'a>(
        &'a self,
        name: &'a str,
        value: V,
    ) -> BoxFuture<'a, Result<(), gerning::Error<V>>> {
        self.0.set_value(name, value).boxed()
    }

    fn get_value<'a>(
        &'a self,
        name: &'a str,
    ) -> BoxFuture<'a, Result<Option<V>, gerning::Error<V>>> {
        self.0.get_value(name).boxed()
    }

    fn call<'a>(
        &'a self,
        ctx: &'a mut C,
        name: &'a str,
        args: Arguments<V>,
    ) -> BoxFuture<'a, Result<V, gerning::Error<V>>> {
        self.0.call(ctx, name, args).boxed()
    }
}
