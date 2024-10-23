use klaver::module_info;
use klaver_shared::iter::AsyncIterable;
use rquickjs::{module::ModuleDef, prelude::Func, Class};

use crate::{
    abort_controller::{AbortController, AbortSignal},
    blob::Blob,
    console::Console,
    dom_exception::DOMException,
    event_target::{Emitter, Event, EventTarget},
    streams::{
        ByteLengthQueuingStrategy, CountQueuingStrategy, ReadableStream,
        ReadableStreamDefaultReader,
    },
};

#[cfg(feature = "encoding")]
use crate::encoding::{TextDecoder, TextEncoder};

pub struct Module;

module_info!("@klaver/wintercg" => Module);

macro_rules! export {
    ($export: expr, $ctx: expr, $($instance: ty),*) => {
        $(
            let i = Class::<$instance>::create_constructor($ctx)?.expect(stringify!($instance));
            $export.export(stringify!($instance), i)?;
        )*
    };
}

impl ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        // decl.declare("EventTarget")?;
        // decl.declare("Event")?;
        // decl.declare("DOMException")?;
        // decl.declare(stringify!(AbortController))?;
        // decl.declare(stringify!(AbortSignal))?;
        // decl.declare(stringify!(Blob))?;

        decl.declare(stringify!(Console))?;
        // // Stream api
        decl.declare(stringify!(ReadableStream))?;
        decl.declare(stringify!(ReadableStreamDefaultReader))?;
        // decl.declare(stringify!(CountQueuingStrategy))?;
        // decl.declare(stringify!(ByteLengthQueuingStrategy))?;

        #[cfg(feature = "encoding")]
        {
            decl.declare("TextDecoder")?;
            decl.declare("TextEncoder")?;
            decl.declare("atob")?;
            decl.declare("btoa")?;
        }

        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::prelude::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        // export!(exports, ctx, DOMException);
        // DOMException::init(ctx)?;

        // let event_target =
        //     Class::<EventTarget>::create_constructor(ctx)?.expect("EventTarget constructor");

        // EventTarget::add_event_target_prototype(ctx)?;
        // exports.export("EventTarget", event_target)?;

        // let event = Class::<Event>::create_constructor(ctx)?.expect("Event constructor");

        // exports.export("Event", event)?;

        // // AbortController
        // let abort_controller =
        //     Class::<AbortController>::create_constructor(ctx)?.expect("AbortController");
        // exports.export(stringify!(AbortController), abort_controller)?;

        // // AbortSignal
        // let signal = Class::<AbortSignal>::create_constructor(ctx)?.expect("AbortSignal");
        // exports.export(stringify!(AbortSignal), signal)?;
        // AbortSignal::add_event_target_prototype(ctx)?;

        // // Blob
        // let blob = Class::<Blob>::create_constructor(ctx)?.expect("Blob");
        // exports.export(stringify!(Blob), blob)?;

        // // Streams
        export!(
            exports,
            ctx,
            ReadableStream,
            ReadableStreamDefaultReader // CountQueuingStrategy,
                                        // ByteLengthQueuingStrategy
        );

        ReadableStream::add_async_iterable_prototype(ctx)?;

        export!(exports, ctx, Console);

        #[cfg(feature = "encoding")]
        {
            export!(exports, ctx, TextEncoder, TextDecoder);
            exports.export("atob", Func::new(crate::encoding::atob))?;
            exports.export("btoa", Func::new(crate::encoding::btoa))?;
        }

        Ok(())
    }
}
