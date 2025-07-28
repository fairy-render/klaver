use klaver_util::{
    rquickjs::{self, Ctx, JsLifetime, String},
    throw_if,
};

use crate::{
    exec_state::ExecState,
    resource::{Resource, TaskCtx},
};

#[derive(Clone)]
pub struct AsyncState {
    pub(crate) exec: ExecState,
}

unsafe impl<'js> JsLifetime<'js> for AsyncState {
    type Changed<'to> = AsyncState;
}

impl AsyncState {
    pub fn get(ctx: &Ctx<'_>) -> rquickjs::Result<AsyncState> {
        match ctx.userdata::<Self>() {
            Some(ret) => Ok(ret.clone()),
            None => {
                let _ = throw_if!(
                    ctx,
                    ctx.store_userdata(AsyncState {
                        exec: Default::default()
                    })
                );

                Ok(ctx.userdata::<Self>().unwrap().clone())
            }
        }
    }

    pub fn push<'js, T: Resource<'js> + 'js>(
        &self,
        ctx: &Ctx<'js>,
        resource: T,
    ) -> rquickjs::Result<()> {
        let parent_id = self.exec.trigger_async_id();
        let id = self.exec.create_task(parent_id);

        let ctx = ctx.clone();

        let ty = String::from_str(ctx.clone(), resource.ty())?;

        let exec = self.exec.clone();

        let task_ctx = TaskCtx::new(ctx.clone(), exec.clone(), ty, id)?;

        ctx.spawn(async move {
            task_ctx.init(parent_id);

            let ret = resource.run(task_ctx).await;

            loop {
                println!("Wait on child id({id:?}): {}", exec.child_count(id));
                if exec.child_count(id) == 0 {
                    break;
                }

                exec.listen().await
            }

            exec.destroy_task(id);
        });

        Ok(())
    }
}

impl AsyncState {
    pub async fn run<'js, T, U, R>(&self, ctx: Ctx<'js>, func: T) -> rquickjs::Result<R>
    where
        T: FnOnce(TaskCtx<'js>) -> U,
        U: Future<Output = rquickjs::Result<R>>,
    {
        let id = self.exec.create_task(None);

        let ctx = ctx.clone();

        let ty = String::from_str(ctx.clone(), "entry")?;

        let task_ctx = TaskCtx::new(ctx.clone(), self.exec.clone(), ty, id)?;

        let ret = self.exec.enter(id, || func(task_ctx)).await;

        self.exec.shutdown(id).await?;

        self.exec.destroy_task(id);

        ret
    }
}
