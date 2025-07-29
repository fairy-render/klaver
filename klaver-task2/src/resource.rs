use klaver_util::rquickjs::{self, Class, Ctx, FromJs, Function, String, prelude::IntoArgs};

use crate::{
    async_state::AsyncState,
    exec_state::{AsyncId, ExecState},
    listener::HookListeners,
    state::HookState,
};

pub struct TaskCtx<'js> {
    pub ctx: Ctx<'js>,
    pub id: AsyncId,
    pub ty: String<'js>,
    pub hook_list: Class<'js, HookListeners<'js>>,
    pub exec: ExecState,
}

impl<'js> TaskCtx<'js> {
    pub(crate) fn new(
        ctx: Ctx<'js>,
        exec: ExecState,
        ty: String<'js>,
        id: AsyncId,
    ) -> rquickjs::Result<TaskCtx<'js>> {
        let hook_list = HookState::get(&ctx)?.borrow().hooks.clone();
        Ok(TaskCtx {
            ctx,
            exec,
            id,
            hook_list,
            ty,
        })
    }

    pub(crate) fn init(&self) -> rquickjs::Result<()> {
        let parent_id = self.exec.parent_id(self.id);

        self.hook_list
            .borrow_mut()
            .init(&self.ctx, self.id, self.ty.clone(), Some(parent_id))
    }
}

impl<'js> TaskCtx<'js> {
    pub fn invoke_callback<A, R>(&self, cb: Function<'js>, args: A) -> rquickjs::Result<R>
    where
        A: IntoArgs<'js>,
        R: FromJs<'js>,
    {
        self.hook_list.borrow().before(&self.ctx, self.id.clone())?;
        let current_id = self.exec.trigger_async_id();

        self.exec.set_current(self.id);

        let ret = cb.call(args);

        self.hook_list.borrow().after(&self.ctx, self.id.clone())?;
        ret
    }

    pub async fn wait_shutdown(&self) -> rquickjs::Result<()> {
        self.exec.wait_shutdown(self.id).await
    }
}

pub trait Resource<'js>: Sized {
    fn ty(&self) -> &str;
    fn run(&self, ctx: TaskCtx<'js>) -> impl Future<Output = rquickjs::Result<()>>;
}
