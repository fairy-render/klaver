use rquickjs::{Class, Ctx, Object, Value, class::JsClass};

use crate::throw;

pub trait Inheritable<'js, T>
where
    Self: JsClass<'js> + Sized + 'js,
    T: JsClass<'js>,
{
    #[allow(unused)]
    fn additional_override(ctx: &Ctx<'js>, prototype: &Object<'js>) -> rquickjs::Result<()> {
        Ok(())
    }

    fn inherit(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let this_proto = Class::<Self>::prototype(ctx)?;
        let proto = Class::<T>::prototype(ctx)?;

        let Some(proto) = proto else {
            throw!(@type ctx, "Could not get prototype")
        };

        proto.set_prototype(this_proto.as_ref())?;

        Self::additional_override(ctx, &proto)?;

        Ok(())
    }

    fn instance_of(ctx: &Ctx<'js>, value: impl AsRef<Value<'js>>) -> rquickjs::Result<bool> {
        let Some(obj) = value.as_ref().as_object() else {
            throw!(@type ctx, "Expected object")
        };

        let ctor = Class::<Self>::create_constructor(ctx)?;

        let Some(ctor) = ctor else {
            throw!(@type ctx, "Could not get constructor")
        };

        Ok(obj.is_instance_of(&ctor))
    }
}

pub trait Subclass<'js, T>
where
    Self: JsClass<'js>,
    T: Inheritable<'js, Self>,
{
    fn inherit(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        <T as Inheritable<'js, Self>>::inherit(ctx)
    }
}

pub trait SuperClass<'js>
where
    Self: JsClass<'js>,
{
    fn is_subclass(ctx: &Ctx<'js>, value: impl AsRef<Value<'js>>) -> rquickjs::Result<bool> {
        let Some(obj) = value.as_ref().as_object() else {
            throw!(@type ctx, "Expected object")
        };

        let ctor = Class::<Self>::create_constructor(ctx)?;

        let Some(ctor) = ctor else {
            throw!(@type ctx, "Could not get constructor")
        };

        Ok(obj.is_instance_of(&ctor))
    }
}
