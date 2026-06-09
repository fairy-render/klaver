use std::time::Instant;

use futures::future::LocalBoxFuture;
use klaver_core::throw;
use rquickjs::{Ctx, JsLifetime};

pub trait TimerBackend {
    type Timer: Future<Output = ()>;

    fn create_timer(&self, instant: Instant) -> Self::Timer;
}

trait DynBackend {
    fn create_timer(
        &self,
        ctx: &Ctx<'_>,
        instant: Instant,
    ) -> rquickjs::Result<LocalBoxFuture<'static, ()>>;
}

pub struct TimingBackend {
    backend: Box<dyn DynBackend>,
    shutdown: bool,
}

impl TimingBackend {
    pub fn new<T>(backend: T) -> TimingBackend
    where
        T: TimerBackend + 'static,
        T::Timer: 'static,
    {
        struct Back<T>(T);

        impl<T> DynBackend for Back<T>
        where
            T: TimerBackend,
            T::Timer: 'static,
        {
            fn create_timer(
                &self,
                _ctx: &Ctx<'_>,
                instant: Instant,
            ) -> rquickjs::Result<LocalBoxFuture<'static, ()>> {
                Ok(Box::pin(self.0.create_timer(instant)))
            }
        }

        TimingBackend {
            backend: Box::new(Back(backend)),
            shutdown: false,
        }
    }

    pub fn null() -> TimingBackend {
        struct NullBackend;

        impl DynBackend for NullBackend {
            fn create_timer(
                &self,
                ctx: &Ctx<'_>,
                _instant: Instant,
            ) -> rquickjs::Result<LocalBoxFuture<'static, ()>> {
                throw!(ctx, "Timing backend not defined")
            }
        }

        TimingBackend {
            backend: Box::new(NullBackend),
            shutdown: false,
        }
    }

    pub fn set_should_shutdown(&mut self, on: bool) {
        self.shutdown = on;
    }

    pub fn with_should_shutdown(mut self, on: bool) -> Self {
        self.shutdown = on;
        self
    }

    pub fn should_shutdown(&self) -> bool {
        self.shutdown
    }

    pub fn create_timer(
        &self,
        ctx: &Ctx<'_>,
        instant: Instant,
    ) -> rquickjs::Result<LocalBoxFuture<'static, ()>> {
        self.backend.create_timer(ctx, instant)
    }
}

unsafe impl<'js> JsLifetime<'js> for TimingBackend {
    type Changed<'to> = TimingBackend;
}
