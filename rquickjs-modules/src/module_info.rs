use std::{borrow::Cow, marker::PhantomData};

use rquickjs::module::ModuleDef;

use crate::{modules_builder::ModulesBuilder, types::Typings};

pub struct ModuleBuilder<'a, M> {
    modules: &'a mut ModulesBuilder,
    typings: &'a mut Typings,
    module: PhantomData<M>,
}

impl<'a, M: ModuleInfo> ModuleBuilder<'a, M> {
    pub fn new(modules: &'a mut ModulesBuilder, typings: &'a mut Typings) -> ModuleBuilder<'a, M> {
        ModuleBuilder {
            modules,
            typings,
            module: PhantomData,
        }
    }

    pub fn dependency<T: ModuleInfo>(&mut self) {
        T::register(&mut ModuleBuilder {
            modules: self.modules,
            typings: self.typings,
            module: PhantomData,
        });
        if let Some(typings) = T::typings() {
            self.typings.add_module(T::NAME, typings);
        }
    }

    pub fn register<T: ModuleDef>(&mut self) {
        self.modules
            .modules
            .insert(M::NAME.to_string(), ModulesBuilder::load_func::<T>);
    }

    pub fn register_source(&mut self, source: Vec<u8>) -> &mut Self {
        self.modules.register_source(M::NAME.to_string(), source);
        self
    }
}

pub trait ModuleInfo: Sized {
    const NAME: &'static str;
    fn register(modules: &mut ModuleBuilder<'_, Self>);
    fn typings() -> Option<Cow<'static, str>> {
        None
    }
}

#[macro_export]
/// module_info!("module" => Module);
macro_rules! module_info {
    ($name: literal => $module: ident) => {
        impl $crate::ModuleInfo for $module {
            const NAME: &'static str = $name;
            fn register(mut modules: &mut $crate::ModuleBuilder<'_, Self>) {
                modules.register::<$module>();
            }
        }
    };
    ($name: literal @types: $types:expr => $module: ident) => {
        impl $crate::ModuleInfo for $module {
            const NAME: &'static str = $name;
            fn register(mut modules: &mut $crate::ModuleBuilder<'_, Self>) {
                modules.register::<$module>();
            }

            fn typings() -> Option<std::borrow::Cow<'static, str>> {
                Some($types.into())
            }
        }
    };
}
