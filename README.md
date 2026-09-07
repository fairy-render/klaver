# Klaver

A work-in-progress, WinterTC-compatible JavaScript runtime written in Rust on top of
[rquickjs](https://github.com/DelSkayn/rquickjs) (QuickJS bindings), designed primarily for
embedding into other languages/applications rather than as a standalone CLI.

## Status

Klaver implements a growing subset of the [WinterTC](https://wintertc.org/) web-platform APIs —
`console`, `fetch`/`Request`/`Response`/`Headers`/`URL`/`URLSearchParams`, timers, Web Crypto,
`Intl`, streams (`ReadableStream`/`WritableStream`/`TransformStream`), `EventTarget`/`Event`,
`MessageChannel`, `Worker`, a sandboxed filesystem, `Blob`, text encoding, and more — plus
TypeScript/JSX support (via SWC or Oxc) and an ECMAScript conformance suite (test262) run against
the bare engine. It's under active development; expect gaps.

## Quick start

```sh
cargo build --workspace              # build everything
cargo run -p klaver-cli -- file.ts   # run a module through the CLI
cargo run -p klaver-cli --           # no path -> drop into a REPL
cargo test --workspace               # run the test suite
```

See [`AGENTS.md`](./AGENTS.md) for the full workspace layout, crate-by-crate breakdown, and
contributor/agent guidance.
