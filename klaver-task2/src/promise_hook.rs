use klaver_util::rquickjs::{self, AsyncRuntime, promise::PromiseHookType};

use crate::AsyncState;

pub async fn set_promise_hook(runtime: &AsyncRuntime) {
    runtime
        .set_promise_hook(Some(Box::new(|ctx, hook, promise, parent| {
            let Ok(state) = AsyncState::get(&ctx) else {
                println!("Could noy uptain state");
                return;
            };

            println!("HOOK {:?}", hook);

            match hook {
                PromiseHookType::Init => {
                    let parent = state.exec.trigger_async_id();
                    let id = state.exec.create_task(parent);
                }
                PromiseHookType::Resolve => {}
                _ => {}
            }
        })))
        .await
}
