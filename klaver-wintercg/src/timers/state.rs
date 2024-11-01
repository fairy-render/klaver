use std::{cell::RefCell, ops::Add, rc::Rc, time::Duration};

use rquickjs::{class::Trace, Ctx, FromJs, Function, IntoJs, Value};
use slotmap::{new_key_type, KeyData, SlotMap};
use tokio::time::{Instant, Sleep};

use crate::config::WinterCG;

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

#[derive(Default, Clone)]
pub struct Timers<'js> {
    time_ref: Rc<RefCell<SlotMap<TimeId, TimeRef<'js>>>>,
}

impl<'js> Trace<'js> for Timers<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (_, time) in &*self.time_ref.borrow() {
            time.trace(tracer)
        }
    }
}

impl<'js> Timers<'js> {
    pub fn next_time(&self) -> Instant {
        self.time_ref
            .borrow()
            .values()
            .min_by_key(|m| m.expires)
            .map(|m| m.expires)
            .unwrap_or(Instant::now().add(Duration::from_millis(4)))
    }

    pub fn clear(&self) {
        self.time_ref.borrow_mut().clear();
    }

    pub fn sleep(&self) -> Sleep {
        tokio::time::sleep_until(self.next_time())
    }

    /// Advance timers
    /// Return false if no times is defined
    pub fn process(&self, ctx: &Ctx<'_>) -> rquickjs::Result<bool> {
        let current = Instant::now();

        let ids = self
            .time_ref
            .borrow()
            .iter()
            .filter(|(_, v)| v.expires <= current)
            .map(|m| (m.0, m.1.func.clone(), m.1.repeat))
            .collect::<Vec<_>>();

        for (id, func, repeat) in ids {
            func.call::<_, ()>(())?;
            if !repeat {
                self.time_ref.borrow_mut().remove(id);
            } else {
                let mut time_ref = self.time_ref.borrow_mut();
                time_ref[id].expires = current.add(time_ref[id].duration);
            }
        }

        Ok(!self.time_ref.borrow().is_empty())
    }
}

impl<'js> Timers<'js> {
    pub fn create_timer(
        &self,
        func: Function<'js>,
        timeout: u64,
        repeat: bool,
    ) -> rquickjs::Result<TimeId> {
        let id = self.time_ref.borrow_mut().insert(TimeRef {
            func,
            expires: Instant::now().add(Duration::from_millis(timeout)),
            repeat,
            duration: Duration::from_millis(timeout),
        });

        Ok(id)
    }

    pub fn clear_timer(&self, time_id: TimeId) -> rquickjs::Result<()> {
        self.time_ref.borrow_mut().remove(time_id);
        Ok(())
    }
}

async fn get_timers<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Timers<'js>> {
    let core = WinterCG::get(ctx).await?;
    let core = core.borrow();
    Ok(core.timers().clone())
}

pub async fn poll_timers(ctx: &Ctx<'_>) -> rquickjs::Result<Sleep> {
    let timers = get_timers(ctx).await?;
    Ok(timers.sleep())
}

pub async fn process_timers(ctx: &Ctx<'_>) -> rquickjs::Result<bool> {
    let timers = get_timers(ctx).await?;
    timers.process(ctx)
}
