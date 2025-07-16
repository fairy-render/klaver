mod data;
mod queue;
mod queue_strategy;
pub mod readable;
pub mod writable;

use rquickjs::class::JsClass;

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

pub fn evaluate<'js>(
    ctx: &rquickjs::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    writable::evaluate(ctx, exports)?;
    readable::evaluate(ctx, exports)?;

    define!(
        exports,
        ctx,
        ByteLengthQueuingStrategy,
        CountQueuingStrategy
    );

    Ok(())
}
