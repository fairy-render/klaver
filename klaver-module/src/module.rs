use rquickjs::module::ModuleDef;

use crate::Modules;

pub struct Builder<'a> {
    modules: &'a mut Modules,
}

impl<'a> Builder<'a> {
    pub fn new(modules: &'a mut Modules) -> Builder<'a> {
        Builder { modules }
    }

    pub fn register<T: ModuleDef>(&mut self, name: impl ToString) {
        self.modules
            .modules
            .insert(name.to_string(), Modules::load_func::<T>);
    }
}

pub trait ModuleInfo {
    fn register(modules: Builder<'_>);
}

#[macro_export]
macro_rules! module_info {
    ($name: literal => $module: ident) => {
        impl $crate::ModuleInfo for $module {
            fn register(mut modules: $crate::Builder<'_>) {
                modules.register::<$module>($name);
            }
        }
    };
}
