use klaver_util::rquickjs::{self, Class, Ctx, FromJs, Function, String, prelude::IntoArgs};

use crate::{
    async_state::AsyncState,
    exec_state::AsyncId,
    exec_state::ExecState,
    listener::{HookListeners, get_listeners},
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
        let hook_list = get_listeners(&ctx)?;
        Ok(TaskCtx {
            ctx,
            exec,
            id,
            hook_list,
            ty,
        })
    }

    pub(crate) fn init(&self, parent_id: Option<AsyncId>) -> rquickjs::Result<()> {
        self.hook_list
            .borrow_mut()
            .init(&self.ctx, self.id, self.ty.clone(), parent_id)
    }
}

impl<'js> TaskCtx<'js> {
    pub fn invoke_callback<A, R>(&self, cb: Function<'js>, args: A) -> rquickjs::Result<R>
    where
        A: IntoArgs<'js>,
        R: FromJs<'js>,
    {
        self.hook_list.borrow().before(&self.ctx, self.id.clone())?;
        let ret = AsyncState::get(&self.ctx)?.exec.enter(self.id, || {
            let ret = cb.call(args);
            ret
        });
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
