use futures::future::LocalBoxFuture;
use relative_path::{RelativePath, RelativePathBuf};
use vfs::boxed::LocalBoxVPath;

pub trait FileSystemBackend {
    fn open<'a>(
        &'a self,
        path: &'a RelativePath,
    ) -> LocalBoxFuture<'a, Result<LocalBoxVPath, vfs::Error>>;
}

#[derive(Default)]
pub struct FileSystemSettings {
    backend: Option<Box<dyn FileSystemBackend>>,
    cwd: Option<RelativePathBuf>,
}

impl FileSystemSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_backend(mut self, backend: Box<dyn FileSystemBackend>) -> Self {
        self.backend = Some(backend);
        self
    }

    pub fn backend(&self) -> Option<&Box<dyn FileSystemBackend>> {
        self.backend.as_ref()
    }

    pub fn with_cwd(mut self, cwd: RelativePathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    pub fn cwd(&self) -> Option<&RelativePathBuf> {
        self.cwd.as_ref()
    }

    pub fn set_cwd(&mut self, cwd: RelativePathBuf) {
        self.cwd = Some(cwd);
    }

    pub async fn open(&self, path: &str) -> Result<LocalBoxVPath, vfs::Error> {
        let Some(backend) = &self.backend else {
            return Err(vfs::Error::new(
                vfs::ErrorKind::Unsupported,
                "No backend configured",
            ));
        };

        let path = if let Some(cwd) = &self.cwd {
            let full_path = cwd.join_normalized(path);
            full_path
        } else {
            RelativePathBuf::from(path)
        };

        backend.open(&path).await
    }
}
