use klaver_util::rquickjs::{self, AsyncRuntime, String, Symbol, promise::PromiseHookType};

use crate::{AsyncState, exec_state::AsyncId, state::HookState};

pub async fn set_promise_hook(runtime: &AsyncRuntime) {
    runtime
        .set_promise_hook(Some(Box::new(|ctx, hook, promise, parent| {
            let Ok(state) = AsyncState::get(&ctx) else {
                println!("Could noy uptain state");
                return;
            };

            let hook_state = HookState::get(&ctx).unwrap();

            // println!("HOOK {:?}", hook);

            let Some(promise) = promise.as_object() else {
                // Promise an object
                println!("Promise not a project");
                return;
            };

            let parent_id = parent
                .as_object()
                .and_then(|m| m.get::<_, AsyncId>("$aid").ok());

            // println!("PARENT {:?} {:?}", parent_id, parent);

            match hook {
                PromiseHookType::Init => {
                    let id = state.exec.create_task(parent_id, true);
                    // println!("ID {:?} {:?}", id, parent_id);
                    promise.set("$aid", id).unwrap();

                    hook_state
                        .borrow()
                        .registry
                        .register(promise.clone().into_value(), id)
                        .expect("Register promise");

                    let ty = String::from_str(ctx.clone(), "PROMISE").unwrap();

                    hook_state
                        .borrow()
                        .hooks
                        .borrow_mut()
                        .init(
                            &ctx,
                            id,
                            ty,
                            Some(parent_id.unwrap_or_else(|| state.exec.trigger_async_id())),
                        )
                        .unwrap();

                    // println!("AFTER ID {:?}", id);

                    state.exec.set_current(id);

                    // let id = state.exec.create_task(parent);
                }
                PromiseHookType::Resolve => {
                    let id: AsyncId = promise.get("$aid").unwrap();
                    // println!("Resolve: {:?}", id);
                    // state.exec.destroy_task(id);
                }
                _ => {
                    println!("Unknown");
                }
            }
        })))
        .await
}
