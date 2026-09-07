use klaver_core::{Subclass, value::StringRef};
use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, Object, String, Value,
    class::{JsClass, Trace},
    prelude::{Opt, This},
};

use crate::events::{Event, EventInit, NativeEvent};

/// `ErrorEvent`, per <https://html.spec.whatwg.org/multipage/webappapis.html#errorevent>.
#[derive(Debug, Trace, JsLifetime)]
#[rquickjs::class]
pub struct ErrorEvent<'js> {
    pub base: Event<'js>,
    #[qjs(get)]
    pub message: String<'js>,
    #[qjs(get)]
    pub filename: String<'js>,
    #[qjs(get)]
    pub lineno: u32,
    #[qjs(get)]
    pub colno: u32,
    #[qjs(get)]
    pub error: Value<'js>,
}

/// `ErrorEventInit`, per <https://html.spec.whatwg.org/multipage/webappapis.html#errorevent>.
pub struct ErrorEventInit<'js> {
    pub base: EventInit,
    pub message: Option<String<'js>>,
    pub filename: Option<String<'js>>,
    pub lineno: Option<u32>,
    pub colno: Option<u32>,
    pub error: Option<Value<'js>>,
}

impl<'js> FromJs<'js> for ErrorEventInit<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value.clone())?;

        Ok(ErrorEventInit {
            base: EventInit::from_js(ctx, value)?,
            message: obj.get("message")?,
            filename: obj.get("filename")?,
            lineno: obj.get("lineno")?,
            colno: obj.get("colno")?,
            error: obj.get("error")?,
        })
    }
}

#[rquickjs::methods]
impl<'js> ErrorEvent<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        ty: StringRef<'js>,
        init: Opt<ErrorEventInit<'js>>,
    ) -> rquickjs::Result<ErrorEvent<'js>> {
        let (event_init, message, filename, lineno, colno, error) = match init.0 {
            Some(init) => (
                init.base,
                init.message,
                init.filename,
                init.lineno,
                init.colno,
                init.error,
            ),
            None => (EventInit::default(), None, None, None, None, None),
        };

        let message = match message {
            Some(message) => message,
            None => String::from_str(ctx.clone(), "")?,
        };
        let filename = match filename {
            Some(filename) => filename,
            None => String::from_str(ctx.clone(), "")?,
        };

        Ok(ErrorEvent {
            base: Event::new(ty, Opt(Some(event_init)))?,
            message,
            filename,
            lineno: lineno.unwrap_or(0),
            colno: colno.unwrap_or(0),
            error: error.unwrap_or_else(|| Value::new_null(ctx.clone())),
        })
    }
}

impl<'js> NativeEvent<'js> for ErrorEvent<'js> {
    fn ty(this: This<Class<'js, Self>>, _ctx: Ctx<'js>) -> rquickjs::Result<String<'js>> {
        Ok(this.borrow().base.ty.to_js_string())
    }

    fn event(&self) -> &Event<'js> {
        &self.base
    }
}

impl<'js> Subclass<'js, Event<'js>> for ErrorEvent<'js> {}

impl<'js> klaver_core::Exportable<'js> for ErrorEvent<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            ErrorEvent::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        Self::inherit(ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Emitter, EventTarget};
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    /// Runs `body` as the contents of a plain function, with `EventTarget`, `Event` and
    /// `ErrorEvent` available as globals. `body` is expected to throw on failure (e.g. via a
    /// plain `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                ctx.globals().set(
                    "EventTarget",
                    Class::<EventTarget>::create_constructor(&ctx)?,
                )?;
                EventTarget::add_event_target_prototype(&ctx)?;

                ctx.globals()
                    .set("Event", Class::<Event>::create_constructor(&ctx)?)?;
                Event::add_event_prototype(&ctx)?;

                ctx.globals().set(
                    "ErrorEvent",
                    Class::<ErrorEvent>::create_constructor(&ctx)?,
                )?;
                ErrorEvent::inherit(&ctx)?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn fields_default_per_spec() {
        run(r#"
            const event = new ErrorEvent("error");
            if (event.type !== "error") throw new Error(`type was ${event.type}`);
            if (event.message !== "") throw new Error(`message was ${JSON.stringify(event.message)}`);
            if (event.filename !== "") throw new Error(`filename was ${JSON.stringify(event.filename)}`);
            if (event.lineno !== 0) throw new Error(`lineno was ${event.lineno}`);
            if (event.colno !== 0) throw new Error(`colno was ${event.colno}`);
            if (event.error !== null) throw new Error(`error was ${event.error}`);
        "#);
    }

    #[test]
    fn fields_are_set_from_init() {
        run(r#"
            const cause = new Error("boom");
            const event = new ErrorEvent("error", {
                message: "bad things happened",
                filename: "app.js",
                lineno: 42,
                colno: 7,
                error: cause,
                bubbles: true,
            });
            if (event.message !== "bad things happened") {
                throw new Error(`message was ${event.message}`);
            }
            if (event.filename !== "app.js") throw new Error(`filename was ${event.filename}`);
            if (event.lineno !== 42) throw new Error(`lineno was ${event.lineno}`);
            if (event.colno !== 7) throw new Error(`colno was ${event.colno}`);
            if (event.error !== cause) throw new Error(`error was ${event.error}`);
            if (event.bubbles !== true) throw new Error(`bubbles was ${event.bubbles}`);
        "#);
    }

    #[test]
    fn is_instance_of_event_and_error_event() {
        run(r#"
            const event = new ErrorEvent("error");
            if (!(event instanceof Event)) throw new Error("expected instanceof Event");
            if (!(event instanceof ErrorEvent)) throw new Error("expected instanceof ErrorEvent");
        "#);
    }

    #[test]
    fn dispatches_through_event_target() {
        run(r#"
            const target = new EventTarget();
            let seenMessage;
            target.addEventListener("error", (e) => { seenMessage = e.message; });
            target.dispatchEvent(new ErrorEvent("error", { message: "oops" }));
            if (seenMessage !== "oops") throw new Error(`seenMessage was ${seenMessage}`);
        "#);
    }
}
