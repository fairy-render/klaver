use std::collections::HashSet;

use relative_path::RelativePath;
use rquickjs::Ctx;
use tracing::trace;

use super::loader::Resolver;

#[derive(Debug, Default)]
pub struct BuiltinResolver {
    modules: HashSet<String>,
}

impl BuiltinResolver {
    /// Add builtin module
    pub fn add_module<P: Into<String>>(&mut self, path: P) -> &mut Self {
        self.modules.insert(path.into());
        self
    }

    // /// Add builtin module
    // #[must_use]
    // pub fn with_module<P: Into<String>>(mut self, path: P) -> Self {
    //     self.add_module(path);
    //     self
    // }
}

impl Resolver for BuiltinResolver {
    fn resolve<'js>(&self, _ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String> {
        let full = if !name.starts_with('.') {
            name.to_string()
        } else {
            let base = RelativePath::new(base);
            if let Some(dir) = base.parent() {
                dir.join_normalized(name).to_string()
            } else {
                name.to_string()
            }
        };

        trace!(base = %base, name = %name, path = %full, "Resolved builtin module");

        if self.modules.contains(&full) {
            Ok(full)
        } else {
            Err(rquickjs::Error::new_resolving(base, name))
        }
    }
}
