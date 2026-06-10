use std::path::PathBuf;

use oxc_resolver::ResolveOptions;

use crate::{
    builtin_loader::BuiltinLoader,
    builtin_resolver::BuiltinResolver,
    environ::Environ,
    file_resolver::ModuleResolver,
    global_info::{GlobalBuilder, GlobalInfo},
    globals::Globals,
    loader::{Loader, Resolver},
    module_info::{ModuleBuilder, ModuleInfo},
    modules::Modules,
    modules_builder::ModulesBuilder,
    transformer::{Transformer, Transpiler},
    types::Typings,
};

#[derive(Default)]
pub struct Builder {
    modules: ModulesBuilder,
    typings: Typings,
    resolve_options: Option<ResolveOptions>,
    search_paths: Vec<PathBuf>,
    transformer: Option<Transformer>,
    cache: bool,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            modules: ModulesBuilder::default(),
            typings: Typings::default(),
            resolve_options: None,
            search_paths: Vec::default(),
            transformer: None,
            cache: false,
        }
    }

    pub fn search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    pub fn resolve_options(mut self, options: ResolveOptions) -> Self {
        self.resolve_options = Some(options);
        self
    }

    pub fn cache(mut self, on: bool) -> Self {
        self.cache = on;
        self
    }

    pub fn transpiler<T: Transpiler + 'static>(mut self, transpiler: T) -> Self {
        self.transformer = Some(Transformer::new(transpiler));
        self
    }

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
        for path in self.search_paths {
            let path = path.canonicalize().expect("path does not exists");
            let resolver = ModuleResolver::new_with(
                path,
                self.resolve_options.as_ref().cloned().unwrap_or_default(),
            );
            resolvers.push(Box::new(resolver));
        }

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

        if let Some(transformer) = self.transformer.as_ref().cloned() {
            loaders.push(Box::new(transformer));
        } else {
            let loader = rquickjs::loader::ScriptLoader::default();
            loaders.push(Box::new(crate::loader::QuickWrap::new(loader)))
        }

        let modules = Modules::new(self.transformer, resolvers, loaders);

        let globals = Globals::new(self.modules.globals);

        Environ::new(modules, globals, self.typings)
    }
}
