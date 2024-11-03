use std::path::PathBuf;

use rquickjs_modules::{Environ, GlobalInfo, ModuleInfo, ResolveOptions};
use rquickjs_util::RuntimeError;

use crate::Vm;

pub struct Options {
    pub builder: rquickjs_modules::Builder,
    pub max_stack_size: Option<usize>,
    pub memory_limit: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            builder: rquickjs_modules::Builder::default().global::<rquickjs_wintercg::Globals>(),
            max_stack_size: None,
            memory_limit: None,
        }
    }
}

impl Options {
    pub fn search_path(self, path: impl Into<PathBuf>) -> Self {
        Options {
            builder: self.builder.search_path(path),
            ..self
        }
    }

    pub fn resolve_options(self, options: ResolveOptions) -> Self {
        Options {
            builder: self.builder.resolve_options(options),
            ..self
        }
    }

    #[cfg(feature = "transform")]
    pub fn compiler(self, compiler: rquickjs_modules::transformer::Compiler) -> Self {
        Options {
            builder: self.builder.compiler(compiler),
            ..self
        }
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
        Vm::new_with(&env, None, None).await
    }
}
