use rquickjs::{class::Trace, Class, Ctx, Module, Object};

use crate::timers::Timers;

pub const WINTERCG_KEY: &'static str = "__engine";

#[rquickjs::class]
pub struct WinterCG<'js> {
    #[cfg(feature = "http")]
    http_client: reggie::Client,
    #[cfg(feature = "http")]
    base_url: url::Url,
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
            #[cfg(feature = "http")]
            http_client: reggie::Client::new(reqwest::Client::new()),
            #[cfg(feature = "http")]
            base_url: url::Url::parse("internal://internal.com").expect("base url"),
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

    #[cfg(feature = "http")]
    pub fn set_http_client(&mut self, client: reggie::Client) {
        self.http_client = client;
    }

    #[cfg(feature = "http")]
    pub fn http_client(&self) -> &reggie::Client {
        &self.http_client
    }

    #[cfg(feature = "http")]
    pub fn set_base_url(&mut self, url: url::Url) {
        self.base_url = url;
    }

    #[cfg(feature = "http")]
    pub fn base_url(&self) -> &url::Url {
        &self.base_url
    }

    pub fn timers(&self) -> &Timers<'js> {
        &self.timers
    }
}
