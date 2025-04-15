use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::channel::oneshot;
use rquickjs::{class::Trace, CatchResultExt, CaughtError, Ctx, FromJs, Function, IntoJs, Value};
use rquickjs_util::{RuntimeError, StackTrace};
use slotmap::{new_key_type, KeyData, SlotMap};
use tokio::time::Instant;

use tokio::sync::broadcast;

use super::timer::Timer;

new_key_type! {
  pub struct TimeId;
}

impl<'js> FromJs<'js> for TimeId {
    fn from_js(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Some(number) = value.as_int() {
            Ok(TimeId(KeyData::from_ffi(number as u64)))
        } else {
            Err(rquickjs::Error::new_from_js(value.type_name(), "TimeId"))
        }
    }
}

impl<'js> IntoJs<'js> for TimeId {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self.0.as_ffi() as i32))
    }
}

#[derive(Clone)]
pub struct Timers {
    time_ref: Rc<RefCell<SlotMap<TimeId, oneshot::Sender<()>>>>,
    err_chann: broadcast::Sender<UncaugthException>,
}
impl Default for Timers {
    fn default() -> Self {
        let (err_sx, _) = broadcast::channel(1);
        Timers {
            time_ref: Default::default(),
            err_chann: err_sx,
        }
    }
}

impl<'js> Trace<'js> for Timers {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl Timers {
    pub fn create_timer<'js>(
        &self,
        ctx: Ctx<'js>,
        func: Function<'js>,
        timeout: u64,
        repeat: bool,
    ) -> rquickjs::Result<TimeId> {
        let (sx, rx) = oneshot::channel();

        let id = self.time_ref.borrow_mut().insert(sx);

        let timer = Timer {
            id,
            callback: func,
            repeat,
            duration: Duration::from_millis(timeout),
        };

        let err_sx = self.err_chann.clone();
        let timers = self.time_ref.clone();

        ctx.clone().spawn(async move {
            if let Err(err) = timer.run(rx).await.catch(&ctx) {
                err_sx.send(err.into()).ok();
                //
                tokio::task::yield_now().await;
            }

            timers.borrow_mut().remove(id);
        });

        Ok(id)
    }

    pub fn clear_timer(&self, time_id: TimeId) -> rquickjs::Result<()> {
        if let Some(sx) = self.time_ref.borrow_mut().remove(time_id) {
            sx.send(()).ok();
        }
        Ok(())
    }

    pub fn create_err_chan(&self) -> TimeErrorChan {
        TimeErrorChan {
            chan: self.err_chann.subscribe(),
        }
    }

    pub fn has_timers(&self) -> bool {
        !self.time_ref.borrow().is_empty()
    }
}

// fn get_timers<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Timers> {
//     let core = WinterCG::get(ctx)?;
//     let core = core.borrow();
//     Ok(core.timers().clone())
// }

// pub fn poll_timers(ctx: &Ctx<'_>) -> rquickjs::Result<Sleep> {
//     let timers = get_timers(ctx)?;
//     Ok(timers.sleep())
// }

// pub fn process_timers(ctx: &Ctx<'_>) -> rquickjs::Result<bool> {
//     let timers = get_timers(ctx)?;
//     timers.process(ctx)
// }

pub struct TimeErrorChan {
    chan: broadcast::Receiver<UncaugthException>,
}

impl TimeErrorChan {
    pub async fn wait(&mut self) -> Option<UncaugthException> {
        self.chan.recv().await.ok()
    }
}

#[derive(Debug, Clone)]
pub struct UncaugthException {
    message: Option<String>,
    stack: Vec<StackTrace>,
}

impl<'js> From<CaughtError<'js>> for UncaugthException {
    fn from(value: CaughtError<'js>) -> Self {
        match value {
            CaughtError::Error(err) => UncaugthException {
                message: Some(err.to_string()),
                stack: Default::default(),
            },
            CaughtError::Exception(e) => {
                let stack = if let Some(stack) = e.stack() {
                    let traces = match rquickjs_util::stack_trace::parse(&stack) {
                        Ok(ret) => ret,
                        Err(_err) => Vec::default(),
                    };
                    traces
                } else {
                    Vec::default()
                };
                UncaugthException {
                    message: e.message(),
                    stack,
                }
            }
            CaughtError::Value(e) => UncaugthException {
                message: e.as_string().and_then(|m| m.to_string().ok()),
                stack: Default::default(),
            },
        }
    }
}

impl From<UncaugthException> for RuntimeError {
    fn from(value: UncaugthException) -> Self {
        RuntimeError::Exception {
            message: value.message,
            stack: value.stack,
        }
    }
}
