use rquickjs::module::ModuleDef;

use super::ModulesBuilder;

pub struct Builder<'a, 'b> {
    modules: &'a mut ModulesBuilder<'b>,
}

impl<'a, 'b> Builder<'a, 'b> {
    pub fn new(modules: &'a mut ModulesBuilder<'b>) -> Builder<'a, 'b> {
        Builder { modules }
    }

    pub fn register<T: ModuleDef>(&mut self, name: impl ToString) {
        self.modules
            .modules
            .insert(name.to_string(), ModulesBuilder::<'a>::load_func::<T>);
    }

    pub fn register_src(&mut self, name: impl ToString, source: Vec<u8>) -> &mut Self {
        self.modules.register_src(name, source);
        self
    }
}

pub trait ModuleInfo {
    fn register(modules: &mut Builder<'_, '_>);
}

#[macro_export]
macro_rules! module_info {
    ($name: literal => $module: ident) => {
        impl $crate::modules::ModuleInfo for $module {
            fn register(mut modules: &mut $crate::modules::Builder<'_, '_>) {
                modules.register::<$module>($name);
            }
        }
    };
}
