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
#[cfg(feature = "crypto")]

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
        {
            decl.declare(stringify!(Response))?;
            decl.declare(stringify!(Request))?;
            decl.declare(stringify!(Headers))?;
            decl.declare(stringify!(URL))?;
            decl.declare(stringify!(Client))?;
            decl.declare(stringify!(fetch))?;
        }

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
        {
            export!(exports, ctx, Response, Request, Headers, Client);
            exports.export("URL", Class::<Url>::create_constructor(&ctx)?)?;

            let fetch = Func::new(Async(fetch))
                .into_js(&ctx)?
                .into_function()
                .unwrap();

            let client = Class::instance(ctx.clone(), Client::new(ctx.clone())?)?;
            let fetch = fetch.bind(ctx.clone(), (ctx.globals(), client))?;

            exports.export("fetch", fetch)?;
        }

        #[cfg(feature = "crypto")]
        crypto::evaluate(ctx, exports)?;

        Ok(())
    }
}
