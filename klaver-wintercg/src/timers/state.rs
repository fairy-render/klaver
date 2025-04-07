use std::{cell::RefCell, ops::Add, rc::Rc, sync::Arc, time::Duration};

use futures::channel::oneshot;
use rquickjs::{class::Trace, CatchResultExt, Ctx, FromJs, Function, IntoJs, Value};
use rquickjs_util::RuntimeError;
use slotmap::{new_key_type, KeyData, SlotMap};
use tokio::time::{Instant, Sleep};

use crate::config::WinterCG;

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

#[derive(Debug)]
struct TimeRef<'js> {
    func: Function<'js>,
    expires: Instant,
    repeat: bool,
    duration: Duration,
}

impl<'js> Trace<'js> for TimeRef<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.func.trace(tracer)
    }
}

#[derive(Clone)]
pub struct Timers {
    time_ref: Rc<RefCell<SlotMap<TimeId, oneshot::Sender<()>>>>,
    err_rx: Rc<RefCell<tokio::sync::mpsc::UnboundedReceiver<RuntimeError>>>,
    err_sx: tokio::sync::mpsc::UnboundedSender<RuntimeError>,
}

impl Default for Timers {
    fn default() -> Self {
        let (err_sx, err_rx) = tokio::sync::mpsc::unbounded_channel();
        Timers {
            time_ref: Default::default(),
            err_rx: Rc::new(RefCell::new(err_rx)),
            err_sx,
        }
    }
}

impl<'js> Trace<'js> for Timers {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        // for (_, time) in &*self.time_ref.borrow() {
        //     time.trace(tracer)
        // }
    }
}

// impl Timers {
//     pub fn next_time(&self) -> Instant {
//         self.time_ref
//             .borrow()
//             .values()
//             .min_by_key(|m| m.expires)
//             .map(|m| m.expires)
//             .unwrap_or(Instant::now().add(Duration::from_millis(0)))
//     }

//     pub fn clear(&self) {
//         self.time_ref.borrow_mut().clear();
//     }

//     pub fn sleep(&self) -> Sleep {
//         tokio::time::sleep_until(self.next_time())
//     }

//     /// Advance timers
//     /// Return false if no times is defined
//     pub fn process(&self, _ctx: &Ctx<'_>) -> rquickjs::Result<bool> {
//         let current = Instant::now();

//         let ids = self
//             .time_ref
//             .borrow()
//             .iter()
//             .filter(|(_, v)| v.expires <= current)
//             .map(|m| (m.0, m.1.func.clone(), m.1.repeat))
//             .collect::<Vec<_>>();

//         for (id, func, repeat) in ids {
//             func.call::<_, ()>(())?;
//             if !repeat {
//                 self.time_ref.borrow_mut().remove(id);
//             } else {
//                 let mut time_ref = self.time_ref.borrow_mut();
//                 time_ref[id].expires = current.add(time_ref[id].duration);
//             }
//         }

//         Ok(!self.time_ref.borrow().is_empty())
//     }
// }

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

        let err_sx = self.err_sx.clone();
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
            chan: self.err_rx.clone(),
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
    chan: Rc<RefCell<tokio::sync::mpsc::UnboundedReceiver<RuntimeError>>>,
}

impl TimeErrorChan {
    pub async fn wait(&self) -> Option<RuntimeError> {
        self.chan.borrow_mut().recv().await
    }
}
