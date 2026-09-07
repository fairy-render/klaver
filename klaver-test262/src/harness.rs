use std::{collections::HashMap, fs, path::PathBuf};

use crate::meta::Frontmatter;

/// Loads and caches harness helper scripts (`test262/harness/*.js`) and composes the final
/// source text to execute for a given test case + mode, per
/// https://github.com/tc39/test262/blob/main/INTERPRETING.md#harness
pub struct Harness {
    dir: PathBuf,
    cache: HashMap<String, String>,
}

impl Harness {
    pub fn new(dir: PathBuf) -> Self {
        Harness {
            dir,
            cache: HashMap::new(),
        }
    }

    fn get(&mut self, name: &str) -> anyhow::Result<&str> {
        if !self.cache.contains_key(name) {
            let path = self.dir.join(name);
            let content = fs::read_to_string(&path)
                .map_err(|err| anyhow::anyhow!("reading harness file {path:?}: {err}"))?;
            self.cache.insert(name.to_string(), content);
        }
        Ok(self.cache.get(name).unwrap())
    }

    /// Composes the full source to evaluate for one (test, strict-mode) execution.
    pub fn compose(
        &mut self,
        meta: &Frontmatter,
        body: &str,
        strict: bool,
    ) -> anyhow::Result<String> {
        if meta.is_raw() {
            // Raw tests are evaluated completely as-is: no strict prologue, no harness.
            return Ok(body.to_string());
        }

        let mut out = String::new();

        if strict {
            out.push_str("\"use strict\";\n");
        }

        out.push_str(self.get("assert.js")?);
        out.push('\n');
        out.push_str(self.get("sta.js")?);
        out.push('\n');

        if meta.is_async() {
            out.push_str(self.get("doneprintHandle.js")?);
            out.push('\n');
        }

        for include in &meta.includes {
            out.push_str(self.get(include)?);
            out.push('\n');
        }

        out.push_str(body);

        Ok(out)
    }
}
