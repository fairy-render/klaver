use klaver_util::{
    rquickjs::{self, Ctx, FromJs, JsLifetime},
    throw_if,
};

use crate::{
    context::Context,
    executor::{TaskExecutor, TaskHandle},
    resource::{Resource, ResourceKind},
    runtime::Runtime,
    task_manager::TaskManager,
};

#[derive(Clone)]
pub struct AsyncState {
    tasks: TaskManager,
}

unsafe impl<'js> JsLifetime<'js> for AsyncState {
    type Changed<'to> = AsyncState;
}

impl AsyncState {
    // pub fn instance(ctx: &Ctx<'_>) -> rquickjs::Result<AsyncState> {
    //     match ctx.userdata::<Self>() {
    //         Some(ret) => Ok(ret.clone()),
    //         None => {
    //             let _ = throw_if!(
    //                 ctx,
    //                 ctx.store_userdata(AsyncState {
    //                     exec: Default::default(),
    //                     exception: Rc::new(ObservableRefCell::new(None)),
    //                     resource_map: Rc::new(RefCell::new(ResourceMap::new()))
    //                 })
    //             );

    //             Ok(ctx.userdata::<Self>().unwrap().clone())
    //         }
    //     }
    // }

    pub fn push<'js, T: Resource<'js> + 'js>(
        ctx: &Ctx<'js>,
        resource: T,
    ) -> rquickjs::Result<TaskHandle> {
        let runtime = Runtime::from_ctx(ctx)?;
        let executor = TaskExecutor::new(&*runtime.borrow());
        executor.push(ctx, resource)
    }

    pub async fn run<'js, T, R>(ctx: &Ctx<'js>, runner: T) -> rquickjs::Result<R>
    where
        T: AsyncFnOnce(Context<'js>) -> rquickjs::Result<R>,
        R: FromJs<'js>,
    {
        let runtime = Runtime::from_ctx(ctx)?;
        let executor = TaskExecutor::new(&*runtime.borrow());
        executor.run_async(ctx, ResourceKind::ROOT, runner).await
    }
}
