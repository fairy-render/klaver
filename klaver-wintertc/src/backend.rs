use crate::Settings;

pub trait Backend {
    fn init(&self, settings: &mut Settings) -> rquickjs::Result<()>;
}

impl Backend for () {
    fn init(&self, _settings: &mut Settings) -> rquickjs::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "tokio")]
mod tokio_backend {
    use crate::{Settings, backend::Backend, timers::TimerBackend};

    #[derive(Default)]
    pub struct TokioBackend;

    impl Backend for TokioBackend {
        fn init(&self, settings: &mut Settings) -> rquickjs::Result<()> {
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
#[cfg(feature = "tokio")]
pub use tokio_backend::TokioBackend;
