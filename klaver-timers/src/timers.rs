use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use klaver_runtime::{AsyncState, Resource, ResourceId, TaskHandle};
use klaver_util::throw;
use rquickjs::{Class, Ctx, Function, JsLifetime, class::Trace, prelude::Opt};

use crate::{backend::TimingBackend, id::TimeId};

#[rquickjs::class]
pub struct Timers {
    entries: HashMap<TimeId, TaskHandle>,
}

impl<'js> Trace<'js> for Timers {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for Timers {
    type Changed<'to> = Timers;
}

#[rquickjs::methods]
impl Timers {
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>) -> rquickjs::Result<Class<'js, Timers>> {
        let timers = Class::instance(
            ctx.clone(),
            Timers {
                entries: Default::default(),
            },
        )?;

        Ok(timers)
    }

    #[qjs(rename = "createTimeout")]
    pub fn create_timeout<'js>(
        &mut self,
        ctx: Ctx<'js>,
        callback: Function<'js>,
        timeout: Opt<u64>,
        repeat: Opt<bool>,
    ) -> rquickjs::Result<TimeId> {
        let Some(_) = ctx.userdata::<TimingBackend>() else {
            throw!(@type &ctx, "Timing backend not defined")
        };

        let timeout = Duration::from_millis(timeout.unwrap_or(0));

        let task_handle = AsyncState::push(
            &ctx,
            TimeoutResource {
                timeout,
                repeat: repeat.0.unwrap_or_default(),
                callback,
            },
        )?;

        let id = TimeId(task_handle.id());

        self.entries.insert(id, task_handle);

        Ok(id)
    }

    #[qjs(rename = "clearTimeout")]
    pub fn clear_timeout(&mut self, id: TimeId) -> rquickjs::Result<()> {
        if let Some(ret) = self.entries.remove(&id) {
            ret.kill()
        }

        Ok(())
    }
}

struct TimeoutResourceId;

impl ResourceId for TimeoutResourceId {
    fn name() -> &'static str {
        "Timeout"
    }
}

struct TimeoutResource<'js> {
    timeout: Duration,
    repeat: bool,
    callback: Function<'js>,
}

impl<'js> Resource<'js> for TimeoutResource<'js> {
    type Id = TimeoutResourceId;

    const INTERNAL: bool = false;
    const SCOPED: bool = false;

    async fn run(self, ctx: klaver_runtime::Context<'js>) -> rquickjs::Result<()> {
        loop {
            let timeout = {
                let Some(backend) = ctx.userdata::<TimingBackend>() else {
                    throw!(@type &ctx, "TimingBackend not registered")
                };
                backend.create_timer(Instant::now() + self.timeout)
            };

            let _ = timeout.await;

            ctx.invoke_callback::<_, ()>(self.callback.clone(), ())?;

            if !self.repeat {
                break;
            }
        }

        Ok(())
    }
}
