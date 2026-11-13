// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/kernel/stack.rs
// created: 2026-10-28
// lastModified: 2026-11-13

//! Safe stack-budget guard for the recursive AST walkers (#1581).
//!
//! Uses a descent budget anchored from the entry stack pointer without any
//! `unsafe` OS stack-bounds FFI calls, preventing stack overflows on 1 MiB+
//! worker threads and turning excessive recursion into a clean `defer:` error.

use std::cell::Cell;

/// Descent budget measured from the entry stack pointer — safe on 1 MiB+
/// worker stacks without needing unsafe OS FFI queries.
const FALLBACK_BUDGET: usize = 384 * 1024;

thread_local! {
    /// Lowest stack-pointer value the walker may reach before the guard
    /// trips. `0` = not computed yet for this thread.
    static THRESHOLD: Cell<usize> = const { Cell::new(0) };
    /// Latched by `exhausted()`; read by `run_guarded` after the walk.
    static OVERFLOWED: Cell<bool> = const { Cell::new(false) };
}

/// Approximate current stack pointer: the address of a local. Stacks grow
/// downward on x86_64 and aarch64.
#[inline(always)]
fn current_sp() -> usize {
    let marker = 0u8;
    std::hint::black_box(&marker) as *const u8 as usize
}

/// Arm the guard for one extraction on the calling thread: clear the latch
/// and anchor the descent budget from the entry stack pointer.
pub fn begin() {
    OVERFLOWED.with(|o| o.set(false));
    THRESHOLD.with(|t| t.set(current_sp().saturating_sub(FALLBACK_BUDGET).max(1)));
}

/// `true` once the walker has descended past the entry budget. Latches
/// `OVERFLOWED` so `run_guarded` can discard the partial result.
#[inline(always)]
pub fn exhausted() -> bool {
    let threshold = THRESHOLD.with(|t| t.get());
    if threshold == 0 {
        begin();
        return exhausted();
    }
    if current_sp() < threshold {
        OVERFLOWED.with(|o| o.set(true));
        true
    } else {
        false
    }
}

/// Whether the guard tripped since the last `begin()`.
pub fn overflowed() -> bool {
    OVERFLOWED.with(|o| o.get())
}

/// Run one extraction under the guard. A walk that tripped the guard returns
/// a `defer:` error regardless of what the truncated walk produced.
pub fn run_guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    begin();
    let out = f();
    if overflowed() {
        return Err(
            "defer: nesting too deep for the native walker — wasm recovery handles it".to_string(),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_STACK: usize = 1 << 20;
    const DEPTH: usize = 30_000;

    fn on_small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        std::thread::Builder::new()
            .stack_size(SMALL_STACK)
            .spawn(f)
            .expect("spawn")
            .join()
            .expect("walker thread panicked")
    }

    fn nested_parens(prefix: &str, suffix: &str) -> String {
        format!("{prefix}{}1{}{suffix}", "(".repeat(DEPTH), ")".repeat(DEPTH))
    }

    #[test]
    fn safe_budget_arms_without_unsafe() {
        on_small_stack(|| {
            begin();
            assert!(!overflowed());
            assert!(!exhausted());
            let sp = current_sp();
            let threshold = THRESHOLD.with(|t| t.get());
            assert!(
                threshold < sp && sp - threshold <= FALLBACK_BUDGET + 4096,
                "threshold={threshold:#x}, sp={sp:#x}, budget={FALLBACK_BUDGET}"
            );
        });
    }

    #[test]
    fn deep_braces_c_defer_instead_of_crashing() {
        let src = format!("void foo(void) {{\n{}{}\n}}\n", "{".repeat(DEPTH), "}".repeat(DEPTH));
        let r = on_small_stack(move || run_guarded(|| crate::codegraph::kernel::ccpp::extract("deep.c", &src, "c")));
        let err = r.err().expect("deep nesting must defer");
        assert!(err.starts_with("defer:"), "unexpected error: {err}");
    }

    type Extract = fn(&str) -> Result<crate::codegraph::kernel::buffers::EmitOut, String>;

    #[test]
    fn deep_parens_cpp_rust_ts_python_defer_instead_of_crashing() {
        let cases: [(&str, Extract, String); 4] = [
            ("deep.cpp", |s| crate::codegraph::kernel::ccpp::extract("deep.cpp", s, "cpp"), nested_parens("int f() { return ", "; }\n")),
            ("deep.rs", |s| crate::codegraph::kernel::rustlang::extract("deep.rs", s), nested_parens("fn f() -> i32 { ", " }\n")),
            ("deep.ts", |s| crate::codegraph::kernel::tsjs::extract("deep.ts", s, "typescript"), nested_parens("function f() { return ", "; }\n")),
            ("deep.py", |s| crate::codegraph::kernel::python::extract("deep.py", s), nested_parens("def f():\n    return ", "\n")),
        ];
        for (name, extract, src) in cases {
            let r = on_small_stack(move || run_guarded(|| extract(&src)));
            let err = r.err().unwrap_or_else(|| panic!("{name}: deep nesting must defer"));
            assert!(err.starts_with("defer:"), "{name}: unexpected error: {err}");
        }
    }

    #[test]
    fn normal_files_are_untouched_by_the_guard() {
        let src = "int add(int a, int b) { return a + b; }\nint main(void) { return add(1, 2); }\n";
        let r = on_small_stack(move || run_guarded(|| crate::codegraph::kernel::ccpp::extract("ok.c", src, "c")));
        assert!(r.is_ok(), "a shallow file must not defer: {:?}", r.err());
        assert!(!overflowed());
    }

    #[test]
    fn latch_resets_between_runs() {
        let deep = format!("void foo(void) {{\n{}{}\n}}\n", "{".repeat(DEPTH), "}".repeat(DEPTH));
        on_small_stack(move || {
            assert!(run_guarded(|| crate::codegraph::kernel::ccpp::extract("deep.c", &deep, "c")).is_err());
            let ok = run_guarded(|| crate::codegraph::kernel::ccpp::extract("ok.c", "int x;\n", "c"));
            assert!(ok.is_ok(), "latch leaked into the next run: {:?}", ok.err());
        });
    }
}
