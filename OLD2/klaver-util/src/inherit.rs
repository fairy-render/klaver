use rquickjs::{Class, Ctx, Object, Value, class::JsClass};

use crate::throw;

/// Inheritable is a trait that defines the interface for inheriting from another class.
/// It is used to define classes that can be inherited from another class.
pub trait Inheritable<'js, T>
where
    Self: JsClass<'js> + Sized + 'js,
    T: JsClass<'js>,
{
    #[allow(unused)]
    fn additional_override(ctx: &Ctx<'js>, prototype: &Object<'js>) -> rquickjs::Result<()> {
        Ok(())
    }

    ///Inherits from another class.
    /// This will set the prototype of the class to the prototype of the other class,
    /// and it will also set the constructor of the class to the constructor of the other class.
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
    /// Checks if the value is an instance of the class or any of its subclasses.
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
