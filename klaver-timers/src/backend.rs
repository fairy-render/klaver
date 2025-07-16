use std::time::Instant;

use futures::future::LocalBoxFuture;
use rquickjs::JsLifetime;

pub trait Backend {
    type Timer: Future<Output = ()>;

    fn create_timer(&self, instant: Instant) -> Self::Timer;
}

trait DynBackend {
    fn create_timer(&self, instant: Instant) -> LocalBoxFuture<'static, ()>;
}

pub struct TimingBackend {
    backend: Box<dyn DynBackend>,
    shutdown: bool,
}

impl TimingBackend {
    pub fn new<T>(backend: T) -> TimingBackend
    where
        T: Backend + 'static,
        T::Timer: 'static,
    {
        struct Back<T>(T);

        impl<T> DynBackend for Back<T>
        where
            T: Backend,
            T::Timer: 'static,
        {
            fn create_timer(&self, instant: Instant) -> LocalBoxFuture<'static, ()> {
                Box::pin(self.0.create_timer(instant))
            }
        }

        TimingBackend {
            backend: Box::new(Back(backend)),
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

    pub fn create_timer(&self, instant: Instant) -> LocalBoxFuture<'static, ()> {
        self.backend.create_timer(instant)
    }
}

unsafe impl<'js> JsLifetime<'js> for TimingBackend {
    type Changed<'to> = TimingBackend;
}
