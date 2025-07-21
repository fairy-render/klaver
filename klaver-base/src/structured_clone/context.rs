use std::collections::HashMap;

use rquickjs::{Ctx, Value};
use rquickjs_util::util::AsContext;

use super::{ObjectId, Registry, TransObject, get_tag_value, registry::SerializationOptions};

#[derive(Clone)]
pub struct SerializationContext<'js, 'a> {
    ctx: Ctx<'js>,
    registry: &'a Registry,
    ser_cache: HashMap<Value<'js>, usize>,
    de_cache: HashMap<usize, Value<'js>>,
    opts: &'a SerializationOptions<'js>,
    next_id: u32,
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
            ser_cache: Default::default(),
            de_cache: Default::default(),
            opts,
            next_id: 1,
        }
    }

    pub(super) fn next_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        ObjectId(id)
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

impl<'js, 'a> AsContext<'js> for SerializationContext<'js, 'a> {
    fn as_ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }
}
