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
- ✅ **`CustomEvent`** - `src/events/custom_event.rs`, an `Event` subclass with a `detail` field
  (defaulting to `null`) plus the legacy `initCustomEvent()` method.
- ✅ **`ErrorEvent`** - `src/events/error_event.rs`, an `Event` subclass with
  `message`/`filename`/`lineno`/`colno`/`error` fields (per-field spec defaults `""`/`""`/`0`/`0`/`null`).
- ✅ **`performance`** - `src/performance.rs`. QuickJS-ng's native `performance` global (`now()`/
  `timeOrigin`) seeds `timeOrigin` from `CLOCK_MONOTONIC`, not wall-clock time, so it isn't
  epoch-relative and `timeOrigin + now()` doesn't approximate `Date.now()` as HR-TIME requires.
  Replaced with a `Performance` class that fixes that (`timeOrigin` from `SystemTime`, `now()`
  from a monotonic `Instant` anchored at the same instant) and adds the `EventTarget` inheritance
  and `toJSON()` the spec's WebIDL also calls for.

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
| `performance` | ✅ | `src/performance.rs` - see [Quick wins](#quick-wins) above. |
| `WebAssembly.*` | ❌ | No WebAssembly support anywhere in the workspace (QuickJS has no built-in Wasm engine; this would mean embedding one, e.g. `wasmtime`/`wasmer`, and bridging its API - a substantial undertaking, not a small gap). |

## DOM & events

| API | Status | Notes |
|---|---|---|
| `EventTarget` | ✅ | `src/events/event_target.rs` - `addEventListener`/`removeEventListener`/`dispatchEvent`, `signal` option support. |
| `Event` | ✅ | `src/events/event.rs`. |
| `CustomEvent` | ✅ | `src/events/custom_event.rs`. |
| `ErrorEvent` | ✅ | `src/events/error_event.rs`. Not yet wired up to anything that would dispatch one automatically (see the missing `onerror`/`window.onerror`-style global error reporting above) - only the constructor/fields are implemented. |
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
| `crypto.subtle` (`SubtleCrypto`) | ✅ | `digest(algo, buffer)` (SHA-1/256/384/512, `src/crypto/digest.rs`) is always available under the base `crypto` feature. Behind `crypto-cipher` (nested under `crypto`): a real `CryptoKey` class (`src/crypto/key.rs`), `generateKey`/`importKey`/`exportKey` (`"raw"`/`"jwk"` formats), `encrypt`/`decrypt` (AES-GCM/CBC/CTR, `src/crypto/aes.rs`), and `sign`/`verify` (HMAC, `src/crypto/hmac.rs`) - symmetric keys. Behind `crypto-asymmetric` (nested under `crypto-cipher`): RSA (RSASSA-PKCS1-v1_5, RSA-PSS, RSA-OAEP, all four hashes including SHA-1 - `src/crypto/rsa.rs`, via a separately-pinned `rsa-sha1` dependency matching `rsa`'s internal digest line, since `rsa` never re-exports a `sha1` of its own), ECDSA/ECDH (P-256/P-384/P-521 - `src/crypto/ec.rs`), the `"pkcs8"`/`"spki"` key formats, `deriveBits`/`deriveKey` (ECDH, HKDF, PBKDF2 - `src/crypto/kdf.rs`), and `wrapKey`/`unwrapKey` (all four key formats, including `"jwk"` via a `JSON.stringify`/`json_parse` round trip around the wrapped bytes). |

## High Resolution Time / WebAssembly

High Resolution Time (`performance`) is implemented - see the Globals table above. WebAssembly
(`WebAssembly.*`) is entirely unimplemented - see the same table.

## Worker global scope

`src/worker/init.js` (run inside a spawned worker) wires up `onmessage`/`postMessage`/
`addEventListener`/`removeEventListener` against the worker's `MessagePort`, and now also sets
`self` (see [Quick wins](#quick-wins)). Per the spec's "if a runtime implements worker global
scopes, it must expose `onerror`, `onunhandledrejection`, `onrejectionhandled`, and `self`"
requirement, the last two are still missing there.

## `Worker` (not part of the WinterTC spec, audited here for completeness)

`src/worker/` implements a `Worker` constructor roughly matching the WHATWG HTML `Worker`
interface: `new Worker(scriptURL)`, `postMessage()`, `onmessage`/`addEventListener`/
`removeEventListener` (for `"message"`), `onerror` (fired - as a `MessageEvent` carrying a
description string as `data`, rather than a proper `ErrorEvent` - when the worker's module throws
during its top-level evaluation), and `terminate()`. Known gaps against the full spec:

- **`WorkerOptions`** (`type`, `credentials`, `name`) - not accepted; the constructor only takes
  a `scriptURL`. Every worker script is always loaded as an ES module (i.e. always behaves as
  `type: "module"`); there's no `"classic"` (`importScripts()`-based) mode.
- **`onmessageerror`** - not implemented; a structured-clone deserialization failure on either
  side currently propagates as a hard error on the underlying `MessagePort`'s background resource
  rather than a `"messageerror"` event.
- **`onerror` fires a `MessageEvent`, not an `ErrorEvent`** - `ErrorEvent` itself now exists
  (`src/events/error_event.rs`), but `worker.rs` hasn't been switched over to use it yet.
- **`Worker instanceof EventTarget`** - `Worker` is not actually a subclass of `EventTarget`
  (unlike `MessagePort`, which it wraps); `addEventListener`/`removeEventListener` are provided as
  ad hoc methods that proxy to the underlying port instead.
- Errors thrown *after* the worker's top-level evaluation (e.g. from within an `onmessage`
  handler) are not reported to `onerror` - only a failure during the initial
  `Module::import(...)` (syntax errors, an uncaught top-level throw) is currently caught.
