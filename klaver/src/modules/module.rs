use rquickjs::module::ModuleDef;

use super::Modules;

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

    pub fn register_src(&mut self, name: impl ToString, source: Vec<u8>) -> &mut Self {
        self.modules.register_src(name, source);
        self
    }
}

pub trait ModuleInfo {
    fn register(modules: &mut Builder<'_>);
}

#[macro_export]
macro_rules! module_info {
    ($name: literal => $module: ident) => {
        impl $crate::modules::ModuleInfo for $module {
            fn register(mut modules: &mut $crate::modules::Builder<'_>) {
                modules.register::<$module>($name);
            }
        }
    };
}
