use std::{borrow::Cow, marker::PhantomData};

use crate::{module_info::ModuleInfo, modules_builder::ModulesBuilder, types::Typings};

pub struct ModuleBuilder<'a, M> {
    modules: &'a mut ModulesBuilder,
    typings: &'a mut Vec<Typings>,
    module: PhantomData<M>,
}

impl<'a, M: GlobalInfo> ModuleBuilder<'a, M> {
    pub fn new(
        modules: &'a mut ModulesBuilder,
        typings: &'a mut Vec<Typings>,
    ) -> ModuleBuilder<'a, M> {
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
            self.typings.push(Typings {
                name: Cow::Borrowed(T::NAME),
                typings,
            });
        }
    }

    // pub fn register<T: ModuleDef>(&mut self) {
    //     self.modules
    //         .modules
    //         .insert(M::NAME.to_string(), ModulesBuilder::load_func::<T>);
    // }

    pub fn register_source(&mut self, source: Vec<u8>) -> &mut Self {
        self.modules.register_source(M::NAME.to_string(), source);
        self
    }
}

pub trait GlobalInfo {
    fn register(&self);
    fn typings(&self) -> Option<Typings>;
}
