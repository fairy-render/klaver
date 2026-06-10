use std::path::PathBuf;

use klaver_core::RuntimeError;
use klaver_modules::{Environ, GlobalInfo, Loader, ModuleInfo, Resolver};

use crate::{Vm, VmOptions};

pub struct Options {
    pub builder: klaver_modules::Builder,
    pub max_stack_size: Option<usize>,
    pub memory_limit: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            builder: klaver_modules::Builder::default(),
            max_stack_size: None,
            memory_limit: None,
        }
    }
}

impl Options {
    /// Add a custom loader to the environment.
    /// Loaders are responsible for loading modules from various sources.
    pub fn loader<L: Loader + Send + Sync + 'static>(mut self, loader: L) -> Self {
        self.builder = self.builder.loader(loader);
        self
    }

    /// Add a custom resolver to the environment.
    /// Resolvers are responsible for resolving module names to their corresponding modules.
    pub fn resolver<R: Resolver + Send + Sync + 'static>(mut self, resolver: R) -> Self {
        self.builder = self.builder.resolver(resolver);
        self
    }

    pub fn module<M: ModuleInfo>(self) -> Self {
        Options {
            builder: self.builder.module::<M>(),
            ..self
        }
    }

    pub fn global<G: GlobalInfo>(self) -> Self {
        Options {
            builder: self.builder.global::<G>(),
            ..self
        }
    }

    pub fn build_environ(self) -> Environ {
        self.builder.build()
    }

    pub async fn build(self) -> Result<Vm, RuntimeError> {
        let env = self.builder.build();
        Vm::new(
            &env,
            VmOptions {
                max_stack_size: self.max_stack_size,
                memory_limit: self.memory_limit,
            },
        )
        .await
    }
}
