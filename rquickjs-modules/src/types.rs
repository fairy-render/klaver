use relative_path::RelativePathBuf;
use std::{borrow::Cow, collections::HashMap};

pub struct File {
    pub path: RelativePathBuf,
    pub content: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct Typings {
    modules: HashMap<Cow<'static, str>, Cow<'static, str>>,
    globals: Vec<Cow<'static, str>>,
}

impl Typings {
    pub fn add_module(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        content: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.modules.insert(name.into(), content.into());
        self
    }

    pub fn add_global(&mut self, content: impl Into<Cow<'static, str>>) -> &mut Self {
        self.globals.push(content.into());
        self
    }

    pub fn files(&self) -> Vec<File> {
        let mut files = Vec::with_capacity(self.modules.len() * 2 + self.globals.len());
        for (name, module) in &self.modules {
            let mod_path = RelativePathBuf::from(name);
            let pkg_path = mod_path.join("package.json");
            let idx_path = mod_path.join("index.d.ts");

            files.push(File {
                path: pkg_path,
                content: format!(include_str!("./package.json"), name).into(),
            });

            files.push(File {
                path: idx_path,
                content: module.to_owned().to_string().into_bytes(),
            });
        }

        let globals = self.globals.join("\n\n");

        files.push(File {
            path: RelativePathBuf::from("globals.d.ts"),
            content: globals.into_bytes(),
        });

        files
    }
}
