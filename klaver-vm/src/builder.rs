use std::path::PathBuf;

use klaver_modules::{Environ, GlobalInfo, ModuleInfo, ResolveOptions, transformer::Transpiler};
use rquickjs_util::RuntimeError;

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

    pub fn transpiler<T: Transpiler + 'static>(self, compiler: T) -> Self {
        Options {
            builder: self.builder.transpiler(compiler),
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
