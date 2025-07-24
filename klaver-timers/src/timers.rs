use std::{
    collections::HashMap,
    time::{Duration, Instant},
    usize,
};

use event_listener::Event;
use futures::FutureExt;
use klaver_runner::{Shutdown, Workers};
use rquickjs::{CatchResultExt, Class, Ctx, Function, JsLifetime, class::Trace, prelude::Opt};
use rquickjs_util::throw;

use crate::{backend::TimingBackend, id::TimeId};

struct Entry<'js> {
    deadline: Instant,
    callback: Function<'js>,
    repeat: bool,
    timeout: Duration,
}

impl<'js> Trace<'js> for Entry<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.callback.trace(tracer);
    }
}

#[rquickjs::class]
pub struct Timers<'js> {
    entries: HashMap<TimeId, Entry<'js>>,
    next_id: usize,
    event: Event,
}

impl<'js> Trace<'js> for Timers<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.entries.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for Timers<'js> {
    type Changed<'to> = Timers<'to>;
}

impl<'js> Timers<'js> {
    fn next_instant(&self) -> Option<Instant> {
        let mut instant = None;

        for v in self.entries.values() {
            if let Some(i) = instant {
                if v.deadline < i {
                    instant = Some(v.deadline);
                }
            } else {
                instant = Some(v.deadline);
            }
        }

        instant
    }

    fn get_callbacks(
        &mut self,
        instant: Instant,
        callback: &mut Vec<Function<'js>>,
    ) -> rquickjs::Result<()> {
        let mut remove = Vec::new();
        for (id, entry) in self
            .entries
            .iter_mut()
            .filter(|(_, e)| e.deadline == instant)
        {
            // Update entry
            if !entry.repeat {
                remove.push(*id);
            } else {
                entry.deadline = Instant::now() + entry.timeout;
            }

            callback.push(entry.callback.clone());
        }

        for id in remove {
            self.entries.remove(&id);
        }

        Ok(())
    }
}

#[rquickjs::methods]
impl<'js> Timers<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<Class<'js, Timers<'js>>> {
        let timers = Class::instance(
            ctx.clone(),
            Timers {
                entries: Default::default(),
                next_id: 1,
                event: Event::new(),
            },
        )?;

        {
            let workers = Workers::from_ctx(&ctx)?;
            let timers = timers.clone();
            workers.push(ctx.clone(), |ctx, kill| async move {
                work(ctx.clone(), timers, kill).await.catch(&ctx)?;
                Ok(())
            });
        }

        Ok(timers)
    }

    #[qjs(rename = "createTimeout")]
    pub fn create_timeout(
        &mut self,
        ctx: Ctx<'js>,
        callback: Function<'js>,
        timeout: Opt<u64>,
        repeat: Opt<bool>,
    ) -> rquickjs::Result<TimeId> {
        let Some(_) = ctx.userdata::<TimingBackend>() else {
            throw!(@type &ctx, "Timing backend not defined")
        };
        let id = TimeId(self.next_id);
        self.next_id += 1;

        let timeout = Duration::from_millis(timeout.unwrap_or(0));

        let deadline = Instant::now().checked_add(timeout).expect("valid duration");

        self.entries.insert(
            id,
            Entry {
                deadline,
                callback,
                repeat: repeat.unwrap_or_default(),
                timeout,
            },
        );

        self.event.notify(usize::MAX);

        Ok(id)
    }

    #[qjs(rename = "clearTimeout")]
    pub fn clear_timeout(&mut self, id: TimeId) -> rquickjs::Result<()> {
        if self.entries.remove(&id).is_some() {
            self.event.notify(usize::MAX);
        }

        Ok(())
    }
}

pub async fn work<'js>(
    ctx: Ctx<'js>,
    timers: Class<'js, Timers<'js>>,
    mut kill: Shutdown,
) -> rquickjs::Result<()> {
    let mut callbacks = Vec::new();

    loop {
        let notifier = timers.borrow().event.listen();

        let next_instant = timers.borrow().next_instant();
        let Some(next_instant) = next_instant else {
            // Wait for a new timer or the kill signal
            futures::select! {
                _ = kill => {
                    return Ok(())
                }
                _ = notifier.fuse() => {
                    continue
                }
            }
        };

        let Some(backend) = ctx.userdata::<TimingBackend>() else {
            throw!(@type &ctx, "TimingBackend not registered")
        };

        let timer = backend.create_timer(next_instant);
        let should_shutdown = backend.should_shutdown();

        drop(backend);

        if should_shutdown {
            futures::select! {
                _ = timer.fuse() => {


                    timers.borrow_mut().get_callbacks(next_instant, &mut callbacks)?;
                    for callback in callbacks.drain(..) {
                        callback.call::<_,()>(())?;
                    }
                }
                _ =  kill => {
                    return Ok(())
                }
                _ = notifier.fuse() => {

                    continue
                }
            }
        } else {
            futures::select! {
                _ = timer.fuse() => {
                    timers.borrow_mut().get_callbacks(next_instant, &mut callbacks)?;
                    for callback in callbacks.drain(..) {
                        callback.call::<_,()>(())?;
                    }
                }
                _ = notifier.fuse() => {
                    continue
                }
            }
        }
    }
}
