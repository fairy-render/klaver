use relative_path::{RelativePath, RelativePathBuf};
use rquickjs::Ctx;

use samling::FileStore;

use super::loader::Resolver;

pub struct FileResolver<T> {
    fs: T,
}

impl<T> FileResolver<T> {
    pub fn new(fs: T) -> FileResolver<T> {
        FileResolver { fs }
    }
}

impl<T> FileResolver<T>
where
    T: FileStore,
{
    fn _resolve(&self, base: &str, name: &str) -> rquickjs::Result<RelativePathBuf> {
        let path = if !name.starts_with('.') {
            let path = RelativePathBuf::from(name);
            if self.fs.exists(&path) {
                Ok(path)
            } else {
                Err(rquickjs::Error::new_resolving(base, name))
            }
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
                if self.fs.exists(&path) {
                    Ok(path)
                } else {
                    Err(rquickjs::Error::new_resolving(base, name))
                }
            } else {
                if self.fs.exists(&path) {
                    Ok(path)
                } else {
                    Err(rquickjs::Error::new_resolving(base, name))
                }
            }
        }?;

        tracing::trace!(base = %base, name = %name, path = %path, "resolved path");

        Ok(path)
    }
}

impl<T> Resolver for FileResolver<T>
where
    T: FileStore,
{
    fn resolve<'js>(&self, _ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String> {
        let path = self._resolve(base, name)?;
        Ok(path.to_string())
    }
}
