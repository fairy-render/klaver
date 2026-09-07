use std::time::Duration;

use rquickjs::{CatchResultExt, Ctx, Value};

use crate::{
    harness::Harness,
    meta::{Frontmatter, Negative},
};

#[derive(Debug)]
pub enum Outcome {
    Passed,
    Failed(String),
    /// A category this harness deliberately doesn't attempt yet (e.g. module-code tests).
    Skipped(&'static str),
}

enum EvalOutcome {
    Completed,
    Threw { kind: String, message: String },
}

/// Runs one (test, strict-mode) execution in a freshly created, fully isolated engine, and
/// judges the result against the test's `negative` expectation (if any).
///
/// The actual engine work happens inside `spawn_blocking`. `rquickjs`'s `AsyncContext::with`
/// only *awaits* to acquire its lock - the closure itself then runs fully synchronously on
/// whatever thread is polling it (see rquickjs's `context/async.rs`). Test262 does contain a
/// handful of pathologically slow or outright hanging cases, and a synchronous, non-yielding
/// computation cannot be preempted by `tokio::time::timeout` on the same executor thread: the
/// timer future never gets polled while that thread is busy. Running it on a dedicated
/// blocking-pool thread instead means a hang only strands that one thread (leaked, since
/// there's no way to actually abort it) rather than stalling every other concurrent test.
pub async fn run_case(
    harness: &mut Harness,
    body: &str,
    meta: &Frontmatter,
    strict: bool,
    timeout: Duration,
) -> anyhow::Result<Outcome> {
    if meta.is_module() {
        return Ok(Outcome::Skipped("module"));
    }

    let source = harness.compose(meta, body, strict)?;
    let is_async = meta.is_async();
    let negative = meta.negative.clone();

    let rt = tokio::runtime::Handle::current();
    let task = tokio::task::spawn_blocking(move || rt.block_on(execute(source, is_async)));

    let eval_outcome = match tokio::time::timeout(timeout, task).await {
        Ok(Ok(Ok(outcome))) => outcome,
        Ok(Ok(Err(err))) => return Ok(Outcome::Failed(format!("engine error: {err}"))),
        Ok(Err(join_err)) => return Ok(Outcome::Failed(format!("panicked: {join_err}"))),
        Err(_) => return Ok(Outcome::Failed("timed out".to_string())),
    };

    Ok(judge(eval_outcome, negative.as_ref()))
}

/// Creates a fresh engine, evaluates `source`, and - for `async`-flagged tests that didn't
/// throw synchronously - drains the promise job queue and inspects what `$DONE` reported.
async fn execute(source: String, is_async: bool) -> Result<EvalOutcome, klaver_core::RuntimeError> {
    let vm = klaver_vm::Options::default().build().await?;

    let outcome = vm.with(move |ctx| Ok(eval_source(ctx, &source))).await?;

    // A synchronous throw already fully determines the outcome - an async test that throws
    // before reaching its `$DONE`-driving code never got a chance to signal completion either
    // way, so it's judged the same as any other throw.
    let outcome = match outcome {
        EvalOutcome::Threw { kind, message } => EvalOutcome::Threw { kind, message },
        EvalOutcome::Completed if !is_async => EvalOutcome::Completed,
        EvalOutcome::Completed => {
            // Drain the promise job queue so any `.then()`-chained `$DONE` call runs, then
            // read back what `print()` (our stand-in for the host print hook) captured.
            vm.idle().await;

            let lines = vm
                .with(|ctx| {
                    Ok(ctx
                        .globals()
                        .get::<_, Vec<String>>("__test262_output")
                        .unwrap_or_default())
                })
                .await?;

            judge_async_output(&lines)
        }
    };

    Ok(outcome)
}

fn eval_source<'js>(ctx: Ctx<'js>, source: &str) -> EvalOutcome {
    // `print` stands in for the host print hook the test262 harness (doneprintHandle.js)
    // expects; we redirect it into a global array so it can be inspected after the fact,
    // including after draining the job queue for async tests.
    const BOOTSTRAP: &str = "globalThis.__test262_output = [];\n\
         function print(msg) { __test262_output.push(String(msg)); }\n";

    let full = format!("{BOOTSTRAP}{source}");
    let result: rquickjs::Result<Value> = ctx.eval(full);

    match result.catch(&ctx) {
        Ok(_) => EvalOutcome::Completed,
        Err(caught) => {
            let (kind, message) = describe_caught(&ctx, caught);
            EvalOutcome::Threw { kind, message }
        }
    }
}

fn describe_caught<'js>(ctx: &Ctx<'js>, caught: rquickjs::CaughtError<'js>) -> (String, String) {
    let value = match caught {
        rquickjs::CaughtError::Exception(exc) => exc.into_value(),
        rquickjs::CaughtError::Value(value) => value,
        rquickjs::CaughtError::Error(err) => {
            return (String::new(), err.to_string());
        }
    };

    describe_value(ctx, value)
}

fn describe_value<'js>(_ctx: &Ctx<'js>, value: Value<'js>) -> (String, String) {
    if let Some(s) = value.as_string() {
        return (String::new(), s.to_string().unwrap_or_default());
    }

    let Some(obj) = value.as_object() else {
        return (String::new(), format!("{value:?}"));
    };

    let kind = obj
        .get::<_, Option<rquickjs::Function>>("constructor")
        .ok()
        .flatten()
        .and_then(|f| f.get::<_, String>("name").ok())
        .unwrap_or_default();

    let message = obj
        .get::<_, Option<String>>("message")
        .ok()
        .flatten()
        .unwrap_or_default();

    (kind, message)
}

fn judge_async_output(output: &[String]) -> EvalOutcome {
    for line in output {
        if line == "Test262:AsyncTestComplete" {
            return EvalOutcome::Completed;
        }
        if let Some(rest) = line.strip_prefix("Test262:AsyncTestFailure:") {
            let rest = rest.trim();
            let (kind, message) = rest.split_once(':').unwrap_or((rest, ""));
            return EvalOutcome::Threw {
                kind: kind.trim().to_string(),
                message: message.trim().to_string(),
            };
        }
    }

    EvalOutcome::Threw {
        kind: String::new(),
        message: "async test never called $DONE".to_string(),
    }
}

fn judge(outcome: EvalOutcome, negative: Option<&Negative>) -> Outcome {
    match (outcome, negative) {
        (EvalOutcome::Completed, None) => Outcome::Passed,
        (EvalOutcome::Completed, Some(neg)) => Outcome::Failed(format!(
            "expected a {} to be thrown, but the test completed normally",
            neg.kind
        )),
        (EvalOutcome::Threw { kind, message }, None) => {
            Outcome::Failed(format!("unexpected throw: {kind}: {message}"))
        }
        (EvalOutcome::Threw { kind, message }, Some(neg)) if kind == neg.kind => {
            let _ = message;
            Outcome::Passed
        }
        (EvalOutcome::Threw { kind, message }, Some(neg)) => Outcome::Failed(format!(
            "expected a {} to be thrown, but got {kind}: {message}",
            neg.kind
        )),
    }
}
