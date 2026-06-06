use parking_lot::Mutex;
use rquickjs::Ctx;

/// Loader is a trait that defines the interface for loading modules.
/// Contrary to rquickjs's Loader, self is not mutable, and it is expected to be thread safe.
/// This is because the loader will be shared across multiple runtimes, and it should be able to handle concurrent requests.
pub trait Loader {
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        path: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>>;
}

/// Resolver is a trait that defines the interface for resolving module paths.
/// It is used to resolve the module paths before they are loaded by the loader.
/// Contrary to rquickjs's Resolver, self is not mutable, and it is expected to be thread safe.
/// This is because the resolver will be shared across multiple runtimes, and it should be able to handle concurrent requests.
pub trait Resolver {
    fn resolve<'js>(&self, ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String>;
}

/// QuickWrap is a wrapper around a Loader or Resolver that allows it to be shared across multiple runtimes.
#[derive(Debug)]
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
