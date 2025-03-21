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
    types::Typings,
};

#[cfg(feature = "transform")]
use crate::transformer::{Compiler, FileLoader};

#[derive(Default)]
pub struct Builder {
    modules: ModulesBuilder,
    typings: Typings,
    resolve_options: Option<ResolveOptions>,
    search_paths: Vec<PathBuf>,
    #[cfg(feature = "transform")]
    compiler: Option<Compiler>,
    cache: bool,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            modules: ModulesBuilder::default(),
            typings: Typings::default(),
            resolve_options: None,
            search_paths: Vec::default(),
            #[cfg(feature = "transform")]
            compiler: None,
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

    #[cfg(feature = "transform")]
    pub fn compiler(mut self, compiler: Compiler) -> Self {
        self.compiler = Some(compiler);
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

        let mut loaders = Vec::<Box<dyn Loader + Send + Sync>>::default();

        let builtin_loader = BuiltinLoader {
            modules: self.modules.modules,
            modules_src: self.modules.modules_src,
        };

        loaders.push(Box::new(builtin_loader));

        #[cfg(feature = "transform")]
        let cache = {
            let cache = crate::transformer::Cache::default();
            let loader = if let Some(compiler) = self.compiler {
                FileLoader::new(compiler, cache.clone(), self.cache)
            } else {
                FileLoader::new(Compiler::default(), cache.clone(), self.cache)
            };
            loaders.push(Box::new(loader));

            cache
        };

        #[cfg(not(feature = "transform"))]
        {
            let loader = rquickjs::loader::ScriptLoader::default();
            loaders.push(Box::new(crate::loader::QuickWrap::new(loader)))
        }

        #[cfg(feature = "transform")]
        let modules = Modules::new(cache, resolvers, loaders);
        #[cfg(not(feature = "transform"))]
        let modules = Modules::new(resolvers, loaders);
        let globals = Globals::new(self.modules.globals);

        Environ::new(modules, globals, self.typings)
    }
}
