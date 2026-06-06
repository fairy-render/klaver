use std::{borrow::Cow, future::Future, marker::PhantomData, pin::Pin};

use rquickjs::Ctx;

use crate::{
    module_info::{ModuleBuilder, ModuleInfo},
    modules_builder::ModulesBuilder,
    types::Typings,
};

/// GlobalBuilder is a struct that contains the modules and typings for the global info.
/// It is used to register the globals and their dependencies, and it is also used to generate the typings for the globals.
pub struct GlobalBuilder<'a, M> {
    modules: &'a mut ModulesBuilder,
    typings: &'a mut Typings,
    module: PhantomData<M>,
}

impl<'a, M: GlobalInfo> GlobalBuilder<'a, M> {
    pub(crate) fn new(
        modules: &'a mut ModulesBuilder,
        typings: &'a mut Typings,
    ) -> GlobalBuilder<'a, M> {
        GlobalBuilder {
            modules,
            typings,
            module: PhantomData,
        }
    }

    /// Registers a dependency on another module. This will ensure that the module is loaded before the global is defined.
    pub fn dependency<T: ModuleInfo>(&mut self) {
        T::register(&mut ModuleBuilder::new(self.modules, self.typings));
        if let Some(typings) = T::typings() {
            self.typings.add_module(T::NAME, typings);
        }
    }

    /// Registers a dependency on another global. This will ensure that the global is defined before the global is defined.
    pub fn global_dependency<T: GlobalInfo>(&mut self) {
        T::register(&mut GlobalBuilder::new(self.modules, self.typings));
        if let Some(typings) = T::typings() {
            self.typings.add_global(typings);
        }
    }

    /// Registers a global. This will ensure that the global is defined before it is used.
    pub fn register<T: Global + Send + Sync + 'static>(&mut self, global: T) {
        self.modules.register_global(global);
    }
}

/// Define a global module
pub trait GlobalInfo: Sized {
    fn register(builder: &mut GlobalBuilder<'_, Self>);
    fn typings() -> Option<Cow<'static, str>> {
        None
    }
}

/// Global is a trait that defines the interface for defining globals.
/// It is used to define globals that can be attached to the context.
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
