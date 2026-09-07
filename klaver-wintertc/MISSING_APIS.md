# Missing/partial WinterTC APIs in `klaver-wintertc`

This is a gap audit of `klaver-wintertc` against the [WinterTC Minimum Common Web API](https://min-common-api.proposal.wintertc.org/)
spec (the Ecma TC55 "common minimum API" server-runtime interoperability target). It only covers
APIs the spec actually requires — it is not a general wishlist of web APIs.

Status legend: ✅ implemented · 🟡 partial · ❌ missing.

## Quick wins

All done:

- ✅ **`crypto.getRandomValues` naming.** `src/crypto/module.rs` now exports the function as
  `crypto.getRandomValues` (matching `types/crypto.d.ts`) instead of the old `crypto.randomValues`.
- ✅ **`queueMicrotask`** - implemented in `src/base.rs` as a real QuickJS microtask (via
  `ctx.promise()` + `Promise#then`, not piggybacked on the `klaver-runtime` event loop/timers), so
  ordering relative to promise reactions matches spec.
- ✅ **`self`** - now aliased to the global object in both the main global scope (`src/base.rs`)
  and worker global scopes (`src/worker/init.js`).

## Globals

| API | Status | Notes |
|---|---|---|
| `globalThis` | ✅ | Provided natively by QuickJS; nothing runtime-specific needed. |
| `self` | ✅ | `src/base.rs` (main global scope) and `src/worker/init.js` (worker global scope) alias it to the global object. |
| `setTimeout`/`setInterval`/`clearTimeout`/`clearInterval` | ✅ | `src/timers/` (`timers.rs`, `module.rs`), backed by an `AsyncState`/`TaskHandle` resource. |
| `atob`/`btoa` | ✅ | `src/encoding/b64.rs`. |
| `queueMicrotask` | ✅ | `src/base.rs`. |
| `reportError` | ❌ | Not implemented. |
| `onerror` / `onunhandledrejection` / `onrejectionhandled` | ❌ | None of the three exist on the global scope (main or worker). |
| `navigator.userAgent` | ❌ | No `navigator` object anywhere. |
| `structuredClone` | ✅ | `src/base.rs`, backed by `klaver-core`; supports the `transfer` option. |
| `fetch` | ✅ | `src/fetch/fetch.rs`. |
| `console` | ✅ | `src/console.rs`. |
| `crypto` | 🟡 | See [Web Crypto](#web-crypto) below. |
| `performance` | ❌ | No `performance` global or `Performance` interface anywhere in the workspace. |
| `WebAssembly.*` | ❌ | No WebAssembly support anywhere in the workspace (QuickJS has no built-in Wasm engine; this would mean embedding one, e.g. `wasmtime`/`wasmer`, and bridging its API - a substantial undertaking, not a small gap). |

## DOM & events

| API | Status | Notes |
|---|---|---|
| `EventTarget` | ✅ | `src/events/event_target.rs` - `addEventListener`/`removeEventListener`/`dispatchEvent`, `signal` option support. |
| `Event` | ✅ | `src/events/event.rs`. |
| `CustomEvent` | ❌ | No dedicated type; plain `Event` has no generic `detail` field. JS-side subclassing of `Event` does work (see `event_target.rs`'s tests), so user code can approximate this, but there's no built-in `CustomEvent` constructor. |
| `ErrorEvent` | ❌ | Not implemented. |
| `MessageEvent` | ✅ | `src/channel/event.rs`, an `Event` subclass with a `data` field. |
| `PromiseRejectionEvent` | ❌ | Not implemented (ties into the missing `onunhandledrejection`/`onrejectionhandled` above). |
| `AbortController`/`AbortSignal` | ✅ | `src/abort_controller.rs`. |
| `MessageChannel`/`MessagePort` | ✅ | `src/channel/channel.rs`, `src/channel/port.rs`. |
| `DOMException` | ✅ | `src/dom_exception.rs`. |

## Fetch / File API

| API | Status | Notes |
|---|---|---|
| `Headers` | ✅ | `src/fetch/headers.rs`. |
| `Request` / `Response` | ✅ | `src/fetch/request.rs`, `src/fetch/response.rs`. |
| `FormData` | ✅ | `src/fetch/form_data.rs`. |
| `Blob` / `File` | ✅ | `src/blob.rs`. |
| `URL` / `URLSearchParams` | ✅ | `src/fetch/url.rs`, `src/fetch/url_search_params.rs`. |
| `URLPattern` | ❌ | Not implemented. |

## Streams

| API | Status | Notes |
|---|---|---|
| `ReadableStream` | ✅ | `src/streams/readable/stream.rs`. |
| `ReadableStreamDefaultReader` | ✅ | `src/streams/readable/reader.rs`. |
| `ReadableStreamBYOBReader` | ✅ | `src/streams/readable/byob_reader.rs` (see its doc comment for what's *not* zero-copy there). |
| `ReadableStreamDefaultController` | ✅ | `src/streams/readable/controller.rs`. |
| `ReadableByteStreamController` | 🟡 | Not a distinct type - byte-stream enqueue/BYOB support is folded into the same `ReadableStreamDefaultController` rather than a separate class. |
| `ReadableStreamBYOBRequest` | ❌ | Not implemented. |
| `WritableStream` | ✅ | `src/streams/writable/stream.rs`. |
| `WritableStreamDefaultWriter` | ✅ | `src/streams/writable/writer.rs`. |
| `WritableStreamDefaultController` | ✅ | `src/streams/writable/controller.rs`. |
| `TransformStream` | ✅ | `src/streams/transform/stream.rs`. |
| `TransformStreamDefaultController` | ✅ | `src/streams/transform/controller.rs`. |
| `ByteLengthQueuingStrategy` / `CountQueuingStrategy` | ✅ | `src/streams/queue_strategy.rs`. |

## Encoding / compression

| API | Status | Notes |
|---|---|---|
| `TextEncoder` / `TextDecoder` | ✅ | `src/encoding/encoding.rs`. |
| `TextEncoderStream` / `TextDecoderStream` | ✅ | `src/encoding/streams.rs`, gated behind the `streams` feature (built on the same `TransformStream` machinery, not exposed as JS-visible transformers). |
| `CompressionStream` / `DecompressionStream` | ❌ | Not implemented. |
| `Uint8Array.prototype.{to,from,setFrom}Base64`/`{to,from,setFrom}Hex` | ❌ | Not implemented (newer TC39 proposal, now widely shipped in browsers/Node; not present in `klaver-core`'s typed-array code either). |

## Web Crypto

| API | Status | Notes |
|---|---|---|
| `crypto.randomUUID` | ✅ | `src/crypto/random.rs`. |
| `crypto.getRandomValues` | ✅ | `src/crypto/random.rs`, registered as `crypto.getRandomValues` in `src/crypto/module.rs`. |
| `crypto.subtle` (`SubtleCrypto`) | 🟡 | Plain object, not a real `SubtleCrypto` class; exposes only `digest(algo, buffer)` (SHA-1/256/384/512 via `src/crypto/digest.rs`). Missing `encrypt`/`decrypt`/`sign`/`verify`/`generateKey`/`importKey`/`exportKey`/`deriveKey`/`deriveBits`/`wrapKey`/`unwrapKey`, and there's no `CryptoKey` type at all. |

## High Resolution Time / WebAssembly

Both entirely unimplemented - see the Globals table above (`performance`, `WebAssembly.*`).

## Worker global scope

`src/worker/init.js` (run inside a spawned worker) wires up `onmessage`/`postMessage`/
`addEventListener`/`removeEventListener` against the worker's `MessagePort`, and now also sets
`self` (see [Quick wins](#quick-wins)). Per the spec's "if a runtime implements worker global
scopes, it must expose `onerror`, `onunhandledrejection`, `onrejectionhandled`, and `self`"
requirement, the first three are still missing there.
