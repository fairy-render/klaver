mod data;
mod queue;
mod queue_strategy;
pub mod readable;
pub mod writable;

use rquickjs::class::JsClass;

use crate::Registry;

pub use self::{
    queue_strategy::{ByteLengthQueuingStrategy, CountQueuingStrategy},
    readable::{ReadableStream, ReadableStreamDefaultController, ReadableStreamDefaultReader},
    writable::{WritableStream, WritableStreamDefaultController, WritableStreamDefaultWriter},
};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    writable::declare(decl)?;
    readable::declare(decl)?;

    decl.declare(queue_strategy::ByteLengthQueuingStrategy::NAME)?;
    decl.declare(queue_strategy::CountQueuingStrategy::NAME)?;

    Ok(())
}

pub fn export<'js>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &Registry,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    writable::export(ctx, registry, exports)?;
    readable::export(ctx, registry, exports)?;

    export!(
        ctx,
        registry,
        exports,
        ByteLengthQueuingStrategy,
        CountQueuingStrategy
    );

    Ok(())
}
