mod queue;
mod queue_strategy;
mod writable;

pub use self::writable::{
    WritableStream, WritableStreamDefaultController, WritableStreamDefaultWriter,
};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    writable::declare(decl)?;
    Ok(())
}

pub fn evaluate<'js>(
    ctx: &rquickjs::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    writable::evaluate(ctx, exports)?;
    Ok(())
}
