mod data;
mod queue;
mod queue_strategy;
pub mod readable;
pub mod transform;
pub mod writable;

use rquickjs::{Ctx, IntoJs, Value, class::JsClass};

use klaver_core::Registry;

/// `Option<f64>::IntoJs` maps `None` to `undefined`, but spec-defined `desiredSize` getters
/// (`ReadableStreamDefaultController`, `WritableStreamDefaultWriter`,
/// `TransformStreamDefaultController`) are specifically documented to return `null` once
/// errored/closed, not `undefined`.
pub(crate) fn desired_size_value<'js>(
    ctx: &Ctx<'js>,
    size: Option<f64>,
) -> rquickjs::Result<Value<'js>> {
    match size {
        Some(size) => size.into_js(ctx),
        None => Ok(Value::new_null(ctx.clone())),
    }
}

pub use self::{
    queue_strategy::{ByteLengthQueuingStrategy, CountQueuingStrategy, QueuingStrategy},
    readable::{
        ReadableStream, ReadableStreamBYOBReader, ReadableStreamDefaultController,
        ReadableStreamDefaultReader,
    },
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

    // ---- WritableStream writer completeness ----

    #[tokio::test]
    async fn writer_desired_size_reflects_high_water_mark_and_queue_size() {
        run(r#"
            const stream = new WritableStream({}, new CountQueuingStrategy({ highWaterMark: 3 }));
            const writer = stream.getWriter();
            if (writer.desiredSize !== 3) throw new Error(`desiredSize was ${writer.desiredSize}`);

            writer.write(1);
            writer.write(2);
            // desiredSize reflects the queue before the sink has drained it.
            if (writer.desiredSize > 1) throw new Error(`desiredSize was ${writer.desiredSize}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn writer_desired_size_is_null_once_errored() {
        run(r#"
            const stream = new WritableStream({
                write() { throw new Error("boom"); },
            });
            const writer = stream.getWriter();

            let threw = false;
            try {
                await writer.write("x");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected write() to reject");
            if (writer.desiredSize !== null) throw new Error(`desiredSize was ${writer.desiredSize}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn writer_closed_resolves_once_the_stream_finishes_closing() {
        run(r#"
            const stream = new WritableStream({});
            const writer = stream.getWriter();
            await writer.close();
            await writer.closed;
        "#)
        .await;
    }

    #[tokio::test]
    async fn writer_write_rejects_once_the_stream_is_closed() {
        run(r#"
            const stream = new WritableStream({});
            const writer = stream.getWriter();
            await writer.close();

            let threw = false;
            try {
                await writer.write("too late");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected write() after close() to reject");
        "#)
        .await;
    }

    #[tokio::test]
    async fn writer_write_rejects_with_the_abort_reason_once_aborted() {
        run(r#"
            const stream = new WritableStream({});
            const writer = stream.getWriter();
            await writer.abort("stop");

            let seen;
            try {
                await writer.write("too late");
            } catch (e) {
                seen = e;
            }
            if (seen !== "stop") throw new Error(`rejection reason was ${seen}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn writable_controller_signal_aborts_with_the_stream() {
        run(r#"
            let seenReason;
            const stream = new WritableStream({
                start(controller) {
                    controller.signal.addEventListener("abort", () => {
                        seenReason = controller.signal.reason;
                    });
                },
            });

            await stream.abort("stop");
            if (seenReason !== "stop") throw new Error(`signal reason was ${seenReason}`);
        "#)
        .await;
    }

    // ---- pipeTo options ----

    #[tokio::test]
    async fn pipe_to_prevent_close_leaves_destination_open() {
        run(r#"
            const source = new ReadableStream({
                start(controller) { controller.enqueue("x"); controller.close(); },
            });

            let closed = false;
            const dest = new WritableStream({
                close() { closed = true; },
            });

            await source.pipeTo(dest, { preventClose: true });
            if (closed) throw new Error("destination should not have been closed");

            // The destination writer is free (pipeTo released its lock on completion).
            const writer = dest.getWriter();
            await writer.close();
        "#)
        .await;
    }

    #[tokio::test]
    async fn pipe_to_aborts_destination_when_source_errors() {
        run(r#"
            const source = new ReadableStream({
                start(controller) { controller.error(new Error("source broke")); },
            });

            let abortReason;
            const dest = new WritableStream({
                abort(reason) { abortReason = reason; },
            });

            let threw = false;
            try {
                await source.pipeTo(dest);
            } catch (e) {
                threw = true;
                if (e.message !== "source broke") throw new Error(`wrong error: ${e.message}`);
            }
            if (!threw) throw new Error("expected pipeTo() to reject");
            if (!abortReason || abortReason.message !== "source broke") {
                throw new Error(`destination was not aborted with the source's error`);
            }
        "#)
        .await;
    }

    #[tokio::test]
    async fn pipe_to_cancels_source_when_destination_errors() {
        run(r#"
            let cancelReason;
            const source = new ReadableStream({
                start(controller) { controller.enqueue("x"); },
                cancel(reason) { cancelReason = reason; },
            });

            const dest = new WritableStream({
                write() { throw new Error("dest broke"); },
            });

            let threw = false;
            try {
                await source.pipeTo(dest);
            } catch (e) {
                threw = true;
                if (e.message !== "dest broke") throw new Error(`wrong error: ${e.message}`);
            }
            if (!threw) throw new Error("expected pipeTo() to reject");
            if (!cancelReason || cancelReason.message !== "dest broke") {
                throw new Error("source was not cancelled with the destination's error");
            }
        "#)
        .await;
    }

    #[tokio::test]
    async fn pipe_to_respects_an_already_aborted_signal() {
        run(r#"
            const controller = new AbortController();
            controller.abort("nope");

            const source = new ReadableStream({ start(c) { c.enqueue("x"); } });
            const dest = new WritableStream({});

            let seen;
            try {
                await source.pipeTo(dest, { signal: controller.signal });
            } catch (e) {
                seen = e;
            }
            if (seen !== "nope") throw new Error(`rejection reason was ${seen}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn pipe_through_returns_the_transforms_readable_side() {
        run(r#"
            const source = new ReadableStream({
                start(controller) {
                    controller.enqueue("a");
                    controller.enqueue("b");
                    controller.close();
                },
            });

            const ts = new TransformStream({
                transform(chunk, controller) { controller.enqueue(chunk.toUpperCase()); },
            });

            const out = source.pipeThrough(ts);
            const reader = out.getReader();
            const seen = [];
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                seen.push(value);
            }
            if (seen.join(",") !== "A,B") throw new Error(`values were ${seen}`);
        "#)
        .await;
    }

    // ---- tee() ----

    #[tokio::test]
    async fn tee_delivers_every_chunk_to_both_branches() {
        run(r#"
            const source = new ReadableStream({
                start(controller) {
                    controller.enqueue(1);
                    controller.enqueue(2);
                    controller.enqueue(3);
                    controller.close();
                },
            });

            const [a, b] = source.tee();

            async function drain(stream) {
                const reader = stream.getReader();
                const seen = [];
                while (true) {
                    const { value, done } = await reader.read();
                    if (done) break;
                    seen.push(value);
                }
                return seen.join(",");
            }

            const [aValues, bValues] = await Promise.all([drain(a), drain(b)]);
            if (aValues !== "1,2,3") throw new Error(`branch a saw ${aValues}`);
            if (bValues !== "1,2,3") throw new Error(`branch b saw ${bValues}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn tee_branches_are_independently_readable_streams() {
        run(r#"
            const source = new ReadableStream({
                start(controller) { controller.enqueue("x"); controller.close(); },
            });
            const [a, b] = source.tee();
            if (a === b) throw new Error("branches should be distinct streams");
            if (!(a instanceof ReadableStream) || !(b instanceof ReadableStream)) {
                throw new Error("branches should be ReadableStream instances");
            }
        "#)
        .await;
    }

    // ---- Byte streams / BYOB reader ----

    #[tokio::test]
    async fn byte_stream_enqueues_and_reads_typed_arrays_via_default_reader() {
        run(r#"
            const stream = new ReadableStream({
                type: "bytes",
                start(controller) {
                    controller.enqueue(new Uint8Array([1, 2, 3]));
                    controller.close();
                },
            });

            const reader = stream.getReader();
            const { value, done } = await reader.read();
            if (done) throw new Error("expected a chunk");
            if (value.length !== 3 || value[0] !== 1 || value[2] !== 3) {
                throw new Error(`chunk was ${value}`);
            }
        "#)
        .await;
    }

    #[tokio::test]
    async fn byte_stream_enqueue_rejects_non_array_buffer_view() {
        run(r#"
            const stream = new ReadableStream({
                type: "bytes",
                start(controller) {
                    let threw = false;
                    try {
                        controller.enqueue("not a view");
                    } catch {
                        threw = true;
                    }
                    if (!threw) throw new Error("expected enqueue() to throw for a non-view chunk");
                    controller.close();
                },
            });
            await stream.getReader().read();
        "#)
        .await;
    }

    #[tokio::test]
    async fn get_reader_byob_mode_requires_a_byte_stream() {
        run(r#"
            const stream = new ReadableStream({});
            let threw = false;
            try {
                stream.getReader({ mode: "byob" });
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected getReader({mode:'byob'}) to throw for a non-byte stream");
        "#)
        .await;
    }

    #[tokio::test]
    async fn byob_reader_reads_enqueued_bytes() {
        run(r#"
            const stream = new ReadableStream({
                type: "bytes",
                start(controller) {
                    controller.enqueue(new Uint8Array([10, 20, 30]));
                    controller.close();
                },
            });

            const reader = stream.getReader({ mode: "byob" });
            const buffer = new Uint8Array(3);
            const { value, done } = await reader.read(buffer);
            if (done) throw new Error("expected a chunk");
            if (value.length !== 3 || value[0] !== 10 || value[1] !== 20 || value[2] !== 30) {
                throw new Error(`value was ${value}`);
            }

            const next = await reader.read(buffer);
            if (!next.done) throw new Error("expected the stream to be done");
        "#)
        .await;
    }

    #[tokio::test]
    async fn byob_reader_controller_byob_request_is_null() {
        // Documented simplification: this implementation doesn't support the zero-copy
        // `byobRequest` path.
        run(r#"
            let seenRequest = "unset";
            const stream = new ReadableStream({
                type: "bytes",
                pull(controller) {
                    seenRequest = controller.byobRequest;
                    controller.enqueue(new Uint8Array([1]));
                    controller.close();
                },
            });

            const reader = stream.getReader({ mode: "byob" });
            await reader.read(new Uint8Array(1));
            if (seenRequest !== null) throw new Error(`byobRequest was ${seenRequest}`);
        "#)
        .await;
    }

    // ---- TextEncoderStream / TextDecoderStream ----
    //
    // These live here rather than alongside `TextEncoderStream`/`TextDecoderStream` in
    // `encoding/streams.rs` because this module's test name sorts after `fetch`'s (whose own
    // tests build a bare `rquickjs::AsyncRuntime` via `futures::executor::block_on`, not a real
    // `klaver_vm::Vm`) - running enough real, event-loop-driven `Vm`s *before* that fetch test in
    // the same test binary made it hang. Consolidating here matches every other stream test.

    #[tokio::test]
    async fn text_encoder_stream_encodes_written_strings() {
        run(r#"
            const stream = new TextEncoderStream();
            if (stream.encoding !== "utf-8") throw new Error(`encoding was ${stream.encoding}`);

            const writer = stream.writable.getWriter();
            const reader = stream.readable.getReader();

            writer.write("hello ");
            writer.write("world");
            writer.close();

            const chunks = [];
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                if (!(value instanceof Uint8Array)) throw new Error("expected a Uint8Array chunk");
                chunks.push(...value);
            }

            const text = new TextDecoder().decode(new Uint8Array(chunks));
            if (text !== "hello world") throw new Error(`decoded text was ${text}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn text_encoder_stream_encodes_multibyte_utf8() {
        run(r#"
            const stream = new TextEncoderStream();
            const writer = stream.writable.getWriter();
            const reader = stream.readable.getReader();

            writer.write("é");
            writer.close();

            const { value, done } = await reader.read();
            if (done) throw new Error("expected a chunk");
            // U+00E9 is encoded as the two UTF-8 bytes 0xC3 0xA9.
            if (value.length !== 2 || value[0] !== 0xc3 || value[1] !== 0xa9) {
                throw new Error(`bytes were ${value}`);
            }
        "#)
        .await;
    }

    #[tokio::test]
    async fn text_decoder_stream_decodes_written_bytes() {
        run(r#"
            const stream = new TextDecoderStream();
            if (stream.encoding !== "utf-8") throw new Error(`encoding was ${stream.encoding}`);

            const writer = stream.writable.getWriter();
            const reader = stream.readable.getReader();

            const bytes = new TextEncoder().encode("hello world");
            writer.write(bytes);
            writer.close();

            let text = "";
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                text += value;
            }
            if (text !== "hello world") throw new Error(`text was ${text}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn text_decoder_stream_reassembles_a_multibyte_sequence_split_across_writes() {
        // The whole point of TextDecoderStream over one-shot TextDecoder: a multi-byte UTF-8
        // sequence split across two separate `write()` calls must still decode correctly.
        run(r#"
            const stream = new TextDecoderStream();
            const writer = stream.writable.getWriter();
            const reader = stream.readable.getReader();

            const bytes = new TextEncoder().encode("é"); // [0xc3, 0xa9]
            writer.write(bytes.slice(0, 1));
            writer.write(bytes.slice(1));
            writer.close();

            let text = "";
            while (true) {
                const { value, done } = await reader.read();
                if (done) break;
                text += value;
            }
            if (text !== "é") throw new Error(`text was ${JSON.stringify(text)}`);
        "#)
        .await;
    }

    #[tokio::test]
    async fn text_decoder_stream_accepts_a_label() {
        run(r#"
            if (new TextDecoderStream("UTF8").encoding !== "utf-8") {
                throw new Error(`encoding was ${new TextDecoderStream("UTF8").encoding}`);
            }
        "#)
        .await;
    }

    #[tokio::test]
    async fn text_decoder_stream_rejects_unknown_label() {
        run(r#"
            let threw = false;
            try {
                new TextDecoderStream("not-a-real-encoding");
            } catch {
                threw = true;
            }
            if (!threw) throw new Error("expected constructor to throw");
        "#)
        .await;
    }
}
