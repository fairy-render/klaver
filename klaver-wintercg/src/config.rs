use rquickjs::{class::Trace, Class, Ctx, JsLifetime, Object};
use rquickjs_util::{throw, typed_map::TypedMap};

use crate::timers::Timers;

pub const WINTERCG_KEY: &'static str = "__engine";

pub type Environ<'js> = TypedMap<'js, rquickjs::String<'js>, rquickjs::String<'js>>;

#[rquickjs::class]
pub struct WinterCG<'js> {
    #[cfg(feature = "http")]
    http_client: reggie::Client,
    #[cfg(feature = "http")]
    base_url: url::Url,
    timers: Timers,
    env: Environ<'js>,
    #[cfg(feature = "icu")]
    provider: Option<crate::intl::provider::DynProvider>,
}

unsafe impl<'js> JsLifetime<'js> for WinterCG<'js> {
    type Changed<'to> = WinterCG<'to>;
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
                crate::intl::baked::Baked::new(),
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

    pub fn timers(&self) -> &Timers {
        &self.timers
    }

    pub fn env(&self) -> &Environ<'js> {
        &self.env
    }

    #[cfg(feature = "icu")]
    pub fn icu_provider(
        &self,
        ctx: &Ctx<'js>,
    ) -> rquickjs::Result<&crate::intl::provider::DynProvider> {
        let Some(provider) = self.provider.as_ref() else {
            throw!(ctx, "ICU dataprovider not set")
        };

        Ok(provider)
    }

    #[cfg(feature = "icu")]
    pub fn set_icu_provider<P: crate::intl::provider::ProviderTrait + 'static>(
        &mut self,
        provider: P,
    ) {
        self.provider = Some(crate::intl::provider::DynProvider::new(provider));
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
