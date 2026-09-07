use crate::{
    WinterTcInstance,
    channel::{MessageChannel, MessagePort},
    events::{Emitter, EventCallback, EventKey},
};
use klaver_core::{Exportable, Registry, value::structured_clone::SerializationOptions};
use klaver_modules::WeakEnviron;
use klaver_runtime::{AsyncState, TaskHandle};
use rquickjs::{
    Class, Ctx, Function, JsLifetime, Value,
    class::{JsClass, Trace},
    prelude::Opt,
};

use super::resource::WorkerResource;

#[rquickjs::class(rename = "Worker")]
pub struct WebWorker<'js> {
    port: Class<'js, MessagePort<'js>>,
    onmessage: Option<Function<'js>>,
    handle: Option<TaskHandle>,
}

impl<'js> Trace<'js> for WebWorker<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.port.trace(tracer);
        self.onmessage.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for WebWorker<'js> {
    type Changed<'to> = WebWorker<'to>;
}

#[rquickjs::methods]
impl<'js> WebWorker<'js> {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        path: std::string::String,
    ) -> rquickjs::Result<Class<'js, WebWorker<'js>>> {
        let registry = Registry::instance(&ctx)?;

        let channel = MessageChannel::new(ctx.clone())?;

        let port = channel.port1;
        let channel = channel.port2.borrow_mut().detach(&ctx)?;

        let env = ctx
            .userdata::<WeakEnviron>()
            .unwrap()
            .clone()
            .upgrade(&ctx)?;

        let winter = WinterTcInstance::from_ctx(&ctx)?;

        let resource = WorkerResource::new(
            path,
            env,
            channel,
            registry,
            winter.borrow().backend().clone(),
        );

        let handle = AsyncState::push(&ctx, resource)?;

        MessagePort::start_native(&ctx, port.clone())?;

        let this = Class::instance(
            ctx.clone(),
            WebWorker {
                port,
                onmessage: None,
                handle: Some(handle),
            },
        )?;

        Ok(this)
    }

    #[qjs(rename = "postMessage")]
    pub fn post_message(
        &self,
        ctx: Ctx<'js>,
        value: Value<'js>,
        opts: Opt<SerializationOptions<'js>>,
    ) -> rquickjs::Result<()> {
        self.port.borrow().post_message(ctx, value, opts)?;

        Ok(())
    }

    #[qjs(rename = "addEventListener")]
    pub fn add_event_listener(
        &self,
        ctx: Ctx<'js>,
        event: EventKey<'js>,
        cb: EventCallback<'js>,
    ) -> rquickjs::Result<()> {
        MessagePort::add_event_listener_native(&self.port, &ctx, event, cb, Default::default())
    }

    #[qjs(rename = "removeEventListener")]
    pub fn remove_event_listener(
        &self,
        event: EventKey<'js>,
        cb: EventCallback<'js>,
    ) -> rquickjs::Result<()> {
        self.port
            .borrow_mut()
            .remove_event_listener_native(event, cb, false);
        Ok(())
    }

    #[qjs(set, rename = "onmessage")]
    pub fn set_onmessage(&mut self, ctx: Ctx<'js>, cb: Opt<Function<'js>>) -> rquickjs::Result<()> {
        self.port.borrow_mut().set_onmessage(ctx, cb.0)
    }

    #[qjs(get, rename = "onmessage")]
    pub fn get_onmessage(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<Function<'js>>> {
        self.port.borrow().get_onmessage(ctx)
    }

    /// Per the spec's `AbstractWorker` mixin. Fired (as a [`crate::channel::MessageEvent`],
    /// carrying a description of the failure as `data` - there's no dedicated `ErrorEvent` type
    /// yet, see `MISSING_APIS.md`) when the worker's module fails during its top-level
    /// evaluation - see `WorkerResource::run` in `resource.rs`.
    #[qjs(set, rename = "onerror")]
    pub fn set_onerror(&mut self, ctx: Ctx<'js>, cb: Opt<Function<'js>>) -> rquickjs::Result<()> {
        self.port.borrow_mut().set_handler(
            EventKey::from_str(ctx, "error")?,
            cb.0.map(EventCallback::Function),
        );
        Ok(())
    }

    #[qjs(get, rename = "onerror")]
    pub fn get_onerror(&self, ctx: Ctx<'js>) -> rquickjs::Result<Option<Function<'js>>> {
        Ok(self
            .port
            .borrow()
            .get_handler_function(&EventKey::from_str(ctx, "error")?))
    }

    pub fn terminate(&mut self) -> rquickjs::Result<()> {
        let Some(handle) = self.handle.take() else {
            return Ok(());
        };
        handle.kill();
        self.port.borrow_mut().close();

        Ok(())
    }
}

impl<'js> Exportable<'js> for WebWorker<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            WebWorker::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        // Self::inherit(ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        time::Duration,
    };

    use klaver_modules::{loaders::FileLoader, resolvers::FileResolver};
    use klaver_vm::{Options, Vm};
    use rquickjs::CatchResultExt;

    async fn build_vm(work_dir: &Path) -> Vm {
        Options::default()
            .global::<crate::WinterTC>()
            .loader(FileLoader::default().with_transformer(()))
            .resolver(FileResolver::new_with(
                work_dir.to_path_buf(),
                Default::default(),
            ))
            .build()
            .await
            .unwrap()
    }

    fn scratch_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "klaver-wintertc-worker-tests-{}-{test_name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_script(dir: &Path, name: &str, source: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, source).unwrap();
        path
    }

    /// Runs `body` as the contents of an `async` IIFE against `vm`, under a timeout so a stuck
    /// worker thread (e.g. a message that never arrives) fails the test instead of hanging it.
    /// `body` is expected to throw on failure.
    async fn run(vm: &Vm, body: String) {
        let fut = vm.async_with(async move |ctx| {
            let promise: rquickjs::Promise = ctx
                .eval(format!("(async () => {{\n{body}\n}})()"))
                .catch(&ctx)?;
            promise.into_future::<()>().await.catch(&ctx)?;
            Ok(())
        });

        tokio::time::timeout(Duration::from_secs(10), fut)
            .await
            .expect("worker test timed out")
            .unwrap();
    }

    #[tokio::test]
    async fn postmessage_round_trips_between_main_and_worker() {
        let dir = scratch_dir("round_trip");
        let script = write_script(
            &dir,
            "doubler.mjs",
            r#"
                addEventListener("message", (event) => {
                    postMessage(event.data * 2);
                });
            "#,
        );
        let vm = build_vm(&dir).await;

        run(
            &vm,
            format!(
                r#"
                const worker = new Worker({path:?});
                const doubled = await new Promise((resolve) => {{
                    worker.onmessage = (e) => resolve(e.data);
                    worker.postMessage(21);
                }});
                if (doubled !== 42) throw new Error(`expected 42, got ${{doubled}}`);
                worker.terminate();
                "#,
                path = script.display()
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn worker_global_scope_exposes_self_and_postmessage() {
        let dir = scratch_dir("global_scope");
        let script = write_script(
            &dir,
            "greeter.mjs",
            r#"
                if (typeof self === "undefined" || self !== globalThis) {
                    throw new Error("`self` is not aliased to the worker's global object");
                }
                self.postMessage("hello");
            "#,
        );
        let vm = build_vm(&dir).await;

        run(
            &vm,
            format!(
                r#"
                const worker = new Worker({path:?});
                const greeting = await new Promise((resolve) => {{
                    worker.onmessage = (e) => resolve(e.data);
                }});
                if (greeting !== "hello") throw new Error(`expected "hello", got ${{greeting}}`);
                worker.terminate();
                "#,
                path = script.display()
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn onerror_fires_when_the_worker_script_throws() {
        let dir = scratch_dir("onerror");
        let script = write_script(
            &dir,
            "throws.mjs",
            r#"
                throw new Error("boom");
            "#,
        );
        let vm = build_vm(&dir).await;

        run(
            &vm,
            format!(
                r#"
                const worker = new Worker({path:?});
                const reason = await new Promise((resolve) => {{
                    worker.onerror = (e) => resolve(e.data);
                }});
                if (!String(reason).includes("boom")) {{
                    throw new Error(`expected error mentioning "boom", got ${{reason}}`);
                }}
                worker.terminate();
                "#,
                path = script.display()
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn add_event_listener_and_remove_event_listener_both_work() {
        let dir = scratch_dir("listeners");
        let script = write_script(
            &dir,
            "echo.mjs",
            r#"
                addEventListener("message", (event) => {
                    postMessage(event.data);
                });
            "#,
        );
        let vm = build_vm(&dir).await;

        run(
            &vm,
            format!(
                r#"
                const worker = new Worker({path:?});
                const seen = [];

                const removed = (e) => seen.push(`removed:${{e.data}}`);
                worker.addEventListener("message", removed);
                worker.removeEventListener("message", removed);

                worker.addEventListener("message", (e) => seen.push(`kept:${{e.data}}`));

                const first = await new Promise((resolve) => {{
                    worker.addEventListener("message", function once(e) {{
                        worker.removeEventListener("message", once);
                        resolve(e.data);
                    }});
                    worker.postMessage("ping");
                }});

                if (first !== "ping") throw new Error(`expected "ping", got ${{first}}`);
                if (seen.join(",") !== "kept:ping") throw new Error(`listeners fired: ${{seen}}`);
                worker.terminate();
                "#,
                path = script.display()
            ),
        )
        .await;
    }
}
