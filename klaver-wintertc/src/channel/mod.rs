mod channel;
mod event;
mod port;

use klaver_core::{ExportTarget, Exportable};

pub use self::{channel::MessageChannel, event::MessageEvent, port::Channel, port::MessagePort};

pub struct ChannelModule;

impl<'js> Exportable<'js> for ChannelModule {
    fn export<T>(
        ctx: &rquickjs::Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: ExportTarget<'js>,
    {
        MessageChannel::export(ctx, registry, target)?;
        MessagePort::export(ctx, registry, target)?;
        MessageEvent::export(ctx, registry, target)?;
        Ok(())
    }
}

#[cfg(feature = "module")]
impl klaver_modules::Global for ChannelModule {
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
impl klaver_modules::GlobalInfo for ChannelModule {
    fn register(builder: &mut klaver_modules::GlobalBuilder<'_, Self>) {
        builder.register(Self);
    }
}
