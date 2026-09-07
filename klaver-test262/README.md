# klaver-test262

Runs the [test262](https://github.com/tc39/test262) ECMAScript conformance suite against a bare
klaver engine (`klaver_vm::Vm` with no WinterTC globals - just the JS engine itself), and tracks
results against a checked-in baseline so CI only needs to care about regressions, not the full
set of gaps in a partial implementation.

## Setup

The suite itself is vendored as a git submodule at `test262/` (not checked into this repo):

```sh
git submodule update --init test262
```

## Running

```sh
# Everything (minus --exclude'd directories, `intl402` by default)
cargo run -p klaver-test262

# A subset - paths are relative to test262/test
cargo run -p klaver-test262 -- language/expressions/addition built-ins/Promise

# More detail on every failure, not just regressions
cargo run -p klaver-test262 -- --verbose

# Tune concurrency / per-test timeout (a hung test only leaks that one execution's engine
# thread, not the whole run - see src/runner.rs)
cargo run -p klaver-test262 -- --jobs 24 --timeout-ms 8000

# See which test a slow/stuck run is currently on
cargo run -p klaver-test262 -- --trace
```

By default, results are compared against `klaver-test262/expectations.txt`: a plain-text list of
every test currently expected to fail or be skipped. The run exits non-zero only if something
*not* in that file now fails or is skipped (a regression). Tests that pass now but are still
listed as failing are reported too, since they mean the baseline is stale.

To (re)generate the baseline after intentionally fixing or accepting new gaps:

```sh
cargo run -p klaver-test262 -- --update-expectations
```

## Scope

- `intl402/` (ECMA-402/`Intl`) is excluded by default - the bare engine has no `Intl` global.
- `staging/sm/Set/` is excluded by default: `intersection.js` there reproducibly **aborts the
  whole process** with `JS_FreeRuntime: Assertion 'list_empty(&rt->gc_obj_list)' failed` - a real
  GC-tracing bug somewhere in klaver/QuickJS's `Set.prototype` methods (`intersection`, `union`,
  etc.), separate from anything this harness itself does. It's a C-level `assert()`+`abort()`, so
  it can't be caught from Rust; a run that includes it dies without writing results. Worth fixing
  separately - this is only excluded so the harness stays usable in the meantime.
- `module`-flagged tests are reported as **skipped**, not run - this harness doesn't yet drive
  module graph evaluation/import resolution for test262's module-code tests.
- Tests needing the `$262` host-defined realm/agent object aren't specially supported; they show
  up as ordinary (expected) failures.
- Each (test file, strict-mode) execution gets its own freshly created `Vm`/engine instance, so
  test state can never leak between cases, and a hung test only strands that one execution rather
  than the whole run.
