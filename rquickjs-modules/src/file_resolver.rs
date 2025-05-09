use crate::loader::Resolver;
use oxc_resolver::ResolveOptions;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
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
}

impl ModuleResolver {}

impl Resolver for ModuleResolver {
    fn resolve<'js>(
        &self,
        _ctx: &rquickjs::Ctx<'js>,
        base: &str,
        name: &str,
    ) -> rquickjs::Result<String> {
        let parent: Cow<'_, Path> = if base.is_empty() {
            self.work_dir.as_path().into()
        } else {
            let path = Path::new(base).parent().expect("parent");
            if !path.is_absolute() {
                self.work_dir.join(path).into()
            } else {
                path.into()
            }
        };

        trace!(parent = ?parent, base = %base, path = %name, "Resolving path");

        let resolution = self
            .resolver
            .resolve(parent, name)
            .map_err(|err| rquickjs::Error::new_resolving_message(base, name, err.to_string()))?;

        trace!(base = %base, name = %name, path = ?resolution.full_path(), "Resolved path");

        Ok(resolution.full_path().display().to_string())
    }
}
