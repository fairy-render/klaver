use klaver_timers::Backend as TimeBackend;
use rquickjs::Ctx;

#[derive(Clone, Default)]
pub struct Tokio {}

impl TimeBackend for Tokio {
    type Timer = tokio::time::Sleep;

    fn create_timer(&self, instant: std::time::Instant) -> Self::Timer {
        tokio::time::sleep_until(instant.into())
    }
}

impl Tokio {
    pub fn set_runtime(&self, ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        klaver_timers::set_backend(ctx, self.clone())?;
        klaver_fetch::set_shared_client(ctx, klaver_fetch::reqwest::Client::new())?;

        Ok(())
    }
}
