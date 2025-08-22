use std::rc::Rc;

use klaver_util::{
    CaugthException,
    rquickjs::{self, Class, Ctx, FromJs, Function, Value, function::Args, prelude::IntoArgs},
    sync::ObservableRefCell,
    throw,
};

use crate::{
    id::AsyncId,
    listener::{HookListeners, ResourceHandle},
    resource::ResourceKind,
    runtime::Runtime,
    task_manager::TaskManager,
};

pub struct Context<'js> {
    pub id: AsyncId,
    pub tasks: TaskManager,
    pub hooks: Class<'js, HookListeners<'js>>,
    pub exception: Rc<ObservableRefCell<Option<CaugthException>>>,
    pub internal: bool,
    pub ctx: Ctx<'js>,
}

impl<'js> Context<'js> {
    pub(crate) fn new(
        ctx: Ctx<'js>,
        runtime: &Runtime<'js>,
        id: AsyncId,
        kind: ResourceKind,
        internal: bool,
    ) -> Context<'js> {
        Context {
            id,
            tasks: runtime.manager.clone(),
            hooks: runtime.hooks.clone(),
            exception: runtime.exception.clone(),
            internal,
            ctx,
        }
    }

    pub fn handle(&self) -> rquickjs::Result<ResourceHandle<'js>> {
        self.hooks.borrow().get_resource_handle(&self.ctx, self.id)
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }

    pub fn invoke_callback<A, R>(&self, cb: Function<'js>, args: A) -> rquickjs::Result<R>
    where
        A: IntoArgs<'js>,
        R: FromJs<'js>,
    {
        if self.internal {
            throw!(@internal self.ctx, "Internal resource cannot have children");
        };

        let id = self.tasks.exectution_trigger_id();

        self.hooks.borrow().before(&self.ctx, self.id.clone())?;

        self.tasks.set_current(self.id);
        let ret = cb.call(args);
        self.tasks.set_current(id);

        self.hooks.borrow().after(&self.ctx, self.id.clone())?;
        ret
    }

    pub fn invoke_callback_arg<R>(&self, cb: Function<'js>, args: Args<'js>) -> rquickjs::Result<R>
    where
        R: FromJs<'js>,
    {
        if self.internal {
            throw!(@internal self.ctx, "Internal resource cannot have children");
        };

        let id = self.tasks.exectution_trigger_id();

        self.hooks.borrow().before(&self.ctx, self.id.clone())?;

        self.tasks.set_current(self.id);
        let ret = cb.call_arg(args);
        self.tasks.set_current(id);

        self.hooks.borrow().after(&self.ctx, self.id.clone())?;
        ret
    }

    pub fn incr(&self) {
        self.tasks.increment_ref(self.id);
    }
}
