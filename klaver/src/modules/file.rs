use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use relative_path::{RelativePath, RelativePathBuf};
use rquickjs::{loader::Resolver, Ctx, Error, Result};

/// The file module resolver
///
/// This resolver can be used as the nested backing resolver in user-defined resolvers.
#[derive(Debug)]
pub struct FileResolver {
    paths: Vec<PathBuf>,
    patterns: Vec<String>,
    cache: HashMap<String, String>,
}

impl FileResolver {
    /// Add search path for modules
    pub fn add_path<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.paths.push(path.into());
        self
    }

    /// Add search path for modules
    #[must_use]
    pub fn with_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.add_path(path);
        self
    }

    /// Add search paths for modules
    pub fn add_paths<I: IntoIterator<Item = P>, P: Into<PathBuf>>(
        &mut self,
        paths: I,
    ) -> &mut Self {
        self.paths.extend(paths.into_iter().map(|path| path.into()));
        self
    }

    /// Add search paths for modules
    #[must_use]
    pub fn with_paths<I: IntoIterator<Item = P>, P: Into<PathBuf>>(mut self, paths: I) -> Self {
        self.add_paths(paths);
        self
    }

    /// Add module file pattern
    pub fn add_pattern<P: Into<String>>(&mut self, pattern: P) -> &mut Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Add module file pattern
    #[must_use]
    pub fn with_pattern<P: Into<String>>(mut self, pattern: P) -> Self {
        self.add_pattern(pattern);
        self
    }

    /// Add support for native modules
    pub fn add_native(&mut self) -> &mut Self {
        #[cfg(target_family = "windows")]
        self.add_pattern("{}.dll");

        #[cfg(target_vendor = "apple")]
        self.add_pattern("{}.dylib").add_pattern("lib{}.dylib");

        #[cfg(target_family = "unix")]
        self.add_pattern("{}.so").add_pattern("lib{}.so");

        self
    }

    /// Add support for native modules
    #[must_use]
    pub fn with_native(mut self) -> Self {
        self.add_native();
        self
    }

    fn try_patterns(&self, path: &RelativePath) -> Option<RelativePathBuf> {
        if let Some(extension) = &path.extension() {
            if !is_file(path) {
                return None;
            }

            // check for known extensions
            self.patterns
                .iter()
                .find(|pattern| {
                    let path = RelativePath::new(pattern);
                    if let Some(known_extension) = &path.extension() {
                        known_extension == extension
                    } else {
                        false
                    }
                })
                .map(|_| path.to_relative_path_buf())
        } else {
            // try with known patterns
            self.patterns.iter().find_map(|pattern| {
                let name = pattern.replace("{}", path.file_name()?);
                let file = path.with_file_name(name);
                if is_file(&file) {
                    Some(file)
                } else {
                    None
                }
            })
        }
    }
}

impl Default for FileResolver {
    fn default() -> Self {
        Self {
            paths: vec![],
            patterns: vec!["{}.js".into()],
            cache: Default::default(),
        }
    }
}

impl Resolver for FileResolver {
    fn resolve<'js>(&mut self, _ctx: &Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let key = format!("{base}_{name}");

        if let Some(path) = self.cache.get(&key) {
            return Ok(path.clone());
        }

        let path = if !name.starts_with('.') {
            self.paths.iter().find_map(|path| {
                let path = RelativePathBuf::from(path.display().to_string()).join_normalized(name);
                self.try_patterns(&path)
            })
        } else {
            let path = RelativePath::new(base);
            let mut path = if let Some(dir) = path.parent() {
                dir.join_normalized(name)
            } else {
                name.into()
            };

            if base.starts_with("/")
                && !<RelativePathBuf as AsRef<str>>::as_ref(&path).starts_with("/")
            {
                path = RelativePathBuf::from(format!("/{path}"));
                self.try_patterns(&path)
            } else {
                self.paths.iter().find_map(|p| {
                    let path = RelativePathBuf::from(p.display().to_string()).join(&path);
                    let ret = self.try_patterns(&path);

                    ret
                })
            }
        }
        .ok_or_else(|| Error::new_resolving(base, name))?;

        tracing::trace!(base = %base, name = %name, path = %path, "resolved path");

        self.cache.insert(key, path.to_string());

        Ok(path.to_string())
    }
}

fn is_file<P: AsRef<str>>(path: P) -> bool {
    Path::new(path.as_ref()).is_file()
}
