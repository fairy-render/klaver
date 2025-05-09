use rquickjs::{
    atom::PredefinedAtom,
    function::{Constructor, Opt},
    Class, Ctx, Object, Result,
};

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
#[derive(rquickjs::class::Trace)]
pub struct DOMException {
    message: String,
    name: String,
    stack: String,
}

impl DOMException {
    pub fn init(ctx: &Ctx<'_>) -> Result<()> {
        let dom_ex_proto = Class::<DOMException>::prototype(ctx)?.expect("DomExpection.prototype");
        let error_ctor: Object = ctx.globals().get(PredefinedAtom::Error)?;
        let error_proto = error_ctor.get_prototype();
        dom_ex_proto.set_prototype(error_proto.as_ref())?;

        Ok(())
    }
}

#[rquickjs::methods]
impl DOMException {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>, message: Opt<String>, name: Opt<String>) -> Result<Self> {
        let error_ctor: Constructor = ctx.globals().get(PredefinedAtom::Error)?;
        let new: Object = error_ctor.construct((message.clone(),))?;

        let message = message.0.unwrap_or(String::from(""));
        let name = name.0.unwrap_or(String::from("Error"));

        Ok(Self {
            message,
            name,
            stack: new.get::<_, String>(PredefinedAtom::Stack)?,
        })
    }

    #[qjs(get)]
    fn message(&self) -> String {
        self.message.clone()
    }

    #[qjs(get)]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[qjs(get)]
    fn stack(&self) -> String {
        self.stack.clone()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string(&self) -> String {
        if self.message.is_empty() {
            return self.name.clone();
        }

        [self.name.as_str(), self.message.as_str()].join(": ")
    }
}
