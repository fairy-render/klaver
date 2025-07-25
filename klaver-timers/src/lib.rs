mod backend;
mod id;
mod module;
mod timers;

use klaver_util::throw_if;
use rquickjs::Ctx;

pub use self::{
    backend::{Backend, TimingBackend},
    id::TimeId,
    module::TimeModule,
    timers::Timers,
};

pub fn set_backend<T: Backend + 'static>(ctx: &Ctx<'_>, backend: T) -> rquickjs::Result<()> {
    throw_if!(ctx, ctx.store_userdata(TimingBackend::new(backend)));
    Ok(())
}
