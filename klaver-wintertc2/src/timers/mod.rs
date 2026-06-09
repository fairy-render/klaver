mod backend;
mod id;
mod module;
mod timers;

use klaver_core::throw_if;
use rquickjs::Ctx;

pub use self::{
    backend::{TimerBackend, TimingBackend},
    id::TimeId,
    module::TimeModule,
    timers::Timers,
};

pub fn set_backend<T: TimerBackend + 'static>(ctx: &Ctx<'_>, backend: T) -> rquickjs::Result<()> {
    throw_if!(ctx, ctx.store_userdata(TimingBackend::new(backend)));
    Ok(())
}
