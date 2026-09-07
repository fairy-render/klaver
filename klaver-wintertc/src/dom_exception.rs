use klaver_core::{throw, value::StringRef};
use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, Object, Result, String,
    atom::PredefinedAtom,
    class::JsClass,
    function::{Constructor, Opt},
    object::Property,
};

use klaver_core::value::structured_clone::{
    Clonable, SerializationContext, StructuredClone, Tag, TransferData, register,
};

use klaver_core::Exportable;

/// The legacy numeric error codes from the DOM spec's "error names table" (§2.8.1). Per WebIDL's
/// `const`-in-interface rules these are exposed both as static properties on the `DOMException`
/// constructor and as (non-writable, non-configurable, enumerable) properties on
/// `DOMException.prototype`, regardless of whether a given code still has a name mapping below.
const LEGACY_CODES: [(&str, u16); 25] = [
    ("INDEX_SIZE_ERR", 1),
    ("DOMSTRING_SIZE_ERR", 2),
    ("HIERARCHY_REQUEST_ERR", 3),
    ("WRONG_DOCUMENT_ERR", 4),
    ("INVALID_CHARACTER_ERR", 5),
    ("NO_DATA_ALLOWED_ERR", 6),
    ("NO_MODIFICATION_ALLOWED_ERR", 7),
    ("NOT_FOUND_ERR", 8),
    ("NOT_SUPPORTED_ERR", 9),
    ("INUSE_ATTRIBUTE_ERR", 10),
    ("INVALID_STATE_ERR", 11),
    ("SYNTAX_ERR", 12),
    ("INVALID_MODIFICATION_ERR", 13),
    ("NAMESPACE_ERR", 14),
    ("INVALID_ACCESS_ERR", 15),
    ("VALIDATION_ERR", 16),
    ("TYPE_MISMATCH_ERR", 17),
    ("SECURITY_ERR", 18),
    ("NETWORK_ERR", 19),
    ("ABORT_ERR", 20),
    ("URL_MISMATCH_ERR", 21),
    ("QUOTA_EXCEEDED_ERR", 22),
    ("TIMEOUT_ERR", 23),
    ("INVALID_NODE_TYPE_ERR", 24),
    ("DATA_CLONE_ERR", 25),
];

/// The subset of [`LEGACY_CODES`] that a `name` string maps to, used to compute the `code`
/// getter. Names not in this table (and the historical, name-less codes above) yield `code === 0`.
const NAME_TO_LEGACY_CODE: [(&str, u16); 22] = [
    ("IndexSizeError", 1),
    ("HierarchyRequestError", 3),
    ("WrongDocumentError", 4),
    ("InvalidCharacterError", 5),
    ("NoModificationAllowedError", 7),
    ("NotFoundError", 8),
    ("NotSupportedError", 9),
    ("InUseAttributeError", 10),
    ("InvalidStateError", 11),
    ("SyntaxError", 12),
    ("InvalidModificationError", 13),
    ("NamespaceError", 14),
    ("InvalidAccessError", 15),
    ("TypeMismatchError", 17),
    ("SecurityError", 18),
    ("NetworkError", 19),
    ("AbortError", 20),
    ("URLMismatchError", 21),
    ("QuotaExceededError", 22),
    ("TimeoutError", 23),
    ("InvalidNodeTypeError", 24),
    ("DataCloneError", 25),
];

#[rquickjs::class]
#[derive(rquickjs::class::Trace)]
pub struct DOMException<'js> {
    message: String<'js>,
    name: String<'js>,
    stack: String<'js>,
}

unsafe impl<'js> JsLifetime<'js> for DOMException<'js> {
    type Changed<'to> = DOMException<'to>;
}

impl<'js> DOMException<'js> {
    pub fn init(ctx: &Ctx<'js>, constructor: &Constructor<'js>) -> Result<()> {
        let dom_ex_proto = Class::<DOMException>::prototype(ctx)?.expect("DomExpection.prototype");
        let error_ctor: Object = ctx.globals().get(PredefinedAtom::Error)?;
        // `.get_prototype()` would return the *constructor's own* [[Prototype]] (i.e.
        // `Function.prototype`, since `Error` is itself a function) - what's needed here is the
        // `prototype` property that `Error` instances actually inherit from.
        let error_proto: Object = error_ctor.get(PredefinedAtom::Prototype)?;
        dom_ex_proto.set_prototype(Some(&error_proto))?;

        for (name, code) in LEGACY_CODES {
            constructor.prop(name, Property::from(code).enumerable())?;
            dom_ex_proto.prop(name, Property::from(code).enumerable())?;
        }

        Ok(())
    }
}

#[rquickjs::methods]
impl<'js> DOMException<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, message: Opt<String<'js>>, name: Opt<String<'js>>) -> Result<Self> {
        let error_ctor: Constructor = ctx.globals().get(PredefinedAtom::Error)?;
        let new: Object = error_ctor.construct((message.clone(),))?;

        let message = new.get(PredefinedAtom::Message)?;

        let name = match name.0 {
            Some(name) => name,
            None => String::from_str(ctx.clone(), "Error")?,
        };

        Ok(Self {
            message,
            name,
            stack: new.get::<_, String>(PredefinedAtom::Stack)?,
        })
    }

    #[qjs(get)]
    fn message(&self) -> String<'js> {
        self.message.clone()
    }

    #[qjs(get)]
    fn name(&self) -> String<'js> {
        self.name.clone()
    }

    #[qjs(get)]
    fn stack(&self) -> String<'js> {
        self.stack.clone()
    }

    #[qjs(get)]
    fn code(&self) -> rquickjs::Result<u16> {
        let name = StringRef::from_string(self.name.clone())?;
        Ok(NAME_TO_LEGACY_CODE
            .iter()
            .find(|(n, _)| *n == name.as_str())
            .map(|(_, code)| *code)
            .unwrap_or(0))
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&self) -> rquickjs::Result<std::string::String> {
        let name = StringRef::from_string(self.name.clone())?;
        let message = StringRef::from_string(self.message.clone())?;

        if message.as_str().is_empty() {
            return Ok(name.as_str().to_string());
        }

        Ok([name.as_str(), message.as_str()].join(": "))
    }
}

pub struct DomExceptionCloner;

impl StructuredClone for DomExceptionCloner {
    type Item<'js> = Class<'js, DOMException<'js>>;

    fn tag() -> &'static Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        let TransferData::List(mut list) = obj else {
            throw!(@type ctx, "Expected a list with 3 items")
        };

        if list.len() != 3 {
            throw!(@type ctx, "Expected a list with 3 items")
        }

        let stack = list.pop().unwrap();
        let message = list.pop().unwrap();
        let name = list.pop().unwrap();

        let name = ctx.from_transfer_object(name)?;
        let message = ctx.from_transfer_object(message)?;
        let stack = ctx.from_transfer_object(stack)?;

        let name = String::from_js(ctx.ctx(), name)?;
        let message = String::from_js(ctx.ctx(), message)?;
        let stack = String::from_js(ctx.ctx(), stack)?;

        let this = Class::instance(
            ctx.ctx().clone(),
            DOMException {
                name,
                message,
                stack,
            },
        )?;

        Ok(this)
    }

    fn to_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<TransferData> {
        let mut obj = Vec::with_capacity(3);

        let this = value.borrow();

        let name = ctx.to_transfer_object(this.name.as_value())?;
        let message = ctx.to_transfer_object(this.message.as_value())?;
        let stack = ctx.to_transfer_object(this.stack.as_value())?;

        obj.push(name);
        obj.push(message);
        obj.push(stack);

        Ok(TransferData::List(obj))
    }
}

impl<'js> Clonable for DOMException<'js> {
    type Cloner = DomExceptionCloner;
}

impl<'js> Exportable<'js> for DOMException<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        register::<DOMException>(ctx, registry)?;

        let constructor = Class::<DOMException>::create_constructor(ctx)?
            .expect("DOMException constructor");
        DOMException::init(ctx, &constructor)?;
        target.set(ctx, DOMException::NAME, constructor)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    /// Runs `body` as the contents of a plain function, with a global `DOMException`
    /// constructor available (constants + `Error.prototype` chain installed, matching what
    /// `Exportable::export` does). `body` is expected to throw on failure (e.g. via a plain
    /// `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                let constructor = Class::<DOMException>::create_constructor(&ctx)?
                    .expect("DOMException constructor");
                DOMException::init(&ctx, &constructor)?;
                ctx.globals().set(DOMException::NAME, constructor)?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn name_message_and_to_string() {
        run(r#"
            const err = new DOMException("bad state", "InvalidStateError");
            if (err.name !== "InvalidStateError") throw new Error(`name was ${err.name}`);
            if (err.message !== "bad state") throw new Error(`message was ${err.message}`);
            const str = err.toString();
            if (str !== "InvalidStateError: bad state") throw new Error(`toString was ${str}`);
        "#);
    }

    #[test]
    fn default_name_is_error() {
        run(r#"
            const err = new DOMException("oops");
            if (err.name !== "Error") throw new Error(`name was ${err.name}`);
        "#);
    }

    #[test]
    fn is_instance_of_error_and_dom_exception() {
        run(r#"
            const err = new DOMException("oops", "AbortError");
            if (!(err instanceof Error)) throw new Error("expected instanceof Error");
            if (!(err instanceof DOMException)) throw new Error("expected instanceof DOMException");
        "#);
    }

    #[test]
    fn code_is_derived_from_name() {
        run(r#"
            const notFound = new DOMException("missing", "NotFoundError");
            if (notFound.code !== 8) throw new Error(`code was ${notFound.code}`);

            const unrecognized = new DOMException("mystery", "SomethingElseError");
            if (unrecognized.code !== 0) throw new Error(`code was ${unrecognized.code}`);

            const noName = new DOMException("mystery");
            if (noName.code !== 0) throw new Error(`code was ${noName.code}`);
        "#);
    }

    #[test]
    fn legacy_constants_exist_on_constructor_and_instances() {
        run(r#"
            if (DOMException.NOT_FOUND_ERR !== 8) throw new Error(`static was ${DOMException.NOT_FOUND_ERR}`);
            if (DOMException.INVALID_STATE_ERR !== 11) throw new Error(`static was ${DOMException.INVALID_STATE_ERR}`);
            // Historical codes with no `name` mapping still exist as constants.
            if (DOMException.DOMSTRING_SIZE_ERR !== 2) throw new Error("missing DOMSTRING_SIZE_ERR");
            if (DOMException.NO_DATA_ALLOWED_ERR !== 6) throw new Error("missing NO_DATA_ALLOWED_ERR");
            if (DOMException.VALIDATION_ERR !== 16) throw new Error("missing VALIDATION_ERR");

            const err = new DOMException("oops", "NotFoundError");
            if (err.NOT_FOUND_ERR !== 8) throw new Error(`instance constant was ${err.NOT_FOUND_ERR}`);
            if (err.INVALID_STATE_ERR !== 11) throw new Error(`instance constant was ${err.INVALID_STATE_ERR}`);
        "#);
    }

    #[test]
    fn legacy_constants_are_read_only_and_non_configurable() {
        run(r#"
            "use strict";
            const desc = Object.getOwnPropertyDescriptor(DOMException, "NOT_FOUND_ERR");
            if (desc.writable !== false) throw new Error("expected non-writable");
            if (desc.configurable !== false) throw new Error("expected non-configurable");
            if (desc.enumerable !== true) throw new Error("expected enumerable");

            let threw = false;
            try {
                DOMException.NOT_FOUND_ERR = 99;
            } catch (e) {
                threw = true;
            }
            if (!threw) throw new Error("expected assignment to throw in strict mode");
        "#);
    }
}
