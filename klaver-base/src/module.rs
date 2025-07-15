use rquickjs::{Class, class::JsClass, module::ModuleDef};

pub struct BaseModule;

use crate::{
    Emitter,
    abort_controller::AbortController,
    abort_signal::AbortSignal,
    dom_exception::DOMException,
    event_target::{Event, EventTarget},
};

impl ModuleDef for BaseModule {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        declare!(
            decl,
            AbortController,
            AbortSignal,
            EventTarget,
            Event,
            DOMException
        );

        crate::streams::declare(decl)?;

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
            DOMException
        );

        AbortSignal::add_event_target_prototype(ctx)?;
        EventTarget::add_event_target_prototype(ctx)?;
        DOMException::init(ctx)?;

        crate::streams::evaluate(ctx, exports)?;

        Ok(())
    }
}
