use rquickjs::{class::Trace, Class, Ctx, Object};
use rquickjs_util::typed_map::TypedMap;

use crate::timers::Timers;

pub const WINTERCG_KEY: &'static str = "__engine";

pub type Environ<'js> = TypedMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>;

#[rquickjs::class]
pub struct WinterCG<'js> {
    #[cfg(feature = "http")]
    http_client: reggie::Client,
    #[cfg(feature = "http")]
    base_url: url::Url,
    timers: Timers<'js>,
    env: Environ<'js>,
    #[cfg(feature = "icu")]
    provider: Option<crate::intl::provider::DynProvider>,
}

impl<'js> Trace<'js> for WinterCG<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.timers.trace(tracer);
        self.env.trace(tracer);
    }
}

impl<'js> WinterCG<'js> {
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<WinterCG<'js>> {
        Ok(WinterCG {
            #[cfg(feature = "http")]
            http_client: reggie::Client::new(reqwest::Client::new()),
            #[cfg(feature = "http")]
            base_url: url::Url::parse("internal://internal.com").expect("base url"),
            timers: Timers::default(),
            env: TypedMap::new(ctx)?,
            #[cfg(all(feature = "icu", not(feature = "icu-compiled")))]
            provider: None,
            #[cfg(all(feature = "icu", feature = "icu-compiled"))]
            provider: Some(crate::intl::provider::DynProvider::new(
                icu::datetime::provider::Baked,
            )),
        })
    }
}

impl<'js> WinterCG<'js> {
    pub fn get(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, WinterCG<'js>>> {
        ctx.globals().get(WINTERCG_KEY)
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

    pub fn set_args(&self, ctx: Ctx<'js>, args: Vec<String>) -> rquickjs::Result<()> {
        ctx.globals().get::<_, Object>("process")?.set("args", args)
    }

    pub fn timers(&self) -> &Timers<'js> {
        &self.timers
    }

    pub fn env(&self) -> &Environ<'js> {
        &self.env
    }

    pub fn init_env_from_os(&self, ctx: Ctx<'js>) -> rquickjs::Result<()> {
        for (k, v) in std::env::vars() {
            self.env.set(
                rquickjs::String::from_str(ctx.clone(), &k)?,
                rquickjs::String::from_str(ctx.clone(), &v)?,
            )?;
        }

        Ok(())
    }
}
