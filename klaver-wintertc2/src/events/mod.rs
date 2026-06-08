mod dyn_event;
mod emitter;
mod event;
mod event_target;
mod listener;

use klaver_core::{ExportTarget, Exportable};

pub use self::{dyn_event::*, emitter::*, event::*, event_target::*, listener::*};

pub struct EventsModule;

impl<'js> Exportable<'js> for EventsModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: ExportTarget<'js>,
    {
        EventTarget::export(ctx, registry, target)?;
        Event::export(ctx, registry, target)?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for EventsModule {
    async fn define<'a, 'js: 'a>(&'a self, ctx: rquickjs::Ctx<'js>) -> rquickjs::Result<()> {
        Self::export(
            &ctx,
            &klaver_core::Registry::instance(&ctx)?,
            &ctx.globals(),
        )?;

        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::GlobalInfo for EventsModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }
}
