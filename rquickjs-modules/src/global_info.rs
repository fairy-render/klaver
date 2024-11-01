use std::{borrow::Cow, future::Future, marker::PhantomData, pin::Pin};

use rquickjs::Ctx;

use crate::{
    module_info::{ModuleBuilder, ModuleInfo},
    modules_builder::ModulesBuilder,
    types::Typings,
};

pub struct GlobalBuilder<'a, M> {
    modules: &'a mut ModulesBuilder,
    typings: &'a mut Typings,
    module: PhantomData<M>,
}

impl<'a, M: GlobalInfo> GlobalBuilder<'a, M> {
    pub fn new(modules: &'a mut ModulesBuilder, typings: &'a mut Typings) -> GlobalBuilder<'a, M> {
        GlobalBuilder {
            modules,
            typings,
            module: PhantomData,
        }
    }

    pub fn dependency<T: ModuleInfo>(&mut self) {
        T::register(&mut ModuleBuilder::new(self.modules, self.typings));
        if let Some(typings) = T::typings() {
            self.typings.add_module(T::NAME, typings);
        }
    }

    pub fn global_dependency<T: GlobalInfo>(&mut self) {
        T::register(&mut GlobalBuilder::new(self.modules, self.typings));
        if let Some(typings) = T::typings() {
            self.typings.add_global(typings);
        }
    }

    pub fn register<T: Global + Send + Sync + 'static>(&mut self, global: T) {
        self.modules.register_global(global);
    }
}

pub trait GlobalInfo: Sized {
    fn register(builder: &mut GlobalBuilder<'_, Self>);
    fn typings() -> Option<Cow<'static, str>> {
        None
    }
}

pub trait Global {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a;
}

impl<T> Global for T
where
    for<'js> T: Fn(Ctx<'js>) -> rquickjs::Result<()>,
{
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> impl Future<Output = rquickjs::Result<()>> + 'a {
        async move { (self)(ctx) }
    }
}

pub(crate) trait DynamicGlobal {
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> Pin<Box<dyn Future<Output = rquickjs::Result<()>> + 'a>>;
}

pub(crate) struct GlobalBox<T>(pub T);

impl<T> DynamicGlobal for GlobalBox<T>
where
    T: Global,
{
    fn define<'a, 'js: 'a>(
        &'a self,
        ctx: Ctx<'js>,
    ) -> Pin<Box<dyn Future<Output = rquickjs::Result<()>> + 'a>> {
        Box::pin(async move { self.0.define(ctx).await })
    }
}
