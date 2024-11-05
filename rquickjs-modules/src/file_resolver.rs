use crate::loader::Resolver;
use oxc_resolver::ResolveOptions;
use std::path::{Path, PathBuf};
use tracing::trace;

pub struct ModuleResolver {
    resolver: oxc_resolver::Resolver,
    work_dir: PathBuf,
}

impl ModuleResolver {
    pub fn new_with(work_dir: PathBuf, options: ResolveOptions) -> ModuleResolver {
        ModuleResolver {
            resolver: oxc_resolver::Resolver::new(options),
            work_dir,
        }
    }

    // pub fn new() -> ModuleResolver {
    //     Self::new_with(
    //         std::env::current_dir().unwrap(),
    //         ResolveOptions {
    //             #[cfg(feature = "transform")]
    //             extensions: vec![
    //                 ".js".to_string(),
    //                 ".ts".to_string(),
    //                 ".tsx".to_string(),
    //                 ".jsx".to_string(),
    //             ],
    //             ..Default::default()
    //         },
    //     )
    // }
}

impl ModuleResolver {}

impl Resolver for ModuleResolver {
    fn resolve<'js>(
        &self,
        _ctx: &rquickjs::Ctx<'js>,
        base: &str,
        name: &str,
    ) -> rquickjs::Result<String> {
        let parent = if base == "" {
            self.work_dir.as_path()
        } else {
            Path::new(base).parent().expect("parent")
        };

        let resolution = self
            .resolver
            .resolve(parent, name)
            .map_err(|err| rquickjs::Error::new_resolving_message(base, name, err.to_string()))?;

        trace!(base = %base, name = %name, path = ?resolution.full_path(), "Resolved path");

        Ok(resolution.full_path().display().to_string())
    }
}
