use klaver_util::{
    rquickjs::{self, Ctx, JsLifetime},
    throw_if,
};

use crate::task_manager::TaskManager;

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
}
