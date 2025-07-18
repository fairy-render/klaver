use std::marker::PhantomData;

use indexmap::IndexMap;
use rquickjs::{Ctx, FromJs, IntoJs, JsLifetime, Value};
use rquickjs_util::{Date, RuntimeError, throw};

use crate::{
    get_tag_value,
    tag::Tag,
    traits::{Clonable, StructuredClone},
    value::TransObject,
};

pub struct Registry {
    types: IndexMap<Tag, Box<dyn Cloner + Send + Sync>>,
}

unsafe impl<'js> JsLifetime<'js> for Registry {
    type Changed<'to> = Registry;
}

impl Registry {
    pub fn new() -> Result<Registry, RuntimeError> {
        let mut registry = Registry {
            types: Default::default(),
        };

        registry.register::<i64>()?;
        registry.register::<f64>()?;
        registry.register::<bool>()?;
        registry.register::<Date>()?;
        registry.register::<rquickjs::String>()?;
        registry.register::<rquickjs::Object>()?;
        registry.register::<rquickjs::Value>()?;

        Ok(registry)
    }

    pub fn get_by_tag(&self, ctx: &Ctx<'_>, tag: &Tag) -> rquickjs::Result<&dyn Cloner> {
        let Some(cloner) = self.types.get(tag) else {
            throw!(@type ctx, "No cloner for tag")
        };

        Ok(&**cloner)
    }

    pub fn register<T>(&mut self) -> Result<(), RuntimeError>
    where
        T: Clonable,
        T::Cloner: Send + Sync,
    {
        if self.types.contains_key(T::Cloner::tag()) {
            return Err(RuntimeError::Custom(Box::from(format!(
                "Tag '{}' already defined in registry",
                T::Cloner::tag()
            ))));
        }

        self.types.insert(
            T::Cloner::tag().clone(),
            Box::new(ClonerImpl::<T::Cloner>(PhantomData)),
        );

        Ok(())
    }

    pub fn structured_clone<'js, T: Clonable>(
        &self,
        ctx: &Ctx<'js>,
        value: &<T::Cloner as StructuredClone>::Item<'js>,
    ) -> rquickjs::Result<<T::Cloner as StructuredClone>::Item<'js>> {
        let data = T::Cloner::to_transfer_object(ctx, self, value)?;
        let clone = T::Cloner::from_transfer_object(ctx, self, data)?;
        Ok(clone)
    }

    pub fn structured_clone_value<'js>(
        &self,
        ctx: &Ctx<'js>,
        value: &Value<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let data = self.to_transfer_object_value(ctx, value)?;
        let value = self.from_transfer_object_value(ctx, data)?;
        Ok(value)
    }

    pub fn to_transfer_object<'js, T: Clonable>(
        &self,
        ctx: &Ctx<'js>,
        value: <T::Cloner as StructuredClone>::Item<'js>,
    ) -> rquickjs::Result<TransObject> {
        let value = T::Cloner::to_transfer_object(ctx, self, &value)?;

        Ok(TransObject {
            tag: T::Cloner::tag().clone(),
            data: value,
        })
    }

    pub fn to_transfer_object_value<'js>(
        &self,
        ctx: &Ctx<'js>,
        value: &Value<'js>,
    ) -> rquickjs::Result<TransObject> {
        let tag = get_tag_value(ctx, value)?;
        self.get_by_tag(ctx, &tag)?
            .to_transfer_object(ctx, self, value)
    }

    pub fn from_transfer_object_value<'js>(
        &self,
        ctx: &Ctx<'js>,
        object: TransObject,
    ) -> rquickjs::Result<Value<'js>> {
        let cloner = self.get_by_tag(ctx, &object.tag)?;
        cloner.from_transfer_object(ctx, self, object)
    }
}

pub trait Cloner {
    fn tag(&self) -> Tag;

    fn from_transfer_object<'js>(
        &self,
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransObject,
    ) -> rquickjs::Result<Value<'js>>;

    fn to_transfer_object<'js>(
        &self,
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Value<'js>,
    ) -> rquickjs::Result<TransObject>;
}

struct ClonerImpl<T>(PhantomData<T>);

impl<T> Cloner for ClonerImpl<T>
where
    T: StructuredClone,
{
    fn tag(&self) -> Tag {
        T::tag().clone()
    }

    fn from_transfer_object<'js>(
        &self,
        ctx: &Ctx<'js>,
        registry: &Registry,
        obj: TransObject,
    ) -> rquickjs::Result<Value<'js>> {
        let value = T::from_transfer_object(ctx, registry, obj.data)?;
        value.into_js(ctx)
    }

    fn to_transfer_object<'js>(
        &self,
        ctx: &Ctx<'js>,
        registry: &Registry,
        value: &Value<'js>,
    ) -> rquickjs::Result<TransObject> {
        let value = T::Item::from_js(ctx, value.clone())?;

        let data = T::to_transfer_object(ctx, registry, &value)?;

        Ok(TransObject {
            tag: T::tag().clone(),
            data,
        })
    }
}
