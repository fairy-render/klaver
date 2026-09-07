use serde::Deserialize;

/// The YAML frontmatter test262 embeds in every test file, delimited by `/*---` and `---*/`.
/// See https://github.com/tc39/test262/blob/main/INTERPRETING.md
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Frontmatter {
    pub negative: Option<Negative>,
    pub includes: Vec<String>,
    pub flags: Vec<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Negative {
    #[allow(dead_code)]
    pub phase: String,
    #[serde(rename = "type")]
    pub kind: String,
}

impl Frontmatter {
    fn has_flag(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }

    pub fn is_raw(&self) -> bool {
        self.has_flag("raw")
    }

    pub fn is_module(&self) -> bool {
        self.has_flag("module")
    }

    pub fn is_async(&self) -> bool {
        self.has_flag("async")
    }

    pub fn only_strict(&self) -> bool {
        self.has_flag("onlyStrict")
    }

    pub fn no_strict(&self) -> bool {
        self.has_flag("noStrict")
    }

    /// The distinct `strict` values this test must be run under.
    pub fn modes(&self) -> &'static [bool] {
        if self.is_raw() || self.is_module() {
            // `raw` is never wrapped in a strict-mode prologue; module code is always strict
            // implicitly, so there is only one meaningful mode.
            &[false]
        } else if self.only_strict() {
            &[true]
        } else if self.no_strict() {
            &[false]
        } else {
            &[false, true]
        }
    }
}

/// Extracts and parses the `/*--- ... ---*/` YAML block from a test262 source file. Returns
/// the default (empty) frontmatter if the file has none, which is valid for a handful of
/// harness-adjacent files.
pub fn parse_frontmatter(source: &str) -> Frontmatter {
    let Some(start) = source.find("/*---") else {
        return Frontmatter::default();
    };
    let rest = &source[start + 5..];
    let Some(end) = rest.find("---*/") else {
        return Frontmatter::default();
    };

    serde_yaml::from_str(&rest[..end]).unwrap_or_default()
}
