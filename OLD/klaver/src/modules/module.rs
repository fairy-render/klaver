use std::borrow::Cow;

use rquickjs::module::ModuleDef;

use crate::vm::ModuleTypings;

use super::ModulesBuilder;

pub struct Builder<'a> {
    modules: &'a mut ModulesBuilder,
    typings: &'a mut Vec<ModuleTypings>,
}

impl<'a> Builder<'a> {
    pub fn new(
        modules: &'a mut ModulesBuilder,
        typings: &'a mut Vec<ModuleTypings>,
    ) -> Builder<'a> {
        Builder { modules, typings }
    }

    pub fn dependency<T: ModuleInfo>(&mut self) {
        T::register(self);
        if let Some(typings) = T::typings() {
            self.typings.push(ModuleTypings {
                name: T::NAME,
                typings,
            });
        }
    }

    pub fn register<T: ModuleDef>(&mut self, name: impl ToString) {
        self.modules
            .modules
            .insert(name.to_string(), ModulesBuilder::load_func::<T>);
    }

    pub fn register_src(&mut self, name: impl ToString, source: Vec<u8>) -> &mut Self {
        self.modules.register_src(name, source);
        self
    }
}

pub trait ModuleInfo {
    const NAME: &'static str;
    fn register(modules: &mut Builder<'_>);
    fn typings() -> Option<Cow<'static, str>> {
        None
    }
}

#[macro_export]
/// module_info!("module" => Module);
macro_rules! module_info {
    ($name: literal => $module: ident) => {
        impl $crate::modules::ModuleInfo for $module {
            const NAME: &'static str = $name;
            fn register(mut modules: &mut $crate::modules::Builder<'_>) {
                modules.register::<$module>($name);
            }
        }
    };
    ($name: literal @types: $types:expr => $module: ident) => {
        impl $crate::modules::ModuleInfo for $module {
            const NAME: &'static str = $name;
            fn register(mut modules: &mut $crate::modules::Builder<'_>) {
                modules.register::<$module>($name);
            }

            fn typings() -> Option<std::borrow::Cow<'static, str>> {
                Some($types.into())
            }
        }
    };
}
