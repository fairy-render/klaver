use std::io::Read;

use relative_path::RelativePath;
use samling::{File, FileStore};

#[cfg(not(feature = "typescript"))]
#[derive(Debug, Clone)]
pub struct FileLoader<T> {
    fs: T,
}

pub fn load<T: FileStore>(fs: &T, name: &str) -> rquickjs::Result<Vec<u8>> {
    let path = RelativePath::new(name);

    let meta = fs
        .metadata(path)
        .map_err(|err| rquickjs::Error::new_loading_message(name, err.to_string()))?;

    let file = fs
        .open_file(path)
        .map_err(|err| rquickjs::Error::new_loading_message(name, err.to_string()))?;

    let mut reader = file
        .reader()
        .map_err(|err| rquickjs::Error::new_loading_message(name, err.to_string()))?;

    let mut content = Vec::with_capacity(meta.size as usize);
    reader.read_to_end(&mut content)?;
    Ok(content)
}

#[cfg(not(feature = "typescript"))]
impl<T> FileLoader<T> {
    pub(crate) fn new(fs: T) -> FileLoader<T> {
        FileLoader { fs }
    }
}

#[cfg(not(feature = "typescript"))]
impl<T> super::loader::Loader for FileLoader<T>
where
    T: FileStore,
{
    fn load<'js>(
        &self,
        ctx: &rquickjs::prelude::Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::module::Declared>> {
        let content = load(&self.fs, name)?;
        rquickjs::Module::declare(ctx.clone(), name, content)
    }
}
