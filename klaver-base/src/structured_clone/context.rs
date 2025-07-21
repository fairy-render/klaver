use rquickjs::{Ctx, Value};

use crate::{
    Registry, TransObject, get_tag_value, structured_clone::registry::SerializationOptions,
};

#[derive(Clone)]
pub struct SerializationContext<'js, 'a> {
    ctx: Ctx<'js>,
    registry: &'a Registry,
    opts: &'a SerializationOptions<'js>,
}

impl<'js, 'a> SerializationContext<'js, 'a> {
    pub fn new(
        ctx: Ctx<'js>,
        registry: &'a Registry,
        opts: &'a SerializationOptions<'js>,
    ) -> SerializationContext<'js, 'a> {
        Self {
            ctx,
            registry,
            opts,
        }
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }

    pub fn from_transfer_object(&mut self, value: TransObject) -> rquickjs::Result<Value<'js>> {
        let cloner = self.registry.get_by_tag(&self.ctx, &value.tag)?;
        cloner.from_transfer_object(self, value)
    }

    pub fn to_transfer_object(&mut self, value: &Value<'js>) -> rquickjs::Result<TransObject> {
        let tag = get_tag_value(&self.ctx, value)?;
        let cloner = self.registry.get_by_tag(&self.ctx, &tag)?;
        cloner.to_transfer_object(self, value)
    }
}
