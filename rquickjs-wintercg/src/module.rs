use rquickjs::{
    module::ModuleDef,
    prelude::{Async, Func},
    Class, IntoJs,
};
use rquickjs_modules::module_info;
use rquickjs_util::{async_iterator::AsyncIterable, util::FunctionExt};

use crate::{
    abort_controller::{AbortController, AbortSignal},
    blob::Blob,
    config::WinterCG,
    console::Console,
    dom_exception::DOMException,
    event_target::{Emitter, Event, EventTarget},
    performance::Performance,
    streams::{
        ByteLengthQueuingStrategy, CountQueuingStrategy, ReadableStream,
        ReadableStreamDefaultReader,
    },
    timers,
};

#[cfg(feature = "encoding")]
use crate::encoding::{TextDecoder, TextEncoder};

pub struct Module;

module_info!("@klaver/wintercg" => Module);

impl ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare(stringify!(config))?;
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

        timers::declare(decl)?;

        #[cfg(feature = "encoding")]
        {
            decl.declare("TextDecoder")?;
            decl.declare("TextEncoder")?;
            decl.declare("atob")?;
            decl.declare("btoa")?;
        }

        #[cfg(feature = "http")]
        crate::http::declare(decl)?;

        #[cfg(feature = "crypto")]
        crate::crypto::declare(decl)?;

        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::prelude::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        let config = Class::instance(ctx.clone(), WinterCG::new(ctx.clone())?)?;
        exports.export(stringify!(config), config.clone())?;

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

        timers::evaluate(ctx, exports, &config)?;

        #[cfg(feature = "encoding")]
        {
            export!(exports, ctx, TextEncoder, TextDecoder);
            exports.export("atob", Func::new(crate::encoding::atob))?;
            exports.export("btoa", Func::new(crate::encoding::btoa))?;
        }

        #[cfg(feature = "http")]
        crate::http::evaluate(ctx, exports, &config)?;

        #[cfg(feature = "crypto")]
        crate::crypto::evaluate(ctx, exports)?;

        Ok(())
    }
}
