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
    use rquickjs::Ctx;

    use crate::{Settings, backend::Backend, timers::TimerBackend};

    #[derive(Default)]
    pub struct TokioBackend;

    impl Backend for TokioBackend {
        fn init(&self, _ctx: &Ctx<'_>, settings: &mut Settings) -> rquickjs::Result<()> {
            settings.set_timers(TokioBackend);
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
}
use rquickjs::Ctx;
#[cfg(feature = "tokio")]
pub use tokio_backend::TokioBackend;

#[cfg(feature = "compio")]
mod compio_backend {
    use futures::future::LocalBoxFuture;
    use klaver_core::throw_if;
    use rquickjs::Ctx;

    use crate::{Settings, backend::Backend, timers::TimerBackend};

    pub struct CompioBackend;

    impl Backend for CompioBackend {
        fn init(&self, ctx: &Ctx<'_>, settings: &mut Settings) -> rquickjs::Result<()> {
            settings.set_timers(CompioBackend);
            settings.set_local_http_client(throw_if!(ctx, cyper::Client::new()));
            Ok(())
        }
    }

    impl TimerBackend for CompioBackend {
        type Timer = LocalBoxFuture<'static, ()>;

        fn create_timer(&self, instant: std::time::Instant) -> Self::Timer {
            Box::pin(compio::time::sleep_until(instant))
        }
    }
}

#[cfg(feature = "compio")]
pub use compio_backend::CompioBackend;
