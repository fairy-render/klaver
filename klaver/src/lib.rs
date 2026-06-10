use std::path::PathBuf;

#[cfg(feature = "swc")]
use klaver_modules::loaders::{SwcCompilerOptions, SwcDecocators, SwcTransformer};
use klaver_modules::{
    GlobalInfo, ModuleInfo,
    loaders::FileLoader,
    resolvers::{FileResolver, ResolveOptions},
};

use klaver_vm::Options;
use klaver_wintertc::TokioBackend;
use rquickjs::CatchResultExt;

#[derive(Default)]
pub struct Builder {
    opts: Options,
    resolver_opts: Option<ResolveOptions>,
    search_paths: Vec<PathBuf>,
}

impl Builder {
    pub fn search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    pub fn resolve_options(mut self, options: ResolveOptions) -> Self {
        self.resolver_opts = Some(options);
        self
    }

    pub fn module<M: ModuleInfo>(self) -> Self {
        Self {
            opts: self.opts.module::<M>(),
            resolver_opts: self.resolver_opts,
            search_paths: self.search_paths,
        }
    }

    pub fn global<G: GlobalInfo>(self) -> Self {
        Self {
            opts: self.opts.global::<G>(),
            resolver_opts: self.resolver_opts,
            search_paths: self.search_paths,
        }
    }

    #[allow(unused_mut)]
    pub async fn build(self) -> klaver_vm::Result<Vm> {
        let mut opts = self.opts;

        let resolver_opts = self.resolver_opts.unwrap_or_default();

        for path in self.search_paths {
            let path = path.canonicalize().expect("path does not exists");
            let file_resolver = FileResolver::new_with(path.clone(), resolver_opts.clone());
            opts = opts.resolver(file_resolver);
        }

        let mut file_loader = FileLoader::default();

        #[cfg(feature = "swc")]
        {
            let swc_transformer = SwcTransformer::new_with(SwcCompilerOptions {
                decorators: SwcDecocators::Legacy,
                async_context: false,
                explicit_resource_management: true,
            });

            file_loader.add_transformer(swc_transformer);
        }

        opts = opts.loader(file_loader);

        let vm = opts.global::<klaver_wintertc::WinterTC>().build().await?;

        vm.async_with(async |ctx| {
            klaver_wintertc::set_backend(&ctx, TokioBackend::default()).catch(&ctx)?;
            Ok(())
        })
        .await?;

        Ok(Vm { vm })
    }
}

pub struct Vm {
    vm: klaver_vm::Vm,
}

impl Vm {
    pub fn new() -> Builder {
        Builder::default()
    }
}

impl std::ops::Deref for Vm {
    type Target = klaver_vm::Vm;

    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}
