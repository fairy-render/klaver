use std::sync::{Arc, Mutex};

use rquickjs::{Ctx, JsLifetime};

#[cfg(feature = "fetch")]
use crate::fetch::{Client, SharedClient};
use crate::timers::Backend;
#[cfg(feature = "timers")]
use crate::timers::TimingBackend;

struct SettingsInner {
    #[cfg(feature = "fetch")]
    http_client: Client,
    #[cfg(feature = "timers")]
    timers: Option<TimingBackend>,
}

impl Default for SettingsInner {
    fn default() -> Self {
        SettingsInner {
            #[cfg(feature = "fetch")]
            http_client: Client::new(),
            #[cfg(feature = "timers")]
            timers: None,
        }
    }
}

#[derive(Clone)]
pub struct Settings(Arc<Mutex<SettingsInner>>);

unsafe impl<'js> JsLifetime<'js> for Settings {
    type Changed<'to> = Settings;
}

impl Settings {
    pub fn from_ctx(ctx: &Ctx<'_>) -> rquickjs::Result<Self> {
        if let Some(settings) = ctx.userdata::<Settings>() {
            Ok(settings.clone())
        } else {
            let settings = Settings(Arc::new(Mutex::new(SettingsInner::default())));
            ctx.store_userdata(settings.clone())?;
            Ok(settings)
        }
    }

    #[cfg(feature = "fetch")]
    pub fn set_http_client<T: SharedClient + 'static>(&self, client: T) {
        self.0.lock().unwrap().http_client.set_shared_client(client);
    }

    #[cfg(feature = "timers")]
    pub fn set_timers<B: Backend + 'static>(&mut self, timers: B) {
        self.0.lock().unwrap().timers = Some(TimingBackend::new(timers));
    }
}
