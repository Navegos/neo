// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/kernel/parse_budget.rs
// created: 2026-11-18
// lastModified: 2026-11-18

//! A parse with a wall-clock budget (#3274).
//!
//! tree-sitter 0.25's error recovery can loop without reading on: a
//! half-typed JSX attribute (`<a {{b:>c:d(e f)` in a .tsx file) never
//! finishes parsing. `extract_file` gives its call a deadline, every
//! language's parse goes through [`parse`], and its progress callback cancels
//! the parse once the deadline passes.

use std::cell::Cell;
use std::time::{Duration, Instant};
use tree_sitter::{ParseOptions, ParseState, Parser, Point, Tree};

thread_local! {
    /// The deadline of the extraction running on this thread, and its budget in ms.
    static DEADLINE: Cell<Option<(Instant, u32)>> = const { Cell::new(None) };
}

/// Restores the previous deadline when the extraction ends, panicking or not.
struct Restore(Option<(Instant, u32)>);

impl Drop for Restore {
    fn drop(&mut self) {
        DEADLINE.with(|d| d.set(self.0));
    }
}

/// Run `f` with a deadline `budget_ms` from now for every [`parse`] it makes;
/// `None` leaves the parses unbounded.
pub fn with_budget<T>(budget_ms: Option<u32>, f: impl FnOnce() -> T) -> T {
    let deadline = budget_ms.map(|ms| (Instant::now() + Duration::from_millis(u64::from(ms)), ms));
    let _restore = Restore(DEADLINE.with(|d| d.replace(deadline)));
    f()
}

/// `parser.parse(source, None)`, cancelled once the extraction's deadline
/// passes: `Err("timeout: ...")` then, and `Err("parser returned null tree")`
/// for any other null tree.
pub fn parse(parser: &mut Parser, source: &str) -> Result<Tree, String> {
    let Some((deadline, ms)) = DEADLINE.with(Cell::get) else {
        return parser
            .parse(source, None)
            .ok_or_else(|| "parser returned null tree".to_string());
    };
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut read = |i: usize, _: Point| if i < len { &bytes[i..] } else { Default::default() };
    let mut cancel = |_: &ParseState| Instant::now() >= deadline;
    let tree = parser.parse_with_options(
        &mut read,
        None,
        Some(ParseOptions::new().progress_callback(&mut cancel)),
    );
    tree.ok_or_else(|| {
        if Instant::now() >= deadline {
            format!("timeout: the parse ran past its {ms}ms budget")
        } else {
            "parser returned null tree".to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tsx_parser() -> Parser {
        let mut parser = Parser::new();
        let lang = crate::codegraph::kernel::langs::grammar_for("tsx");
        assert!(lang.is_some(), "tsx grammar");
        if let Some(l) = lang {
            assert!(parser.set_language(&l).is_ok());
        }
        parser
    }

    #[test]
    fn a_parse_that_never_ends_times_out() {
        let started = Instant::now();
        let out = with_budget(Some(200), || parse(&mut tsx_parser(), "<a {{b:>c:d(e f)"));
        let err = out.err().unwrap_or_default();
        assert!(err.starts_with("timeout:"), "{err}");
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "cancelled late: {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn a_parse_within_its_budget_is_unchanged() {
        let tree = with_budget(Some(10_000), || parse(&mut tsx_parser(), "<a {{b:>c:d(e)"));
        assert!(tree.as_ref().is_ok_and(|t| t.root_node().has_error()));
        let plain = tsx_parser().parse("export const a = <div />;", None);
        let bounded = with_budget(Some(10_000), || {
            parse(&mut tsx_parser(), "export const a = <div />;")
        });
        assert_eq!(
            plain.map(|t| t.root_node().to_sexp()),
            bounded.ok().map(|t| t.root_node().to_sexp())
        );
    }

    #[test]
    fn the_deadline_ends_with_its_extraction() {
        with_budget(Some(10_000), || assert!(DEADLINE.with(Cell::get).is_some()));
        assert!(DEADLINE.with(Cell::get).is_none());
    }
}
