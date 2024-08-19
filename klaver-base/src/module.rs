use klaver::module_info;
use rquickjs::{module::ModuleDef, Class};

use crate::{
    abort_controller::{AbortController, AbortSignal},
    blob::Blob,
    dom_exception::{self, DOMException},
    event_target::{Emitter, Event, EventTarget},
};

pub struct Module;

module_info!("@klaver/base" => Module);

impl ModuleDef for Module {
    fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("EventTarget")?;
        decl.declare("Event")?;
        decl.declare("DOMException")?;
        decl.declare(stringify!(AbortController))?;
        decl.declare(stringify!(AbortSignal))?;
        decl.declare(stringify!(Blob))?;

        Ok(())
    }

    fn evaluate<'js>(
        ctx: &rquickjs::prelude::Ctx<'js>,
        exports: &rquickjs::module::Exports<'js>,
    ) -> rquickjs::Result<()> {
        let dom_exception =
            Class::<DOMException>::create_constructor(ctx)?.expect("DomException costructor");
        exports.export(stringify!(DOMException), dom_exception)?;
        DOMException::init(ctx)?;

        let event_target =
            Class::<EventTarget>::create_constructor(ctx)?.expect("EventTarget constructor");

        EventTarget::add_event_target_prototype(ctx)?;
        exports.export("EventTarget", event_target)?;

        let event = Class::<Event>::create_constructor(ctx)?.expect("Event constructor");

        exports.export("Event", event)?;

        // AbortController
        let abort_controller =
            Class::<AbortController>::create_constructor(ctx)?.expect("AbortController");
        exports.export(stringify!(AbortController), abort_controller)?;

        // AbortSignal
        let signal = Class::<AbortSignal>::create_constructor(ctx)?.expect("AbortSignal");
        exports.export(stringify!(AbortSignal), signal)?;
        AbortSignal::add_event_target_prototype(ctx)?;

        // Blob
        let blob = Class::<Blob>::create_constructor(ctx)?.expect("Blob");
        exports.export(stringify!(Blob), blob)?;

        Ok(())
    }
}
