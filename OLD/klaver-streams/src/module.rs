use klaver::shared::iter::AsyncIterable;
use rquickjs::Class;

use crate::readable_stream::{ReadableStream, ReadableStreamDefaultReader};

klaver::module_info!("@klaver/streams" => Module);

macro_rules! export {
    ($export: expr, $ctx: expr, $($instance: ty),*) => {
        $(
            let i = Class::<$instance>::create_constructor($ctx)?.expect(stringify!($instance));
            $export.export(stringify!($instance), i)?;
        )*
    };
  }

pub struct Module;

impl rquickjs::module::ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(stringify!(ReadableStream))?;
        decl.declare(stringify!(ReadableStreamDefaultReader))?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::prelude::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        export!(
            exports,
            ctx,
            ReadableStream,
            ReadableStreamDefaultReader // CountQueuingStrategy,
                                        // ByteLengthQueuingStrategy
        );

        ReadableStream::add_async_iterable_prototype(ctx)?;

        Ok(())
    }
}
