use klaver::shared::iter::AsyncIterable;
use rquickjs::Class;

use crate::stream::ReadableStream;

klaver::module_info!("@klaver/streams" => Module);

pub struct Module;

impl rquickjs::module::ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("ReadableStream")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::prelude::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        let ctor = Class::<ReadableStream>::create_constructor(ctx)?;
        exports.export("ReadableStream", ctor)?;
        ReadableStream::add_async_iterable_prototype(ctx)?;

        Ok(())
    }
}
