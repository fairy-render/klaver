mod readable_stream;

pub use self::readable_stream::*;

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    Ok(())
}

pub fn evaluate<'js>(
    ctx: &rquickjs::prelude::Ctx<'js>,
    exports: &rquickjs::module::Exports<'js>,
) -> rquickjs::Result<()> {
    Ok(())
}
