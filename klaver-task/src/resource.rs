use klaver_util::rquickjs::{self, Class, Ctx, FromJs, Function, IntoJs, prelude::IntoArgs};

use crate::{
    exec_state::{AsyncId, ExecState},
    listener::HookListeners,
    state::HookState,
};

#[derive(Clone)]
pub struct TaskCtx<'js> {
    pub ctx: Ctx<'js>,
    pub id: AsyncId,
    pub kind: ResourceKind,
    pub hook_list: Class<'js, HookListeners<'js>>,
    pub exec: ExecState,
}

impl<'js> TaskCtx<'js> {
    pub(crate) fn new(
        ctx: Ctx<'js>,
        exec: ExecState,
        kind: ResourceKind,
        id: AsyncId,
    ) -> rquickjs::Result<TaskCtx<'js>> {
        let hook_list = HookState::get(&ctx)?.borrow().hooks.clone();
        Ok(TaskCtx {
            ctx,
            exec,
            id,
            hook_list,
            kind,
        })
    }

    pub(crate) fn init(&self) -> rquickjs::Result<()> {
        let parent_id = self.exec.parent_id(self.id);

        self.hook_list
            .borrow_mut()
            .init(&self.ctx, self.id, self.kind, Some(parent_id))
    }

    pub(crate) fn destroy(self) -> rquickjs::Result<()> {
        self.hook_list.borrow_mut().destroy(&self.ctx, self.id)?;
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
        self.hook_list.borrow().before(&self.ctx, self.id.clone())?;

        self.exec.set_current(self.id);
        let ret = cb.call(args);

        self.hook_list.borrow().after(&self.ctx, self.id.clone())?;
        ret
    }

    pub async fn wait_shutdown(&self) -> rquickjs::Result<()> {
        self.exec.wait_shutdown(self.id).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKind(pub(crate) u32);

impl ResourceKind {
    pub const PROMISE: ResourceKind = ResourceKind(1);
    pub const SCRIPT: ResourceKind = ResourceKind(2);
    pub const ROOT: ResourceKind = ResourceKind(3);

    pub fn is_native(&self) -> bool {
        self.0 >= 2
    }
}

impl<'js> IntoJs<'js> for ResourceKind {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        self.0.into_js(ctx)
    }
}

pub(crate) const NEXT_ID: u32 = 4;

pub trait Resource<'js>: Sized {
    type Id: std::any::Any;
    fn run(&self, ctx: TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<()>>;
}
