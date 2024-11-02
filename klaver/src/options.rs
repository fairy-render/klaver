use std::path::PathBuf;

use rquickjs_modules::{GlobalInfo, ModuleInfo, ResolveOptions};
use rquickjs_util::RuntimeError;

use crate::Vm;

pub struct Options {
    builder: rquickjs_modules::Builder,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            builder: rquickjs_modules::Builder::default().global::<rquickjs_wintercg::Globals>(),
        }
    }
}

impl Options {
    pub fn search_path(self, path: impl Into<PathBuf>) -> Self {
        Options {
            builder: self.builder.search_path(path),
        }
    }

    pub fn resolve_options(self, options: ResolveOptions) -> Self {
        Options {
            builder: self.builder.resolve_options(options),
        }
    }

    #[cfg(feature = "transform")]
    pub fn compiler(self, compiler: rquickjs_modules::transformer::Compiler) -> Self {
        Options {
            builder: self.builder.compiler(compiler),
        }
    }

    pub fn module<M: ModuleInfo>(self) -> Self {
        Options {
            builder: self.builder.module::<M>(),
        }
    }

    pub fn global<G: GlobalInfo>(self) -> Self {
        Options {
            builder: self.builder.global::<G>(),
        }
    }

    pub async fn build(self) -> Result<Vm, RuntimeError> {
        let env = self.builder.build();
        Vm::new_with(&env, None, None).await
    }
}
