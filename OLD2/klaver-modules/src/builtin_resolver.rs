use std::collections::HashSet;

use relative_path::RelativePath;
use rquickjs::{loader::ImportAttributes, Ctx};
use tracing::trace;

use super::loader::Resolver;

/// BuiltinResolver is a resolver that resolves builtin modules.
/// It is used to resolve modules that are built into the runtime, such as "node:fs" or "node:path".
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
}

impl Resolver for BuiltinResolver {
    fn resolve<'js>(
        &self,
        _ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> rquickjs::Result<String> {
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

        if self.modules.contains(&full) {
            trace!(base = %base, name = %name, path = %full, "Resolved builtin module");

            Ok(full)
        } else {
            Err(rquickjs::Error::new_resolving(base, name))
        }
    }
}
