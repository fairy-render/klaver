use core::fmt;
use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
};

use klaver_util::{
    rquickjs::{self, Class, Ctx, FromJs, Function, IntoJs, prelude::IntoArgs},
    throw,
};

use crate::{
    exec_state::{AsyncId, ExecState},
    listener::HookListeners,
    state::HookState,
};

#[derive(Clone)]
pub struct TaskCtx<'js> {
    ctx: Ctx<'js>,
    id: AsyncId,
    kind: ResourceKind,
    hook_list: Class<'js, HookListeners<'js>>,
    exec: ExecState,
    internal: bool,
}

impl<'js> TaskCtx<'js> {
    pub(crate) fn new(
        ctx: Ctx<'js>,
        exec: ExecState,
        kind: ResourceKind,
        id: AsyncId,
        internal: bool,
    ) -> rquickjs::Result<TaskCtx<'js>> {
        let hook_list = HookState::get(&ctx)?.borrow().hooks.clone();
        Ok(TaskCtx {
            ctx,
            exec,
            id,
            hook_list,
            kind,
            internal,
        })
    }

    pub(crate) fn init(&self) -> rquickjs::Result<()> {
        let parent_id = self.exec.parent_id(self.id);

        if !self.internal {
            self.hook_list
                .borrow_mut()
                .init(&self.ctx, self.id, self.kind, Some(parent_id))?;
        }

        Ok(())
    }

    pub(crate) fn destroy(self) -> rquickjs::Result<()> {
        if !self.internal {
            self.hook_list.borrow_mut().destroy(&self.ctx, self.id)?;
        }

        self.exec.destroy_task(self.id);

        Ok(())
    }
}

impl<'js> TaskCtx<'js> {
    pub fn invoke_callback<A, R>(&self, cb: Function<'js>, args: A) -> rquickjs::Result<R>
    where
        A: IntoArgs<'js>,
        R: FromJs<'js>,
    {
        if self.internal {
            throw!(@internal self.ctx, "Internal resource cannot have children");
        };

        self.hook_list.borrow().before(&self.ctx, self.id.clone())?;

        self.exec.set_current(self.id);
        let ret = cb.call(args);

        self.hook_list.borrow().after(&self.ctx, self.id.clone())?;
        ret
    }

    pub fn id(&self) -> AsyncId {
        self.id
    }

    pub fn kind(&self) -> ResourceKind {
        self.kind
    }

    /// Wait for this task to get the shutdown signal
    ///
    /// This just mean that the task this task is attached to is shut down
    /// The resource does not have to shutdown it self.
    /// Eg. timers should not shut down
    pub async fn wait_shutdown(&self) -> rquickjs::Result<()> {
        self.exec.wait_shutdown(self.id).await
    }

    pub fn is_shutdown(&self) -> bool {
        self.exec.is_shutdown(self.id)
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }
}

impl<'js> std::ops::Deref for TaskCtx<'js> {
    type Target = Ctx<'js>;
    fn deref(&self) -> &Self::Target {
        self.ctx()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKind(pub(crate) u32);

impl ResourceKind {
    pub const PROMISE: ResourceKind = ResourceKind(1);
    pub const SCRIPT: ResourceKind = ResourceKind(2);
    pub const ROOT: ResourceKind = ResourceKind(3);

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

const NEXT_ID: u32 = 4;

pub(crate) struct ResourceMap {
    next_id: u32,
    type_map: BTreeMap<TypeId, ResourceKind>,
    name_map: BTreeMap<ResourceKind, String>,
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
        } else {
            self.name_map.get(&id).map(|m| &**m)
        }
    }
}

pub trait ResourceId: Any {
    fn name() -> &'static str;
}

pub trait Resource<'js>: Sized {
    type Id: ResourceId;
    const INTERNAL: bool = false;
    fn run(self, ctx: TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<()>>;
}
