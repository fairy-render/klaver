use crate::{
    GlobalBuilder, GlobalInfo, Globals, Loader, ModuleBuilder, ModuleInfo, Resolver, Typings,
    environ::Environ, environ_builder::EnvBuilder, loader::ModuleLoader, loaders::BuiltinLoader,
    resolvers::BuiltinResolver,
};
#[derive(Default)]
pub struct Builder {
    modules: EnvBuilder,
    typings: Typings,
    resolvers: Vec<Box<dyn Resolver + Send + Sync>>,
    loaders: Vec<Box<dyn Loader + Send + Sync>>,
    cache: bool,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            modules: EnvBuilder::default(),
            typings: Typings::default(),
            resolvers: Vec::default(),
            loaders: Vec::default(),
            cache: false,
        }
    }

    pub fn cache(mut self, on: bool) -> Self {
        self.cache = on;
        self
    }

    // pub fn loader<L: Loader + Send + Sync + 'static>(mut self, loader: L) -> Self {
    //     self.modules.loader(loader);
    //     self
    // }

    pub fn module<M: ModuleInfo>(mut self) -> Self {
        M::register(&mut ModuleBuilder::new(
            &mut self.modules,
            &mut self.typings,
        ));

        if let Some(typings) = M::typings() {
            self.typings.add_module(M::NAME, typings);
        }
        self
    }

    pub fn global<G: GlobalInfo>(mut self) -> Self {
        G::register(&mut GlobalBuilder::new(
            &mut self.modules,
            &mut self.typings,
        ));

        if let Some(typings) = G::typings() {
            self.typings.add_global(typings);
        }

        self
    }

    pub fn build(self) -> Environ {
        let mut resolvers = Vec::<Box<dyn Resolver + Send + Sync>>::default();

        // Builtins resolver
        let mut builtin_resolver = BuiltinResolver::default();

        for module in self
            .modules
            .modules
            .keys()
            .chain(self.modules.modules_src.keys())
        {
            builtin_resolver.add_module(module);
        }

        resolvers.push(Box::new(builtin_resolver));

        // Builtin resolvers
        let mut loaders = Vec::<Box<dyn Loader + Send + Sync>>::default();

        let builtin_loader = BuiltinLoader {
            modules: self.modules.modules,
            modules_src: self.modules.modules_src,
        };

        loaders.push(Box::new(builtin_loader));

        let modules = ModuleLoader::new(resolvers, loaders);

        let globals = Globals::new(self.modules.globals);

        Environ::new(modules, globals, self.typings)
    }
}
