use klaver_util::rquickjs::{self, AsyncRuntime, Ctx, Value, promise::PromiseHookType};

use crate::{AsyncState, ResourceKind, exec_state::AsyncId, state::HookState};

fn promise_hook<'js>(
    ctx: Ctx<'js>,
    hook: PromiseHookType,
    promise: Value<'js>,
    parent: Value<'js>,
) -> rquickjs::Result<()> {
    let Ok(state) = AsyncState::get(&ctx) else {
        println!("Could noy uptain state");
        return Ok(());
    };

    let hook_state = HookState::get(&ctx)?;

    let Some(promise) = promise.as_object() else {
        // Promise an object
        println!("Promise not a project");
        return Ok(());
    };

    let parent_id = parent
        .as_object()
        .and_then(|m| m.get::<_, AsyncId>("$aid").ok());

    let symbol = hook_state.borrow().promise_symbol.clone();

    match hook {
        PromiseHookType::Init => {
            let id = state.exec.create_task(parent_id, ResourceKind::Promise);
            promise.set(symbol, id)?;

            hook_state
                .borrow()
                .registry
                .register(promise.clone().into_value(), id)?;

            hook_state.borrow().hooks.borrow_mut().init(
                &ctx,
                id,
                ResourceKind::Promise,
                Some(parent_id.unwrap_or_else(|| state.exec.trigger_async_id())),
            )?;

            state.exec.set_current(id);
        }
        PromiseHookType::Resolve => {
            let id: AsyncId = promise.get(symbol)?;

            hook_state
                .borrow()
                .hooks
                .borrow_mut()
                .promise_resolve(&ctx, id)?;
        }
        _ => {
            println!("Unknown");
        }
    }
    Ok(())
}

pub async fn set_promise_hook(runtime: &AsyncRuntime) {
    runtime
        .set_promise_hook(Some(Box::new(|ctx, hook, promise, parent| {
            if let Err(err) = promise_hook(ctx, hook, promise, parent) {
                println!("Promise hook failed: {err}");
            }
        })))
        .await
}
