use goerdet::{BlockingSpawner, BoxError, BoxedBlockingSpawner, DynResult};
use klaver_core::{throw, throw_if};
use rquickjs::Ctx;

#[derive(Default)]
pub struct Blocking {
    spawner: Option<BoxedBlockingSpawner>,
}

impl Blocking {
    pub fn new() -> Self {
        Self { spawner: None }
    }

    pub fn set_spawner<T>(&mut self, spawner: T)
    where
        T: BlockingSpawner + Send + Sync + 'static,
        T::Future<DynResult>: Send,
        T::Error: Into<BoxError>,
    {
        self.spawner = Some(BoxedBlockingSpawner::new(spawner));
    }

    pub async fn spawn_blocking<F, R>(&self, ctx: &Ctx<'_>, work: F) -> Result<R, rquickjs::Error>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let Some(spawner) = &self.spawner else {
            throw!(
                ctx,
                "Spawner not set. Please set a spawner before calling spawn_blocking."
            );
        };

        let ret = throw_if!(ctx, spawner.spawn_blocking(work).await);

        Ok(ret)
    }
}
