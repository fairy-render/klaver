use klaver::throw_if;
use rquickjs::{class::Trace, function::Opt, Ctx};

#[derive(Debug)]
#[rquickjs::class]
pub struct Url {
    i: url::Url,
}

impl<'js> Trace<'js> for Url {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Url {
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, url: String, base: Opt<String>) -> rquickjs::Result<Url> {
        let i = if let Some(base) = base.0 {
            throw_if!(ctx, url::Url::parse(&base).and_then(|m| m.join(&url)))
        } else {
            throw_if!(ctx, url::Url::parse(&url))
        };

        Ok(Url { i })
    }

    #[qjs(get)]
    pub fn hash(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.fragment().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn host(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.host_str().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn password(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.password().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn port(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.port().map(|m| m.to_string()))
    }

    #[qjs(get)]
    pub fn protocol(&self) -> rquickjs::Result<String> {
        Ok(self.i.scheme().to_string())
    }

    #[qjs(get)]
    pub fn href(&self) -> rquickjs::Result<String> {
        Ok(self.i.to_string())
    }

    #[qjs(get)]
    pub fn search(&self) -> rquickjs::Result<Option<String>> {
        Ok(self.i.query().map(|m| m.to_string()))
    }
}
