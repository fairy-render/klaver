use klaver_util::rquickjs::{self, AsyncRuntime, Ctx, IntoJs, Value, promise::PromiseHookType};

use crate::{AsyncId, ResourceKind, runtime::Runtime};

fn promise_hook<'js>(
    ctx: Ctx<'js>,
    hook: PromiseHookType,
    promise: Value<'js>,
    parent: Value<'js>,
) -> rquickjs::Result<()> {
    let runtime = Runtime::from_ctx(&ctx)?;

    let (manager, finalizers, hooks) = {
        let runtime = runtime.borrow();
        (
            runtime.manager.clone(),
            runtime.finalizers.clone(),
            runtime.hooks.clone(),
        )
    };

    let Some(promise) = promise.as_object() else {
        // Promise an object
        println!("Promise not a project");
        return Ok(());
    };

    let parent_id = parent
        .as_object()
        .and_then(|m| m.get::<_, AsyncId>("$aid").ok());

    // println!(
    //     "Hook {:?} {:?}: {}",
    //     hook,
    //     promise.get::<_, Value<'js>>("$aid"),
    //     manager.exectution_trigger_id()
    // );

    match hook {
        PromiseHookType::Init => {
            let id = manager.create_task(parent_id, ResourceKind::PROMISE, false, false);
            promise.set("$aid", id)?;

            // println!("CREATE PROMISE {}", id);

            finalizers.register(promise.clone().into_value(), id.into_js(&ctx)?, None)?;

            hooks.borrow().init(
                &ctx,
                id,
                ResourceKind::PROMISE,
                Some(parent_id.unwrap_or_else(|| manager.trigger_async_id())),
            )?;

            // manager.set_current(id);
        }
        PromiseHookType::Resolve => {
            let id: AsyncId = promise.get("$aid")?;

            hooks.borrow().promise_resolve(&ctx, id)?;
            manager.set_current(id);
        }
        PromiseHookType::Before => {
            let id: AsyncId = promise.get("$aid")?;
            hooks.borrow().before(&ctx, id)?;
            manager.set_current(id);
        }
        PromiseHookType::After => {
            let id: AsyncId = promise.get("$aid")?;
            hooks.borrow().after(&ctx, id)?;
            manager.set_current(id);
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
