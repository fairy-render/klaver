use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;
use klaver_util::{Date, RuntimeError, throw, throw_if};
use rquickjs::{Ctx, FromJs, IntoJs, JsLifetime, Value};

use crate::{ObjectId, TransferData, structured_clone::context::SerializationContext};

use super::{
    get_tag_value,
    tag::Tag,
    traits::{Clonable, StructuredClone},
    value::TransObject,
};

#[derive(Clone, PartialEq, Eq, Default)]
pub struct SerializationOptions<'js> {
    pub transfer: Option<Vec<Value<'js>>>,
}

impl<'js> FromJs<'js> for SerializationOptions<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        let Some(opt) = value.as_object() else {
            return Err(rquickjs::Error::new_from_js("value", "object"));
        };

        Ok(SerializationOptions {
            transfer: opt.get("transfer")?,
        })
    }
}

#[derive(Clone)]
pub struct Registry {
    types: Arc<RwLock<IndexMap<Tag, Arc<dyn DynCloner + Send + Sync>>>>,
}

unsafe impl<'js> JsLifetime<'js> for Registry {
    type Changed<'to> = Registry;
}

impl Registry {
    pub fn new() -> Result<Registry, RuntimeError> {
        let registry = Registry {
            types: Default::default(),
        };

        registry.register::<i64>()?;
        registry.register::<f64>()?;
        registry.register::<bool>()?;
        registry.register::<Date>()?;
        registry.register::<rquickjs::String>()?;
        registry.register::<rquickjs::Object>()?;
        // registry.register::<rquickjs::Value>()?;

        Ok(registry)
    }

    pub fn get(ctx: &Ctx<'_>) -> rquickjs::Result<Registry> {
        let registry = match ctx.userdata::<Registry>() {
            Some(registry) => registry,
            None => {
                let reg = throw_if!(ctx, Registry::new());
                throw_if!(ctx, ctx.store_userdata(reg));
                ctx.userdata::<Registry>().expect("Registry")
            }
        };
        Ok((*registry).clone())
    }

    pub fn get_by_tag(&self, ctx: &Ctx<'_>, tag: &Tag) -> rquickjs::Result<Cloner> {
        let lock = self.types.read().expect("Lock");

        let Some(cloner) = lock.get(tag) else {
            throw!(@type ctx, "No cloner for tag")
        };
        Ok(Cloner(cloner.clone()))
    }

    pub fn register<T>(&self) -> Result<(), RuntimeError>
    where
        T: Clonable,
        T::Cloner: Send + Sync,
    {
        let mut lock = self.types.write().expect("Lock");

        if lock.contains_key(T::Cloner::tag()) {
            return Err(RuntimeError::Custom(Box::from(format!(
                "Tag '{}' already defined in registry",
                T::Cloner::tag()
            ))));
        }

        lock.insert(
            T::Cloner::tag().clone(),
            Arc::new(ClonerImpl::<T::Cloner>(PhantomData)),
        );

        Ok(())
    }

    pub fn serialize<'js>(
        &self,
        ctx: &Ctx<'js>,
        value: &Value<'js>,
        options: &SerializationOptions<'js>,
    ) -> rquickjs::Result<TransObject> {
        let tag = get_tag_value(ctx, value)?;
        let cloner = self.get_by_tag(ctx, &tag)?;

        let (id, data) = self.serialize_inner(&cloner, ctx, value, options)?;

        Ok(TransObject::Data { tag, data, id })
    }

    pub fn deserialize<'js>(
        &self,
        ctx: &Ctx<'js>,
        object: TransObject,
    ) -> rquickjs::Result<Value<'js>> {
        let opts = SerializationOptions::default();
        let mut ctx = SerializationContext::new(ctx.clone(), self, &opts);
        ctx.from_transfer_object(object)
    }

    fn serialize_inner<'js>(
        &self,
        cloner: &Cloner,
        ctx: &Ctx<'js>,
        value: &Value<'js>,
        options: &SerializationOptions<'js>,
    ) -> rquickjs::Result<(ObjectId, TransferData)> {
        let mut ctx = SerializationContext::new(ctx.clone(), self, options);
        let data = cloner.to_transfer_object(&mut ctx, value)?;
        Ok((ctx.id(), data))
    }

    pub fn structured_clone_value<'js>(
        &self,
        ctx: &Ctx<'js>,
        value: &Value<'js>,
        options: &SerializationOptions<'js>,
    ) -> rquickjs::Result<Value<'js>> {
        let tag = get_tag_value(ctx, value)?;
        let cloner = self.get_by_tag(ctx, &tag)?;

        let (_, data) = self.serialize_inner(&cloner, ctx, value, options)?;

        let mut ctx = SerializationContext::new(ctx.clone(), self, options);
        let value = cloner.from_transfer_object(&mut ctx, data)?;

        Ok(value)
    }
}

#[derive(Clone)]
pub struct Cloner(Arc<dyn DynCloner + Send + Sync>);

impl Cloner {
    fn tag(&self) -> &Tag {
        self.0.tag()
    }

    pub fn from_transfer_object<'js>(
        &self,
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Value<'js>> {
        self.0.from_transfer_object(ctx, obj)
    }

    pub fn to_transfer_object<'js>(
        &self,
        ctx: &mut SerializationContext<'js, '_>,
        value: &Value<'js>,
    ) -> rquickjs::Result<TransferData> {
        self.0.to_transfer_object(ctx, value)
    }
}

pub trait DynCloner {
    fn tag(&self) -> &Tag;
    fn from_transfer_object<'js>(
        &self,
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Value<'js>>;

    fn to_transfer_object<'js>(
        &self,
        ctx: &mut SerializationContext<'js, '_>,
        value: &Value<'js>,
    ) -> rquickjs::Result<TransferData>;
}

struct ClonerImpl<T>(PhantomData<T>);

impl<T> DynCloner for ClonerImpl<T>
where
    T: StructuredClone,
{
    fn tag(&self) -> &Tag {
        T::tag()
    }
    fn from_transfer_object<'js>(
        &self,
        ctx: &mut SerializationContext<'js, '_>,
        obj: TransferData,
    ) -> rquickjs::Result<Value<'js>> {
        let value = T::from_transfer_object(ctx, obj)?;
        value.into_js(ctx.ctx())
    }

    fn to_transfer_object<'js>(
        &self,
        ctx: &mut SerializationContext<'js, '_>,
        value: &Value<'js>,
    ) -> rquickjs::Result<TransferData> {
        let value = T::Item::from_js(ctx.ctx(), value.clone())?;

        let data = T::to_transfer_object(ctx, &value)?;

        Ok(data)
    }
}
