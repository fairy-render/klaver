use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use klaver_util::{AsContext, throw};
use rquickjs::{Ctx, Value};

use super::{ObjectId, Registry, TransObject, get_tag_value, registry::SerializationOptions};

pub struct SerializationContext<'js, 'a> {
    ctx: Ctx<'js>,
    registry: &'a Registry,
    ser_cache: Rc<RefCell<HashMap<Value<'js>, u32>>>,
    de_cache: Rc<RefCell<HashMap<u32, Value<'js>>>>,
    opts: &'a SerializationOptions<'js>,
    next_id: Rc<Cell<u32>>,
    id: u32,
}

impl<'js, 'a> SerializationContext<'js, 'a> {
    pub(super) fn new(
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
            next_id: Rc::new(Cell::new(2)),
            id: 1,
        }
    }

    fn next_id(&self) -> u32 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    pub fn id(&self) -> ObjectId {
        ObjectId(self.id)
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }

    pub fn should_move(&self, value: &Value<'js>) -> bool {
        let Some(transfer) = &self.opts.transfer else {
            return false;
        };

        transfer.contains(value)
    }

    pub(crate) fn cache_value(&self, value: impl AsRef<Value<'js>>) {
        self.ser_cache
            .borrow_mut()
            .insert(value.as_ref().clone(), self.id);
        self.de_cache
            .borrow_mut()
            .insert(self.id, value.as_ref().clone());
    }

    pub fn from_transfer_object(&mut self, value: TransObject) -> rquickjs::Result<Value<'js>> {
        match value {
            TransObject::Data { tag, data, id } => {
                let cloner = self.registry.get_by_tag(&self.ctx, &tag)?;

                let mut ctx = Self {
                    ctx: self.ctx.clone(),
                    registry: self.registry,
                    ser_cache: self.ser_cache.clone(),
                    de_cache: self.de_cache.clone(),
                    opts: self.opts,
                    next_id: self.next_id.clone(),
                    id: id.0,
                };

                let value = cloner.from_transfer_object(&mut ctx, data)?;

                if !value.is_bool() && !value.is_number() {
                    ctx.cache_value(&value);
                }

                Ok(value)
            }
            TransObject::Ref { id } => {
                let Some(value) = self.de_cache.borrow().get(&id.0).cloned() else {
                    throw!(@type self.ctx(), "ObjectId not found in cache")
                };
                Ok(value)
            }
        }
    }

    pub fn to_transfer_object(&mut self, value: &Value<'js>) -> rquickjs::Result<TransObject> {
        if let Some(id) = self.ser_cache.borrow().get(value).copied() {
            return Ok(TransObject::Ref { id: ObjectId(id) });
        }
        let tag = get_tag_value(&self.ctx, value)?;
        let cloner = self.registry.get_by_tag(&self.ctx, &tag)?;

        let mut ctx = Self {
            ctx: self.ctx.clone(),
            registry: self.registry,
            ser_cache: self.ser_cache.clone(),
            de_cache: self.de_cache.clone(),
            opts: self.opts,
            next_id: self.next_id.clone(),
            id: self.next_id(),
        };

        let data = cloner.to_transfer_object(&mut ctx, value)?;

        if !value.is_bool() && !value.is_number() {
            ctx.cache_value(value);
        }

        Ok(TransObject::Data {
            tag,
            data,
            id: ctx.id(),
        })
    }
}

impl<'js, 'a> AsContext<'js> for SerializationContext<'js, 'a> {
    fn as_ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }
}
