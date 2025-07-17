use rquickjs::{class::JsClass, module::ModuleDef};
use rquickjs_util::Inheritable2;

pub struct BaseModule;

use crate::{
    Console, Emitter, EventTarget, abort_controller::AbortController, abort_signal::AbortSignal,
    blob::Blob, dom_exception::DOMException, events::Event, file::File,
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

        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        define!(
            exports,
            ctx,
            AbortController,
            AbortSignal,
            EventTarget,
            DOMException,
            Blob,
            File,
            Console
        );

        EventTarget::add_event_target_prototype(ctx)?;

        AbortSignal::add_event_target_prototype(ctx)?;
        <EventTarget as Inheritable2<'js, AbortSignal>>::inherit(ctx)?;

        DOMException::init(ctx)?;

        crate::streams::evaluate(ctx, exports)?;
        crate::encoding::evaluate(ctx, exports)?;

        Ok(())
    }
}
