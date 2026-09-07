# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Klaver is a work-in-progress, WinterTC-compatible JavaScript runtime written in Rust on top of
[rquickjs](https://github.com/DelSkayn/rquickjs) (QuickJS bindings), designed primarily for embedding into other
languages/applications rather than as a standalone CLI (the CLI is mostly a dev/test harness).

## Build / test / run

Standard cargo workspace, resolver "2". `OLD` and `OLD2` at the repo root are legacy/reference code, excluded from
the workspace (`OLD`) or not workspace members at all (`OLD2`) — do not build against them and generally ignore them
unless explicitly asked to look there.

```sh
cargo build --workspace              # build everything
cargo build -p klaver-cli            # build just the CLI binary ("klaver")
cargo test --workspace               # run all tests
cargo test -p klaver-core            # run tests for one crate
cargo test -p klaver-core value::date # run a single test by path filter
```

Tests are plain `#[cfg(test)] mod tests` inline in source files (no separate `tests/` integration dirs in most
crates). There's no workspace-wide lint/format command configured beyond standard `cargo fmt` / `cargo clippy`.

Cross-compilation (musl/aarch64 release builds) is driven by `Justfile` via `cross`:

```sh
just cross-build     # x86_64-unknown-linux-musl release
just build-aarch64   # aarch64-unknown-linux-gnu release
```

`.cargo/config.toml` pins the native linker to clang+mold for local builds.

### Running JS/TS through the CLI

`klaver-cli` builds a binary named `klaver`. From the repo root:

```sh
cargo run -p klaver-cli -- <path-to-file.ts>       # execute a module (search path defaults to ".")
cargo run -p klaver-cli -- --exec 'console.log(1)' # eval a raw expression/promise
cargo run -p klaver-cli --                          # no path -> drop into a REPL (reedline)
cargo run -p klaver-cli -- <file> --types           # write out generated .d.ts typings instead of running
cargo run -p klaver-cli -- <file> --compile         # just run the SWC transform and print the compiled JS
```

The `examples/` directory contains sample `.ts`/`.js` files and a `tsconfig.json`/`package.json` used for manually
exercising the runtime (e.g. `deno.ts`, `test.ts`, `worker.ts`).

## Workspace layout

Members are declared in the root `Cargo.toml`. Several crate directories exist on disk but are **currently commented
out** of `[workspace] members` (`klaver-os`, `klaver-test`, `klaver-dom`) — check that file before assuming a crate
is part of the active build. `klaver-dom` also depends on a `klaver-util` crate that does not exist in this tree, so
it will not currently compile even if re-enabled.

Dependency layering, low-level to high-level:

- **klaver-core** — foundational primitives shared by everything else: the `Core`/`$runtime` global bootstrap,
  error handling (`error/`, including JS stack-trace capture), the `sync/` module (cell/lock/channel/notify
  wrappers built for the single-threaded-but-async QuickJS model), and `value/` (rquickjs value extensions —
  typed arrays, Map/Set/WeakMap/RegExp/Date wrappers, structured clone support, JSON, string refs, etc). Exposes the
  `Exportable`/`ExportTarget` traits used by every module below to attach Rust types to a JS context, and the
  `module_info!`/`global_info!`-style registration story lives one layer up in klaver-modules.
- **klaver-modules** — the module/global registration and resolution system. Defines `ModuleInfo` /
  `GlobalInfo` (via the `module_info!` / `global_info!` macros) which crates implement to declare themselves as an
  importable JS module or a global installed into every context; `Environ`/`Builder`/`EnvBuilder` accumulate
  registered modules/globals/loaders/resolvers into a reusable environment that can spin up rquickjs runtimes.
  `loaders/` and `resolvers/` implement how module specifiers are turned into source code (file loader, builtin
  in-memory sources, and two alternative TS/JSX transform backends implementing the same `Transformer` trait:
  SWC behind the `swc` feature (`loaders/swc/`) and Oxc behind the `oxc` feature (`loaders/oxc/`, using
  `oxc_parser`/`oxc_semantic`/`oxc_transformer`/`oxc_codegen`; note Oxc only supports TypeScript's legacy
  `experimentalDecorators`, not SWC's TC39 stage-3 decorators transform). Also `oxc_resolver`-based Node-style
  module resolution behind `file-resolver`, unrelated to the Oxc transform backend.
- **klaver-runtime** — the async event loop / task scheduling layer on top of an `Environ`+rquickjs runtime:
  `event_loop`, `executor`/`TaskHandle`, `task`/`task_manager`, async-hook style resource tracking
  (`async_hook`, `async_resource`, `promise_hook`) modeled loosely on Node's `async_hooks`, plus `AsyncLocalStorage`
  support.
- **klaver-vm** — ties `klaver-modules` + `klaver-runtime` + `klaver-core` together into a concrete `Vm` /
  `Context` / `Builder` (`Options`) API: creates the rquickjs `Runtime`/`AsyncContext`, applies stack/memory limits,
  and (behind features) offers a worker (`flume`) and a pool (`deadpool`) of VMs.
- **klaver-wintertc** — the actual WinterTC/web-platform surface implemented as globals/modules, each behind its
  own Cargo feature and gated as a `GlobalInfo`/`ModuleInfo` dependency of the top-level `WinterTC` global
  (`src/module.rs`): `console`, `fetch/` (Request/Response/Headers/URL/fetch), `timers/`, `crypto/`, `streams/`,
  `intl/` (ICU-backed `Intl`), `events/` (EventTarget/emitter), `channel/` (MessageChannel/port), `worker/`
  (Worker), `fs/` (sandboxed filesystem, requires a `Backend`), `blob.rs`, `encoding/`, `abort_controller.rs`,
  `dom_exception.rs`. A `Backend` trait (implemented by e.g. `TokioBackend`) supplies the async runtime primitives
  (fs, etc) these need — set per-VM via `klaver_wintertc::set_backend`.
- **klaver** (top-level crate) — the batteries-included `Builder`/`Vm` most consumers use: wires a `Backend`,
  search paths, the file resolver/loader (with the Oxc TS/JSX transform behind the `oxc` feature and/or the SWC
  one behind `swc` — both forward to `klaver-modules`; if both are enabled, Oxc is registered first and wins for
  overlapping extensions), and the `WinterTC` global into one `klaver_vm::Vm`.
- **klaver-cli** — the `klaver` binary: CLI arg parsing (`cli.rs`), building a `klaver::Vm` with the Tokio backend
  and extra modules (`klaver-image`, `klaver-runtime`'s `TaskModule`), and running/REPL logic (`run.rs`).

Optional feature/domain modules that plug into the module system the same way (`ModuleInfo` impl via
`module_info!`, exported as an importable JS module):

- **klaver-image** — image decode/encode module (`image` crate, optional `webp`).
- **klaver-hbs** — Handlebars templating module.
- **klaver-os** — `sysinfo`-backed OS info module (currently disabled workspace member, see above).
- **klaver-dom** — DOM module built on external `domjohnson`/`locket` crates (currently disabled workspace
  member; missing `klaver-util` dependency).

## test262 conformance suite

`klaver-test262` runs the [test262](https://github.com/tc39/test262) ECMAScript conformance suite
against a bare `klaver_vm::Vm` (no WinterTC globals). The suite itself is vendored as a git
submodule at `test262/`, not checked into the repo — run `git submodule update --init test262`
first. See `klaver-test262/README.md` for details; the short version:

```sh
cargo run -p klaver-test262                         # full suite, checked against the baseline
cargo run -p klaver-test262 -- built-ins/Promise     # a subset (paths relative to test262/test)
cargo run -p klaver-test262 -- --update-expectations # regenerate klaver-test262/expectations.txt
```

Results are compared against `klaver-test262/expectations.txt` (every currently-known
failing/skipped test); the run only fails CI-style on *regressions* — tests not in that file that
now fail — since a partial engine won't pass 100% of the suite.

### Adding a new global or module

Look at `klaver-hbs` (module) or `klaver-wintertc/src/module.rs` (`WinterTC`, a global with sub-module
dependencies) as templates: implement `Exportable`/`Global`/`Module` for a type, then register it with
`klaver_modules::module_info!("name" => Type)` or `global_info!(...)`, and add it as a `.module::<T>()` /
`.global::<T>()` call on the `klaver::Builder` (see `klaver-cli/src/cli.rs`) or `klaver_vm::Options`.

## Solid SSR / scratch files

`solid-ssr/` is a separate pnpm/vite JS project (SolidJS SSR build) used for exercising Klaver against real-world
JS output; it is not part of the Rust workspace. Root-level `store.js`, `test.mjs`, `test.tsx`, `tmp.js`, `deno.ts`
are ad hoc scratch scripts used for manual testing against the CLI/runtime, not part of any build.
