mod channel;
mod port;

use crate::{ExportTarget, Exportable};

pub use self::{channel::MessageChannel, port::MessagePort};

use rquickjs::{Ctx, class::JsClass};

pub fn declare<'js>(module: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    declare!(module, MessageChannel, MessagePort);
    Ok(())
}

pub fn export<'js, T>(
    ctx: &Ctx<'js>,
    registry: &crate::Registry,
    target: &T,
) -> rquickjs::Result<()>
where
    T: ExportTarget<'js>,
{
    MessageChannel::export(ctx, registry, target)?;
    MessagePort::export(ctx, registry, target)?;

    Ok(())
}
