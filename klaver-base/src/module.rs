use rquickjs::{Ctx, Value, class::JsClass, module::ModuleDef, prelude::Func};
use rquickjs_util::Subclass;

pub struct BaseModule;

use crate::{
    Console, Emitter, EventTarget, Registry, abort_controller::AbortController,
    abort_signal::AbortSignal, blob::Blob, dom_exception::DOMException, events::Event, file::File,
    structured_clone,
};

impl ModuleDef for BaseModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        declare!(
            decl,
            AbortController,
            AbortSignal,
            EventTarget,
            Event,
            DOMException,
            Blob,
            File,
            Console
        );

        crate::streams::declare(decl)?;
        crate::encoding::declare(decl)?;
        crate::message::declare(decl)?;

        decl.declare("structuredClone")?;

        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        let registry = &Registry::get(ctx)?;

        export!(
            ctx,
            registry,
            exports,
            AbortController,
            AbortSignal,
            EventTarget,
            DOMException,
            Blob,
            Console
        );

        EventTarget::add_event_target_prototype(ctx)?;
        AbortSignal::inherit(ctx)?;

        DOMException::init(ctx)?;

        crate::encoding::export(ctx, registry, exports)?;
        crate::streams::export(ctx, registry, exports)?;
        crate::message::export(ctx, registry, exports)?;

        exports.export("structuredClone", Func::from(structured_clone))?;

        Ok(())
    }
}
