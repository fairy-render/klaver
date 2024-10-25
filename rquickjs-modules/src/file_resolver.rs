use std::path::{Path, PathBuf};

use oxc_resolver::{Alias, AliasValue, ResolveOptions};
use relative_path::{RelativePath, RelativePathBuf};
use rquickjs::loader::Resolver;

pub struct ModuleResolver {
    resolver: oxc_resolver::Resolver,
    work_dir: PathBuf,
}

impl ModuleResolver {
    pub fn new() -> ModuleResolver {
        let options = ResolveOptions {
            alias: vec![(
                "@".to_string(),
                vec![AliasValue::Path(
                    std::env::current_dir()
                        .unwrap()
                        .join("rquickjs-modules/examples/nested/")
                        .display()
                        .to_string(),
                )],
            )],
            extensions: vec![
                ".js".to_string(),
                ".ts".to_string(),
                ".tsx".to_string(),
                ".jsx".to_string(),
            ],
            ..Default::default()
        };

        ModuleResolver {
            resolver: oxc_resolver::Resolver::new(options),
            work_dir: std::env::current_dir().unwrap(),
        }
    }
}

impl ModuleResolver {}

impl Resolver for ModuleResolver {
    fn resolve<'js>(
        &mut self,
        ctx: &rquickjs::Ctx<'js>,
        base: &str,
        name: &str,
    ) -> rquickjs::Result<String> {
        let parent = if base == "" {
            self.work_dir.as_path()
        } else {
            Path::new(base).parent().unwrap()
        };

        let resolution = self
            .resolver
            .resolve(parent, name)
            .map_err(|err| rquickjs::Error::new_resolving_message(base, name, err.to_string()))?;

        Ok(resolution.full_path().display().to_string())
    }
}
