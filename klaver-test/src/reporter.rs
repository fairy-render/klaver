use std::cell::{Cell, RefCell};

use klaver_core::error::CaugthException;

/// Collects test results and prints them to stdout as they come in, mocha/tap style.
pub struct Reporter {
    passed: Cell<usize>,
    failed: Cell<usize>,
    failures: RefCell<Vec<(std::string::String, CaugthException)>>,
}

impl Reporter {
    pub fn new() -> Self {
        Reporter {
            passed: Cell::new(0),
            failed: Cell::new(0),
            failures: RefCell::new(Vec::new()),
        }
    }

    pub fn enter_suite(&self, depth: usize, desc: &str) {
        println!("{}{desc}", indent(depth));
    }

    pub fn pass(&self, depth: usize, desc: &str) {
        self.passed.set(self.passed.get() + 1);
        println!("{}\u{2713} {desc}", indent(depth));
    }

    pub fn fail(&self, depth: usize, path: std::string::String, err: CaugthException) {
        self.failed.set(self.failed.get() + 1);
        let desc = path.rsplit(" > ").next().unwrap_or(&path);
        println!("{}\u{2717} {desc}", indent(depth));
        self.failures.borrow_mut().push((path, err));
    }

    pub fn failed(&self) -> usize {
        self.failed.get()
    }

    pub fn summary(&self) -> std::string::String {
        format!("{} passed, {} failed", self.passed.get(), self.failed.get())
    }

    pub fn failure_report(&self) -> std::string::String {
        let mut report = format!("{} test(s) failed:\n", self.failed.get());
        for (path, err) in self.failures.borrow().iter() {
            report.push_str(&format!("  - {path}: {err}\n"));
        }
        report
    }
}

fn indent(depth: usize) -> std::string::String {
    "  ".repeat(depth)
}
