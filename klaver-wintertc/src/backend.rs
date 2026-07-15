use crate::Settings;

pub trait Backend {
    fn init(&self, ctx: &Ctx<'_>, settings: &mut Settings) -> rquickjs::Result<()>;
}

impl Backend for () {
    fn init(&self, _ctx: &Ctx<'_>, _settings: &mut Settings) -> rquickjs::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "tokio")]
mod tokio_backend {
    use futures::future::LocalBoxFuture;
    use relative_path::RelativePath;
    use rquickjs::Ctx;
    use vfs::{VFS, VPathExt, boxed::LocalBoxVPath};

    use crate::{
        Settings,
        backend::Backend,
        fs::{FileSystemBackend, FileSystemSettings},
        timers::TimerBackend,
    };

    #[derive(Default)]
    pub struct TokioBackend;

    impl Backend for TokioBackend {
        fn init(&self, _ctx: &Ctx<'_>, settings: &mut Settings) -> rquickjs::Result<()> {
            settings.set_timers(TokioBackend);
            settings
                .set_file_system(FileSystemSettings::new().with_backend(Box::new(TokioBackend)));
            settings.set_http_client(reqwest::Client::new());
            Ok(())
        }
    }

    impl TimerBackend for TokioBackend {
        type Timer = tokio::time::Sleep;

        fn create_timer(&self, instant: std::time::Instant) -> Self::Timer {
            tokio::time::sleep_until(instant.into())
        }
    }

    impl FileSystemBackend for TokioBackend {
        fn open<'a>(
            &'a self,
            path: &'a RelativePath,
        ) -> LocalBoxFuture<'a, Result<LocalBoxVPath, vfs::Error>> {
            // Implement the logic to open a file using the Tokio backend
            Box::pin(async move {
                //

                let path = tokio::fs::canonicalize(path.as_str()).await?;

                let v = vfs_tokio::FS::new(path).await?.path(".")?;

                Ok(v.local_box())
            })
        }
    }
}
use rquickjs::Ctx;
#[cfg(feature = "tokio")]
pub use tokio_backend::TokioBackend;

#[cfg(feature = "compio")]
mod compio_backend {

    use futures::future::LocalBoxFuture;
    use klaver_core::throw_if;
    use relative_path::RelativePath;
    use rquickjs::Ctx;
    use vfs::{VFS, VPathExt, boxed::LocalBoxVPath};

    use crate::{
        Settings,
        backend::Backend,
        fs::{FileSystemBackend, FileSystemSettings},
        timers::TimerBackend,
    };

    #[derive(Default)]
    pub struct CompioBackend;

    impl Backend for CompioBackend {
        fn init(&self, ctx: &Ctx<'_>, settings: &mut Settings) -> rquickjs::Result<()> {
            settings.set_timers(CompioBackend);
            settings
                .set_file_system(FileSystemSettings::new().with_backend(Box::new(CompioBackend)));
            settings.set_local_http_client(throw_if!(ctx, cyper::Client::new()));

            Ok(())
        }
    }

    pub struct CompioTimer;

    impl TimerBackend for CompioBackend {
        type Timer = LocalBoxFuture<'static, ()>;

        fn create_timer(&self, instant: std::time::Instant) -> Self::Timer {
            Box::pin(compio::time::sleep_until(instant))
        }
    }

    impl FileSystemBackend for CompioBackend {
        fn open<'a>(
            &'a self,
            path: &'a RelativePath,
        ) -> LocalBoxFuture<'a, Result<LocalBoxVPath, vfs::Error>> {
            // Implement the logic to open a file using the Compio backend
            Box::pin(async move {
                //
                let v = vfs_compio::FS::new(std::path::PathBuf::from(path.as_str()))
                    .await?
                    .path(".")?;

                Ok(v.local_box())
            })
        }
    }
}

#[cfg(feature = "compio")]
pub use compio_backend::CompioBackend;
