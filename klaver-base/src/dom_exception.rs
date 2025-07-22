use std::collections::BTreeMap;

use rquickjs::{
    Class, Ctx, FromJs, JsLifetime, Object, Result, String,
    atom::PredefinedAtom,
    class::JsClass,
    function::{Constructor, Opt},
};
use rquickjs_util::{StringRef, throw};

use crate::{
    Clonable, SerializationContext, StringCloner, StructuredClone, Tag, TransferData,
    export::Exportable, register,
};

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
    pub fn init(ctx: &Ctx<'js>) -> Result<()> {
        let dom_ex_proto = Class::<DOMException>::prototype(ctx)?.expect("DomExpection.prototype");
        let error_ctor: Object = ctx.globals().get(PredefinedAtom::Error)?;
        let error_proto = error_ctor.get_prototype();
        dom_ex_proto.set_prototype(error_proto.as_ref())?;

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

    fn tag() -> &'static crate::Tag {
        static TAG: Tag = Tag::new();
        &TAG
    }

    fn from_transfer_object<'js>(
        ctx: &mut SerializationContext<'js, '_>,
        obj: crate::TransferData,
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
    ) -> rquickjs::Result<crate::TransferData> {
        let mut obj = Vec::with_capacity(3);

        let this = value.borrow();

        let name = ctx.to_transfer_object(this.message.as_value())?;
        let message = ctx.to_transfer_object(this.name.as_value())?;
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
