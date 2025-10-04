use std::path::PathBuf;

use klaver_modules::{GlobalInfo, ModuleInfo, ResolveOptions};
use klaver_vm::Options;
use rquickjs::CatchResultExt;

pub use klaver_vm::async_with;

#[derive(Default)]
pub struct Builder {
    opts: Options,
}

impl Builder {
    pub fn search_path(self, path: impl Into<PathBuf>) -> Self {
        Self {
            opts: self.opts.search_path(path),
        }
    }

    pub fn resolve_options(self, options: ResolveOptions) -> Self {
        Self {
            opts: self.opts.resolve_options(options),
        }
    }

    pub fn module<M: ModuleInfo>(self) -> Self {
        Self {
            opts: self.opts.module::<M>(),
        }
    }

    pub fn global<G: GlobalInfo>(self) -> Self {
        Self {
            opts: self.opts.global::<G>(),
        }
    }

    #[allow(unused_mut)]
    pub async fn build(self) -> klaver_vm::Result<Vm> {
        let mut opts = self.opts;

        #[cfg(feature = "swc")]
        {
            opts = opts.transpiler(klaver_modules::transformer::SwcTranspiler::new_with(
                klaver_modules::transformer::swc::CompilerOptions {
                    decorators: klaver_modules::transformer::swc::Decorators::Legacy,
                    async_context: false,
                    explicit_resource_management: true,
                },
            ));
        }

        let vm = opts
            .search_path(".")
            .global::<klaver_wintertc::WinterCG>()
            .build()
            .await?;

        klaver_vm::async_with!(vm => |ctx| {
            klaver_wintertc::backend::Tokio::default().set_runtime(&ctx).catch(&ctx)?;
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
