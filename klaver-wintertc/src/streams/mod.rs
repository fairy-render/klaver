mod data;
mod queue;
mod queue_strategy;
pub mod readable;
pub mod transform;
pub mod writable;

use rquickjs::class::JsClass;

use klaver_core::Registry;

pub use self::{
    queue_strategy::{ByteLengthQueuingStrategy, CountQueuingStrategy, QueuingStrategy},
    readable::{ReadableStream, ReadableStreamDefaultController, ReadableStreamDefaultReader},
    transform::{TransformStream, TransformStreamDefaultController},
    writable::{WritableStream, WritableStreamDefaultController, WritableStreamDefaultWriter},
};

pub fn declare<'js>(decl: &rquickjs::module::Declarations<'js>) -> rquickjs::Result<()> {
    writable::declare(decl)?;
    readable::declare(decl)?;
    transform::declare(decl)?;

    decl.declare(queue_strategy::ByteLengthQueuingStrategy::NAME)?;
    decl.declare(queue_strategy::CountQueuingStrategy::NAME)?;

    Ok(())
}

pub fn export<'js, T>(
    ctx: &rquickjs::Ctx<'js>,
    registry: &Registry,
    exports: &T,
) -> rquickjs::Result<()>
where
    T: klaver_core::ExportTarget<'js>,
{
    writable::export(ctx, registry, exports)?;
    readable::export(ctx, registry, exports)?;
    transform::export(ctx, registry, exports)?;

    export!(
        ctx,
        registry,
        exports,
        ByteLengthQueuingStrategy,
        CountQueuingStrategy
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use klaver_vm::{Options, Vm};
    use rquickjs::CatchResultExt;

    async fn build_vm() -> Vm {
        Options::default()
            .global::<crate::WinterTC>()
            .build()
            .await
            .unwrap()
    }

    /// Runs `body` as the contents of an `async` IIFE and returns its resolved value. `body` is
    /// expected to throw on failure (e.g. via a plain `if (...) throw ...`).
    async fn run(body: &str) {
        let vm = build_vm().await;

        vm.async_with(async |ctx| {
            let promise: rquickjs::Promise = ctx
                .eval(format!("(async () => {{\n{body}\n}})()"))
                .catch(&ctx)?;

            promise.into_future::<()>().await.catch(&ctx)?;

            Ok(())
        })
        .await
        .unwrap();
    }

    // ---- ReadableStream ----

    #[tokio::test]
    async fn readable_enqueue_and_read_in_order() {
        // Regression test: the internal queue used to pop from the wrong end, delivering
        // enqueued chunks in reverse order.
        run(r#"
            const stream = new ReadableStream({
                start(controller) {
                    controller.enqueue(1);
                    controller.enqueue(2);
                    controller.enqueue(3);
                    controller.close();
                },
            });

            const reader = stream.getReader();
            const seen = [];
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                seen.push(value);
            }
            if (seen.join(",") !== "1,2,3") throw new Error(`read order was ${seen}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn readable_pull_is_called_to_refill_the_queue() {
        run(r#"
            let n = 0;
            const stream = new ReadableStream({
                pull(controller) {
                    n += 1;
                    if (n > 3) {
                        controller.close();
                    } else {
                        controller.enqueue(n);
                    }
                },
            }, new CountQueuingStrategy({ highWaterMark: 1 }));

            const reader = stream.getReader();
            const seen = [];
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                seen.push(value);
            }
            if (seen.join(",") !== "1,2,3") throw new Error(`pulled values were ${seen}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn readable_error_rejects_pending_and_future_reads() {
        run(r#"
            const stream = new ReadableStream({
                start(controller) {
                    controller.error(new Error("boom"));
                },
            });

            const reader = stream.getReader();
            let threw = false;
            try {
                await reader.read();
            } catch (e) {
                threw = true;
                if (e.message !== "boom") throw new Error(`wrong error: ${e.message}`);
            }
            if (!threw) throw new Error("expected read() to reject");
        "#)
        .await;
    }

    #[tokio::test]
    async fn readable_cancel_calls_source_cancel() {
        run(r#"
            let cancelReason;
            const stream = new ReadableStream({
                start(controller) {
                    controller.enqueue(1);
                },
                cancel(reason) {
                    cancelReason = reason;
                },
            });

            await stream.cancel("nope");
            if (cancelReason !== "nope") throw new Error(`cancel reason was ${cancelReason}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn readable_get_reader_locks_the_stream() {
        run(r#"
            const stream = new ReadableStream({ start(controller) { controller.close(); } });
            if (stream.locked) throw new Error("should not be locked yet");

            const reader = stream.getReader();
            if (!stream.locked) throw new Error("should be locked");

            let threw = false;
            try {
                stream.getReader();
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected a second getReader() to throw");

            reader.releaseLock();
            if (stream.locked) throw new Error("should be unlocked after releaseLock()");
        "#)
        .await;
    }

    #[tokio::test]
    async fn readable_async_iteration() {
        run(r#"
            const stream = new ReadableStream({
                start(controller) {
                    controller.enqueue("a");
                    controller.enqueue("b");
                    controller.close();
                },
            });

            const seen = [];
            for await (const chunk of stream) {
                seen.push(chunk);
            }
            if (seen.join(",") !== "a,b") throw new Error(`iterated values were ${seen}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn readable_pipe_to_writable() {
        run(r#"
            const source = new ReadableStream({
                start(controller) {
                    controller.enqueue("x");
                    controller.enqueue("y");
                    controller.close();
                },
            });

            const written = [];
            const dest = new WritableStream({
                write(chunk) { written.push(chunk); },
            });

            await source.pipeTo(dest);
            if (written.join(",") !== "x,y") throw new Error(`written values were ${written}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn readable_from_array() {
        run(r#"
            const stream = ReadableStream.from([1, 2, 3]);
            const reader = stream.getReader();
            const seen = [];
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                seen.push(value);
            }
            if (seen.join(",") !== "1,2,3") throw new Error(`values were ${seen}`);
        "#)
        .await;
    }

    // ---- WritableStream ----

    #[tokio::test]
    async fn writable_writes_are_delivered_in_order() {
        // Regression test: the internal queue used to pop from the wrong end, delivering
        // writes in reverse order.
        run(r#"
            const written = [];
            const stream = new WritableStream({
                write(chunk) { written.push(chunk); },
            });

            const writer = stream.getWriter();
            await writer.write(1);
            await writer.write(2);
            await writer.write(3);
            await writer.close();

            if (written.join(",") !== "1,2,3") throw new Error(`write order was ${written}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn writable_get_writer_locks_the_stream() {
        run(r#"
            const stream = new WritableStream({});
            if (stream.locked) throw new Error("should not be locked yet");

            const writer = stream.getWriter();
            if (!stream.locked) throw new Error("should be locked");

            let threw = false;
            try {
                stream.getWriter();
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected a second getWriter() to throw");
        "#)
        .await;
    }

    #[tokio::test]
    async fn writable_abort_calls_sink_abort() {
        run(r#"
            let abortReason;
            const stream = new WritableStream({
                abort(reason) { abortReason = reason; },
            });

            await stream.abort("stop");
            if (abortReason !== "stop") throw new Error(`abort reason was ${abortReason}`);
        "#)
        .await;
    }

    // ---- TransformStream ----

    #[tokio::test]
    async fn transform_identity_default() {
        run(r#"
            const ts = new TransformStream();
            const writer = ts.writable.getWriter();
            const reader = ts.readable.getReader();

            await writer.write("hello");
            await writer.close();

            const { value, done } = await reader.read();
            if (done) throw new Error("expected a value");
            if (value !== "hello") throw new Error(`value was ${value}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn transform_applies_custom_transform_in_order() {
        run(r#"
            const ts = new TransformStream({
                transform(chunk, controller) {
                    controller.enqueue(chunk.toUpperCase());
                },
            });

            const writer = ts.writable.getWriter();
            const reader = ts.readable.getReader();

            writer.write("a");
            writer.write("b");
            writer.write("c");
            writer.close();

            const seen = [];
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                seen.push(value);
            }
            if (seen.join(",") !== "A,B,C") throw new Error(`values were ${seen}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn transform_flush_can_enqueue_a_final_chunk() {
        run(r#"
            const ts = new TransformStream({
                flush(controller) {
                    controller.enqueue("last");
                },
            });

            const writer = ts.writable.getWriter();
            const reader = ts.readable.getReader();

            await writer.write("first");
            await writer.close();

            const seen = [];
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                seen.push(value);
            }
            if (seen.join(",") !== "first,last") throw new Error(`values were ${seen}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn transform_error_in_transform_fails_the_readable_side_too() {
        run(r#"
            const ts = new TransformStream({
                transform() {
                    throw new Error("bad chunk");
                },
            });

            const reader = ts.readable.getReader();
            const writer = ts.writable.getWriter();

            let threw = false;
            try {
                await writer.write("x");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected write() to reject");

            let readThrew = false;
            try {
                await reader.read();
            } catch (e) {
                readThrew = true;
                if (e.message !== "bad chunk") throw new Error(`wrong error: ${e.message}`);
            }
            if (!readThrew) throw new Error("expected the readable side to be errored too");
        "#)
        .await;
    }

    // ---- Queuing strategies ----

    #[tokio::test]
    async fn count_queuing_strategy_defaults_and_size() {
        run(r#"
            const strategy = new CountQueuingStrategy({});
            if (strategy.highWaterMark !== 1) throw new Error(`highWaterMark was ${strategy.highWaterMark}`);
            if (strategy.size("anything") !== 1) throw new Error("size() should always be 1");
        "#)
        .await;
    }

    #[tokio::test]
    async fn byte_length_queuing_strategy_reflects_high_water_mark() {
        // Regression test: the getter used to be named "highWaterHighway".
        run(r#"
            const strategy = new ByteLengthQueuingStrategy({ highWaterMark: 16 });
            if (strategy.highWaterMark !== 16) throw new Error(`highWaterMark was ${strategy.highWaterMark}`);
        "#)
        .await;
    }
}
