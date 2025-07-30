use klaver_util::rquickjs::{self, Class, JsLifetime, class::Trace};

use crate::{Listener, ScriptListener, listener::HookListeners};

#[rquickjs::class(crate = "rquickjs")]
pub struct AsyncHook<'js> {
    listener: ScriptListener<'js>,
    state: Class<'js, HookListeners<'js>>,
    key: Option<slotmap::DefaultKey>,
}

unsafe impl<'js> JsLifetime<'js> for AsyncHook<'js> {
    type Changed<'to> = AsyncHook<'to>;
}

impl<'js> Trace<'js> for AsyncHook<'js> {
    fn trace<'a>(&self, tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {
        self.listener.trace(tracer);
        self.state.trace(tracer);
    }
}

impl<'js> AsyncHook<'js> {
    pub fn new(
        listener: ScriptListener<'js>,
        state: Class<'js, HookListeners<'js>>,
    ) -> AsyncHook<'js> {
        AsyncHook {
            listener,
            state,
            key: None,
        }
    }
}

#[rquickjs::methods(crate = "rquickjs")]
impl<'js> AsyncHook<'js> {
    pub fn enable(&mut self) -> rquickjs::Result<()> {
        if self.key.is_some() {
            return Ok(());
        }

        let key = self
            .state
            .borrow_mut()
            .add_listener(Listener::Script(self.listener.clone()));

        self.key = Some(key);

        Ok(())
    }

    pub fn disable(&mut self) -> rquickjs::Result<()> {
        if let Some(key) = self.key.take() {
            self.state.borrow_mut().remove_listener(key);
        }

        Ok(())
    }
}
