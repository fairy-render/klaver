use reggie::{Body, BoxClient};
use rquickjs::{class::Trace, Class, Ctx};

pub const WINTERCG_KEY: &'static str = "__engine";

#[rquickjs::class]
pub struct WinterCG {
    #[cfg(feature = "http")]
    http_client: reggie::Client,
}

impl<'js> Trace<'js> for WinterCG {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl WinterCG {
    pub fn new<'js>(ctx: Ctx<'js>) -> rquickjs::Result<WinterCG> {
        Ok(WinterCG {
            http_client: reggie::Client::new(reqwest::Client::new()),
        })
    }
}

impl WinterCG {
    pub fn get<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, WinterCG>> {
        ctx.globals().get(WINTERCG_KEY)
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
}
