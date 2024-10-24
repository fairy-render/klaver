use klaver::module_info;
use klaver_shared::{iter::AsyncIterable, util::FunctionExt};
use rquickjs::{
    module::ModuleDef,
    prelude::{Async, Func},
    Class, IntoJs,
};

use crate::{
    abort_controller::{AbortController, AbortSignal},
    blob::Blob,
    console::Console,
    crypto,
    dom_exception::DOMException,
    event_target::{Emitter, Event, EventTarget},
    http,
    performance::Performance,
    streams::{
        ByteLengthQueuingStrategy, CountQueuingStrategy, ReadableStream,
        ReadableStreamDefaultReader,
    },
};

#[cfg(feature = "encoding")]
use crate::encoding::{TextDecoder, TextEncoder};
#[cfg(feature = "http")]
use crate::http::{fetch, Client, Headers, Request, Response, Url};

pub struct Module;

module_info!("@klaver/wintercg" => Module);

impl ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(stringify!(EventTarget))?;
        decl.declare(stringify!(Event))?;
        decl.declare(stringify!(DOMException))?;
        decl.declare(stringify!(AbortController))?;
        decl.declare(stringify!(AbortSignal))?;
        decl.declare(stringify!(Blob))?;
        decl.declare(stringify!(Console))?;
        decl.declare(stringify!(Performance))?;

        // // Stream api
        decl.declare(stringify!(ReadableStream))?;
        decl.declare(stringify!(ReadableStreamDefaultReader))?;
        decl.declare(stringify!(CountQueuingStrategy))?;
        decl.declare(stringify!(ByteLengthQueuingStrategy))?;

        #[cfg(feature = "encoding")]
        {
            decl.declare("TextDecoder")?;
            decl.declare("TextEncoder")?;
            decl.declare("atob")?;
            decl.declare("btoa")?;
        }

        #[cfg(feature = "http")]
        http::declare(decl)?;

        #[cfg(feature = "crypto")]
        crypto::declare(decl)?;

        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::prelude::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        export!(exports, ctx, DOMException);
        DOMException::init(ctx)?;

        export!(exports, ctx, EventTarget, Event);
        EventTarget::add_event_target_prototype(ctx)?;

        export!(exports, ctx, AbortController, AbortSignal);
        AbortSignal::add_event_target_prototype(ctx)?;

        export!(exports, ctx, Blob, Console, Performance);

        // // Streams
        export!(
            exports,
            ctx,
            ReadableStream,
            ReadableStreamDefaultReader,
            CountQueuingStrategy,
            ByteLengthQueuingStrategy
        );
        ReadableStream::add_async_iterable_prototype(ctx)?;

        #[cfg(feature = "encoding")]
        {
            export!(exports, ctx, TextEncoder, TextDecoder);
            exports.export("atob", Func::new(crate::encoding::atob))?;
            exports.export("btoa", Func::new(crate::encoding::btoa))?;
        }

        #[cfg(feature = "http")]
        http::evaluate(ctx, exports)?;

        #[cfg(feature = "crypto")]
        crypto::evaluate(ctx, exports)?;

        Ok(())
    }
}
