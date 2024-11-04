use reggie::Body;
use rquickjs::{class::Trace, Class, Ctx, Module, Object};

use crate::timers::Timers;

pub const WINTERCG_KEY: &'static str = "__engine";

#[rquickjs::class]
pub struct WinterCG<'js> {
    #[cfg(feature = "http")]
    http_client: reggie::Client,
    timers: Timers<'js>,
}

impl<'js> Trace<'js> for WinterCG<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.timers.trace(tracer)
    }
}

impl<'js> WinterCG<'js> {
    pub fn new(_ctx: Ctx<'js>) -> rquickjs::Result<WinterCG<'js>> {
        Ok(WinterCG {
            http_client: reggie::Client::new(reqwest::Client::new()),
            timers: Timers::default(),
        })
    }
}

impl<'js> WinterCG<'js> {
    pub async fn get(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, WinterCG<'js>>> {
        let obj = Module::import(ctx, "@klaver/wintercg")?
            .into_future::<Object>()
            .await?;
        obj.get("config")
    }

    pub fn set_http_client<T>(&mut self, client: T)
    where
        T: reggie::HttpClient<Body> + Send + Sync + 'static,
        T::Body: Into<Body>,
        for<'a> T::Future<'a>: Send,
    {
        self.http_client = reggie::Client::new(client);
    }

    pub fn http_client(&self) -> &reggie::Client {
        &self.http_client
    }

    pub fn timers(&self) -> &Timers<'js> {
        &self.timers
    }
}
