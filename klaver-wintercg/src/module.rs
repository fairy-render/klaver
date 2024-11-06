use rquickjs::{module::ModuleDef, Class};
use rquickjs_modules::module_info;

use crate::{
    abort_controller::{AbortController, AbortSignal},
    blob::Blob,
    config::WinterCG,
    console::Console,
    dom_exception::DOMException,
    event_target::{Emitter, Event, EventTarget},
    performance::Performance,
    timers,
};

const TYPES: &'static str = include_str!(concat!(env!("OUT_DIR"), "/module.d.ts"));

pub struct Module;

module_info!("@klaver/wintercg" @types: TYPES => Module);

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
        decl.declare(stringify!(process))?;

        // // Stream api
        crate::streams::declare(decl)?;

        timers::declare(decl)?;

        #[cfg(feature = "encoding")]
        crate::encoding::declare(decl)?;

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
        // export!(
        //     exports,
        //     ctx,
        //     ReadableStream,
        //     ReadableStreamDefaultReader,
        //     CountQueuingStrategy,
        //     ByteLengthQueuingStrategy
        // );
        // ReadableStream::add_async_iterable_prototype(ctx)?;
        crate::streams::evaluate(ctx, exports)?;

        timers::evaluate(ctx, exports, &config)?;

        #[cfg(feature = "encoding")]
        crate::encoding::evaluate(ctx, exports)?;

        #[cfg(feature = "http")]
        crate::http::evaluate(ctx, exports, &config)?;

        #[cfg(feature = "crypto")]
        crate::crypto::evaluate(ctx, exports)?;

        let process = crate::process::process(ctx.clone(), config.clone())?;
        exports.export("process", process)?;

        Ok(())
    }
}
