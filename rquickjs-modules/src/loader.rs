use parking_lot::Mutex;
use rquickjs::Ctx;

pub trait Loader {
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>>;
}

pub trait Resolver {
    fn resolve<'js>(&self, ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String>;
}

pub struct QuickWrap<T>(Mutex<T>);

impl<T> QuickWrap<T> {
    pub fn new(item: T) -> QuickWrap<T> {
        QuickWrap(Mutex::new(item))
    }
}

impl<T> Loader for QuickWrap<T>
where
    T: rquickjs::loader::Loader,
{
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        self.0.lock().load(ctx, path)
    }
}

impl<T> Resolver for QuickWrap<T>
where
    T: rquickjs::loader::Resolver,
{
    fn resolve<'js>(&self, ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String> {
        self.0.lock().resolve(ctx, base, name)
    }
}
