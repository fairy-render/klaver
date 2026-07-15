use std::sync::Arc;

use klaver_core::{Core, throw_if};
use rquickjs::{Class, Ctx, JsLifetime, class::Trace};

use crate::backend::Backend;
#[cfg(feature = "fetch")]
use crate::fetch::{Client, LocalClient, SharedClient};
#[cfg(feature = "fs")]
use crate::fs::FileSystemSettings;
#[cfg(feature = "timers")]
use crate::timers::TimingBackend;

#[rquickjs::class]
pub struct WinterTcInstance {
    settings: Settings,
    backend: Arc<dyn Backend + Send + Sync>,
}

impl WinterTcInstance {
    pub fn new(settings: Settings, backend: Arc<dyn Backend + Send + Sync>) -> Self {
        WinterTcInstance { settings, backend }
    }

    pub(crate) fn register(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        let core = Core::from_ctx(ctx)?;
        let settings = Settings::default();
        let backend = Arc::new(());
        let instance = WinterTcInstance::new(settings, backend);
        core.borrow_mut()
            .register("WinterTC", Class::instance(ctx.clone(), instance)?)?;
        Ok(())
    }

    pub fn from_ctx<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, WinterTcInstance>> {
        let core = Core::from_ctx(ctx)?;
        if !core.borrow().has("WinterTC")? {
            throw_if!(ctx, WinterTcInstance::register(ctx));
        }
        let settings: Class<'js, WinterTcInstance> = core.borrow().get("WinterTC")?;
        Ok(settings)
    }

    pub fn set_backend(
        &mut self,
        ctx: &Ctx<'_>,
        backend: Arc<dyn Backend + Send + Sync>,
    ) -> rquickjs::Result<()> {
        self.backend = backend;
        throw_if!(ctx, self.backend.init(ctx, &mut self.settings));
        Ok(())
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub(crate) fn backend(&self) -> &Arc<dyn Backend + Send + Sync> {
        &self.backend
    }
}

impl<'js> Trace<'js> for WinterTcInstance {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for WinterTcInstance {
    type Changed<'to> = WinterTcInstance;
}

pub struct Settings {
    #[cfg(feature = "fetch")]
    http_client: Client,
    #[cfg(feature = "timers")]
    timers: TimingBackend,
    #[cfg(feature = "fs")]
    file_system: FileSystemSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            #[cfg(feature = "fetch")]
            http_client: Client::new(),
            #[cfg(feature = "timers")]
            timers: TimingBackend::null(),
            #[cfg(feature = "fs")]
            file_system: FileSystemSettings::default(),
        }
    }
}

impl Settings {
    #[cfg(feature = "fetch")]
    pub fn set_http_client<T: SharedClient + 'static>(&mut self, client: T) {
        self.http_client.set_shared_client(client);
    }

    #[cfg(feature = "fetch")]
    pub fn set_local_http_client<T: LocalClient + 'static>(&mut self, client: T) {
        self.http_client.set_local_client(client);
    }

    #[cfg(feature = "fetch")]
    pub fn get_http_client(&self) -> &Client {
        &self.http_client
    }

    #[cfg(feature = "timers")]
    pub fn set_timers<B: crate::timers::TimerBackend + 'static>(&mut self, timers: B) {
        self.timers = TimingBackend::new(timers);
    }

    #[cfg(feature = "timers")]
    pub fn timers(&self) -> &TimingBackend {
        &self.timers
    }

    #[cfg(feature = "fs")]
    pub fn set_file_system(&mut self, fs: FileSystemSettings) {
        self.file_system = fs;
    }

    #[cfg(feature = "fs")]
    pub fn file_system(&self) -> &FileSystemSettings {
        &self.file_system
    }
}
