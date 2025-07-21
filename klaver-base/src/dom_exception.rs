use rquickjs::{
    Class, Ctx, Object, Result,
    atom::PredefinedAtom,
    class::JsClass,
    function::{Constructor, Opt},
};

use crate::{Clonable, StructuredClone, Tag, export::Exportable, register};

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

pub struct DomExceptionCloner;

impl StructuredClone for DomExceptionCloner {
    type Item<'js> = Class<'js, DOMException>;

    fn tag() -> &'static crate::Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &crate::Registry,
        obj: crate::TransferData,
    ) -> rquickjs::Result<Self::Item<'js>> {
        todo!()
    }

    fn to_transfer_object<'js>(
        ctx: &Ctx<'js>,
        registry: &crate::Registry,
        value: &Self::Item<'js>,
    ) -> rquickjs::Result<crate::TransferData> {
        todo!()
    }
}

impl Clonable for DOMException {
    type Cloner = DomExceptionCloner;
}

impl<'js> Exportable<'js> for DOMException {
    fn export<T>(ctx: &Ctx<'js>, registry: &crate::Registry, target: &T) -> rquickjs::Result<()>
    where
        T: crate::export::ExportTarget<'js>,
    {
        register::<DOMException>(ctx, registry)?;
        target.set(
            ctx,
            DOMException::NAME,
            Class::<DOMException>::create_constructor(ctx)?,
        )?;

        Ok(())
    }
}
