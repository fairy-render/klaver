use std::{
    ops::Add,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rquickjs::{
    class::Trace, function::Func, CatchResultExt, Class, Ctx, FromJs, Function, IntoJs, Object,
    Value,
};
use slotmap::{new_key_type, KeyData, SlotMap};
use tokio::time::{Instant, Sleep};

use crate::Error;

use super::core::get_core;

const CORE_KEY: &str = "$_klvar";

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
        Ok(Value::new_number(ctx.clone(), self.0.as_ffi() as f64))
    }
}

fn get_current_time_millis() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as usize
}

#[derive(Debug)]
struct TimeRef<'js> {
    func: Function<'js>,
    expires: Instant,
    repeat: bool,
}

impl<'js> Trace<'js> for TimeRef<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.func.trace(tracer)
    }
}

#[derive(Default)]
#[rquickjs::class]
pub struct Timers<'js> {
    time_ref: SlotMap<TimeId, TimeRef<'js>>,
}

impl<'js> Trace<'js> for Timers<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (_, time) in &self.time_ref {
            time.trace(tracer)
        }
    }
}

impl<'js> Timers<'js> {
    pub fn next_time(&self) -> Instant {
        self.time_ref
            .values()
            .min_by_key(|m| m.expires)
            .map(|m| m.expires)
            .unwrap_or(Instant::now().add(Duration::from_millis(1)))
    }

    pub fn process(&mut self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        let current = Instant::now();

        let ids = self
            .time_ref
            .iter()
            .filter(|(_, v)| v.expires <= current)
            .map(|m| m.0)
            .collect::<Vec<_>>();

        for id in ids {
            self.time_ref[id].func.call::<_, ()>(())?;
            if !self.time_ref[id].repeat {
                self.time_ref.remove(id);
            }
        }

        Ok(())
    }
}

#[rquickjs::methods]
impl<'js> Timers<'js> {
    pub fn create_timer(
        &mut self,
        func: Function<'js>,
        timeout: u64,
        repeat: bool,
    ) -> rquickjs::Result<TimeId> {
        let id = self.time_ref.insert(TimeRef {
            func,
            expires: Instant::now().add(Duration::from_millis(timeout)),
            repeat,
        });

        Ok(id)
    }

    pub fn clear_timer(&mut self, time_id: TimeId) -> rquickjs::Result<()> {
        self.time_ref.remove(time_id);
        Ok(())
    }
}

pub fn get_timers<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Class<'js, Timers<'js>>> {
    let core = get_core(ctx)?;

    if let Ok(ret) = core.get::<_, Class<'js, Timers>>("timers") {
        return Ok(ret);
    };

    let timers = Class::instance(ctx.clone(), Timers::default())?;

    core.set("timers", timers.clone())?;

    Ok(timers)
}

pub fn poll_timers(ctx: &Ctx<'_>) -> rquickjs::Result<Sleep> {
    let timer = get_timers(ctx)?;
    let timers = timer.borrow();
    Ok(tokio::time::sleep_until(timers.next_time()))
}

pub fn process_timers(ctx: &Ctx<'_>) -> Result<bool, Error> {
    let timer = get_timers(ctx)?;
    let mut timer = timer.borrow_mut();
    timer.process(ctx.clone()).catch(ctx)?;
    Ok(!timer.time_ref.is_empty())
}

pub fn has_timers(ctx: &Ctx<'_>) -> rquickjs::Result<bool> {
    let timer = get_timers(ctx)?;
    let timer = timer.borrow();
    Ok(!timer.time_ref.is_empty())
}

pub fn init<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    globals.set(
        "setTimeout",
        Func::from(move |ctx, cb, delay| {
            let timers = get_timers(&ctx)?;
            let mut timers = timers.borrow_mut();
            timers.create_timer(cb, delay, false)
        }),
    )?;

    globals.set(
        "setInterval",
        Func::from(move |ctx, cb, delay| {
            let timers = get_timers(&ctx)?;
            let mut timers = timers.borrow_mut();
            timers.create_timer(cb, delay, true)
        }),
    )?;

    globals.set(
        "clearInterval",
        Func::from(move |ctx, id| {
            let timers = get_timers(&ctx)?;
            let mut timers = timers.borrow_mut();
            timers.clear_timer(id)
        }),
    )?;

    globals.set(
        "clearTimeout",
        Func::from(move |ctx, id| {
            let timers = get_timers(&ctx)?;
            let mut timers = timers.borrow_mut();
            timers.clear_timer(id)
        }),
    )?;

    // globals.set(
    //     "setInterval",
    //     Func::from(move |ctx, cb, delay| set_timeout_interval(&ctx, cb, delay, true)),
    // )?;

    // globals.set(
    //     "clearTimeout",
    //     Func::from(move |ctx: Ctx, id: usize| clear_timeout_interval(&ctx, id)),
    // )?;

    // globals.set(
    //     "clearInterval",
    //     Func::from(move |ctx: Ctx, id: usize| clear_timeout_interval(&ctx, id)),
    // )?;

    // globals.set("setImmediate", Func::from(set_immediate))?;

    Ok(())
}
