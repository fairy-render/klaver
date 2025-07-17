use rquickjs::{class::JsClass, Class, Ctx, Object, Value};

use crate::throw;

pub trait Inheritable<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    #[allow(unused)]
    fn additional_override<T>(ctx: &Ctx<'js>, prototype: &Object<'js>) -> rquickjs::Result<()>
    where
        T: JsClass<'js>,
    {
        Ok(())
    }

    fn inherit<T>(ctx: &Ctx<'js>) -> rquickjs::Result<()>
    where
        T: JsClass<'js>,
    {
        let this_proto = Class::<Self>::prototype(ctx)?;
        let proto = Class::<T>::prototype(ctx)?;

        let Some(proto) = proto else {
            throw!(@type ctx, "Could not get prototype")
        };

        proto.set_prototype(this_proto.as_ref())?;

        Self::additional_override::<T>(ctx, &proto)?;

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

pub trait Inheritable2<'js, T>
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
