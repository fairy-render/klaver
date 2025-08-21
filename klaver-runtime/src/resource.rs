use core::fmt;
use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashMap},
};

use klaver_util::rquickjs::{self, Ctx, FromJs, IntoJs};

use crate::context::Context;

pub trait Resource<'js>: Sized {
    type Id: ResourceId;
    const INTERNAL: bool = true;
    const SCOPED: bool = true;

    fn run(self, ctx: Context<'js>) -> impl Future<Output = rquickjs::Result<()>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKind(pub(crate) u32);

impl ResourceKind {
    pub const PROMISE: ResourceKind = ResourceKind(1);
    pub const SCRIPT: ResourceKind = ResourceKind(2);
    pub const ROOT: ResourceKind = ResourceKind(3);
    pub const STORAGE: ResourceKind = ResourceKind(4);

    pub fn is_native(&self) -> bool {
        self.0 > 2
    }
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'js> IntoJs<'js> for ResourceKind {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        self.0.into_js(ctx)
    }
}

impl<'js> FromJs<'js> for ResourceKind {
    fn from_js(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        Ok(ResourceKind(value.get()?))
    }
}

const NEXT_ID: u32 = 5;

pub(crate) struct ResourceMap {
    next_id: u32,
    type_map: HashMap<TypeId, ResourceKind>,
    name_map: HashMap<ResourceKind, String>,
}

impl ResourceMap {
    pub fn new() -> ResourceMap {
        ResourceMap {
            next_id: NEXT_ID,
            type_map: Default::default(),
            name_map: Default::default(),
        }
    }
}

impl ResourceMap {
    pub fn register<'js, T>(&mut self) -> ResourceKind
    where
        T: Resource<'js>,
    {
        let type_id = TypeId::of::<T::Id>();

        if let Some(id) = self.type_map.get(&type_id) {
            *id
        } else {
            let kind = self.next_id;
            self.next_id = kind + 1;
            let kind = ResourceKind(kind);
            self.type_map.insert(type_id, kind);
            self.name_map.insert(kind, T::Id::name().to_string());
            kind
        }
    }

    pub fn name(&self, id: ResourceKind) -> Option<&str> {
        if id == ResourceKind::PROMISE {
            Some("Promise")
        } else if id == ResourceKind::ROOT {
            Some("Root")
        } else if id == ResourceKind::SCRIPT {
            Some("Script")
        } else if id == ResourceKind::STORAGE {
            Some("AsyncLocalStorage")
        } else {
            self.name_map.get(&id).map(|m| &**m)
        }
    }
}

pub trait ResourceId: Any {
    fn name() -> &'static str;
}
