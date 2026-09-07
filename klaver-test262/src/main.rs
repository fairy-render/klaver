mod harness;
mod meta;
mod runner;

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use clap::Parser;
use futures::StreamExt;
use meta::{Frontmatter, parse_frontmatter};
use runner::Outcome;
use walkdir::WalkDir;

/// Runs the test262 ECMAScript conformance suite against klaver's bare JS engine.
#[derive(Parser)]
struct Args {
    /// Specific test files or directories to run, relative to `--test262-dir`/test. Runs
    /// everything (minus `--exclude`d directories) when omitted.
    paths: Vec<PathBuf>,

    /// Path to the test262 checkout.
    #[arg(long, default_value = "test262")]
    test262_dir: PathBuf,

    /// Directory/file name substrings to exclude entirely (matched against the path relative to
    /// `test262/test`). Defaults exclude `intl402` (klaver's bare engine has no `Intl` global)
    /// and `staging/sm/Set` (a test in there - `intersection.js` - reproducibly aborts the
    /// whole process with `JS_FreeRuntime: Assertion 'list_empty(&rt->gc_obj_list)' failed`, a
    /// real GC-tracing bug in klaver/QuickJS's `Set.prototype` methods; it can't be caught from
    /// Rust since it's a C-level `assert()`+`abort()`, so the whole run would otherwise die a
    /// few hundred tests short of the finish line).
    #[arg(long = "exclude", default_values = ["intl402", "staging/sm/Set"])]
    exclude: Vec<String>,

    /// Number of tests to run concurrently.
    #[arg(long, short = 'j')]
    jobs: Option<usize>,

    /// Per-execution timeout, in milliseconds. Guards against tests that hang the engine.
    #[arg(long, default_value_t = 10_000)]
    timeout_ms: u64,

    /// Baseline file of currently-known failing/skipped tests. Compared against on every run
    /// so CI only needs to care about *regressions* (newly failing tests), not the full set of
    /// gaps in a partial implementation.
    #[arg(long, default_value = "klaver-test262/expectations.txt")]
    expectations: PathBuf,

    /// Overwrite `--expectations` with the results of this run instead of comparing against it.
    #[arg(long)]
    update_expectations: bool,

    /// Print every failing test's message, not just the summary.
    #[arg(long)]
    verbose: bool,

    /// Print every test as it starts (to stderr). Useful for spotting which test a slow or
    /// hung run is currently stuck on.
    #[arg(long)]
    trace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Fail,
    Skip,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Fail => "FAIL",
            Status::Skip => "SKIP",
        }
    }
}

struct TestFile {
    /// Path relative to `test262/test`, using `/` separators - the stable id used both for
    /// display and for the expectations file.
    id: String,
    path: PathBuf,
}

fn discover(test_dir: &Path, roots: &[PathBuf], exclude: &[String]) -> anyhow::Result<Vec<TestFile>> {
    let mut out = Vec::new();

    let walk_roots: Vec<PathBuf> = if roots.is_empty() {
        vec![test_dir.to_path_buf()]
    } else {
        roots
            .iter()
            .map(|p| {
                if p.is_absolute() {
                    p.clone()
                } else {
                    test_dir.join(p)
                }
            })
            .collect()
    };

    for root in walk_roots {
        for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("js") {
                continue;
            }

            let Ok(rel) = path.strip_prefix(test_dir) else {
                continue;
            };
            let id = rel.to_string_lossy().replace('\\', "/");

            // Harness self-tests, fixtures (`_FIXTURE.js`) and staging/sm are not real
            // executable test cases.
            if id.starts_with("harness/") || id.ends_with("_FIXTURE.js") {
                continue;
            }
            if exclude.iter().any(|e| id.contains(e.as_str())) {
                continue;
            }

            out.push(TestFile {
                id,
                path: path.to_path_buf(),
            });
        }
    }

    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

struct Job {
    key: String,
    body: String,
    meta: Frontmatter,
    strict: bool,
}

fn expand(files: Vec<TestFile>) -> Vec<Job> {
    let mut jobs = Vec::new();

    for file in files {
        let body = match std::fs::read_to_string(&file.path) {
            Ok(body) => body,
            Err(err) => {
                eprintln!("warning: skipping {}: {err}", file.id);
                continue;
            }
        };
        let meta = parse_frontmatter(&body);

        for &strict in meta.modes() {
            let key = if meta.modes().len() > 1 {
                format!("{}[{}]", file.id, if strict { "strict" } else { "sloppy" })
            } else {
                file.id.clone()
            };

            jobs.push(Job {
                key,
                body: body.clone(),
                meta: meta.clone(),
                strict,
            });
        }
    }

    jobs
}

fn load_expectations(path: &Path) -> anyhow::Result<BTreeMap<String, Status>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = std::fs::read_to_string(path)?;
    let mut map = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((status, key)) = line.split_once(' ') else {
            continue;
        };
        let status = match status {
            "FAIL" => Status::Fail,
            "SKIP" => Status::Skip,
            _ => continue,
        };
        map.insert(key.to_string(), status);
    }

    Ok(map)
}

fn write_expectations(path: &Path, results: &BTreeMap<String, Status>) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut out = String::new();
    out.push_str("# Generated by `cargo run -p klaver-test262 -- --update-expectations`.\n");
    out.push_str("# Lists every test262 case currently failing or skipped, so CI only flags regressions.\n");
    for (key, status) in results {
        out.push_str(status.label());
        out.push(' ');
        out.push_str(key);
        out.push('\n');
    }

    std::fs::write(path, out)?;
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let test_dir = args.test262_dir.join("test");
    if !test_dir.exists() {
        anyhow::bail!(
            "{:?} not found - did you run `git submodule update --init test262`?",
            test_dir
        );
    }

    let files = discover(&test_dir, &args.paths, &args.exclude)?;
    if files.is_empty() {
        anyhow::bail!("no test262 files matched");
    }

    let jobs = expand(files);
    println!("running {} test262 executions...", jobs.len());

    let jobs_count = jobs.len();
    let concurrency = args
        .jobs
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4));
    let timeout = Duration::from_millis(args.timeout_ms);
    let harness_dir = args.test262_dir.join("harness");

    let completed = Arc::new(AtomicUsize::new(0));
    let trace = args.trace;

    let results: Vec<(String, Outcome)> = futures::stream::iter(jobs)
        .map(|job| {
            let harness_dir = harness_dir.clone();
            let completed = completed.clone();
            async move {
                if trace {
                    eprintln!("start {}", job.key);
                }
                let mut harness = harness::Harness::new(harness_dir);
                let outcome = runner::run_case(&mut harness, &job.body, &job.meta, job.strict, timeout)
                    .await
                    .unwrap_or_else(|err| Outcome::Failed(format!("harness error: {err}")));

                let n = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if n % 2000 == 0 {
                    eprintln!("  {n}/{jobs_count}");
                }

                (job.key, outcome)
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let mut current = BTreeMap::new();
    let mut passed = 0usize;
    let mut failed_details: Vec<(String, String)> = Vec::new();

    for (key, outcome) in &results {
        match outcome {
            Outcome::Passed => passed += 1,
            Outcome::Failed(msg) => {
                current.insert(key.clone(), Status::Fail);
                failed_details.push((key.clone(), msg.clone()));
            }
            Outcome::Skipped(reason) => {
                current.insert(key.clone(), Status::Skip);
                let _ = reason;
            }
        }
    }

    println!(
        "\n{passed} passed, {} failed, {} skipped, {} total",
        current.values().filter(|s| **s == Status::Fail).count(),
        current.values().filter(|s| **s == Status::Skip).count(),
        results.len()
    );

    if args.verbose {
        failed_details.sort();
        for (key, msg) in &failed_details {
            println!("FAIL {key}: {msg}");
        }
    }

    if args.update_expectations {
        write_expectations(&args.expectations, &current)?;
        println!("wrote {} entries to {:?}", current.len(), args.expectations);
        return Ok(());
    }

    let baseline = load_expectations(&args.expectations)?;
    // Every key actually exercised by this run, pass or fail - needed to scope the "fixed"
    // check below to tests that were in scope, since a filtered/partial run (via `paths`)
    // legitimately omits most of the baseline without that meaning those tests now pass.
    let ran: std::collections::BTreeSet<&String> = results.iter().map(|(key, _)| key).collect();

    let mut regressions: Vec<&String> = Vec::new();
    let mut fixed: Vec<&String> = Vec::new();

    for key in current.keys() {
        if !baseline.contains_key(key) {
            regressions.push(key);
        }
    }
    for key in baseline.keys() {
        if ran.contains(key) && !current.contains_key(key) {
            fixed.push(key);
        }
    }

    if !fixed.is_empty() {
        println!(
            "\n{} test(s) now pass that were previously expected to fail (run with --update-expectations to record this):",
            fixed.len()
        );
        for key in fixed.iter().take(50) {
            println!("  {key}");
        }
        if fixed.len() > 50 {
            println!("  ... and {} more", fixed.len() - 50);
        }
    }

    if !regressions.is_empty() {
        println!(
            "\n{} REGRESSION(S) - newly failing/skipped tests not in {:?}:",
            regressions.len(),
            args.expectations
        );
        for key in &regressions {
            let msg = failed_details
                .iter()
                .find(|(k, _)| k == *key)
                .map(|(_, m)| m.as_str())
                .unwrap_or("skipped");
            println!("  {key}: {msg}");
        }
        anyhow::bail!("{} regression(s) found", regressions.len());
    }

    println!("\nno regressions against {:?}", args.expectations);
    Ok(())
}
