// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/filters/build.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::collections::BTreeMap;
use crate::rtk::ansi::strip_ansi;

/// Filter build / compiler command outputs.
pub fn filter_build(cmd: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{}\n{}", stdout, stderr)
    };
    let clean = strip_ansi(&combined);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let bin = parts.first().copied().unwrap_or("");
    let sub = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    match bin {
        "cargo" => filter_cargo_build(&clean, sub, exit_code),
        "tsc" => filter_tsc(&clean, exit_code),
        "go" if sub == "build" || sub == "vet" => filter_go_build(&clean, exit_code),
        "dotnet" if sub == "build" => filter_dotnet_build(&clean, exit_code),
        "mvn" | "mvn.cmd" => filter_mvn_build(&clean, exit_code),
        "gradle" | "gradlew" => filter_gradle_build(&clean, exit_code),
        "mypy" => filter_mypy(&clean, exit_code),
        "ruff" => filter_ruff(&clean, exit_code),
        "make" => filter_make(&clean, exit_code),
        _ => clean,
    }
}

/// Filter `cargo build`, `cargo check`, `cargo clippy`.
pub fn filter_cargo_build(raw: &str, subcommand: &str, exit_code: i32) -> String {
    let mut compiled_count = 0;
    let mut error_blocks = Vec::new();
    let mut warning_blocks = Vec::new();
    let mut current_block = Vec::new();
    let mut in_error = false;
    let mut in_warning = false;
    let mut finished_line = None;

    for line in raw.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("Compiling ") || trimmed.starts_with("Checking ") {
            compiled_count += 1;
            continue;
        }
        if trimmed.starts_with("Downloading ")
            || trimmed.starts_with("Downloaded ")
            || trimmed.starts_with("Updating ")
            || trimmed.starts_with("Locking ")
        {
            continue;
        }
        if trimmed.starts_with("Finished ") {
            finished_line = Some(trimmed.to_string());
            continue;
        }

        if (line.starts_with("error:") || line.starts_with("error["))
            && (line.contains("could not compile") || line.contains("aborting due to"))
        {
            continue;
        }

        if line.starts_with("error[") || line.starts_with("error:") {
            if in_error && !current_block.is_empty() {
                error_blocks.push(current_block.join("\n"));
                current_block.clear();
            } else if in_warning && !current_block.is_empty() {
                warning_blocks.push(current_block.join("\n"));
                current_block.clear();
            }
            in_error = true;
            in_warning = false;
            current_block.push(line.to_string());
            continue;
        }

        if line.starts_with("warning:") || line.starts_with("warning[") {
            if in_error && !current_block.is_empty() {
                error_blocks.push(current_block.join("\n"));
                current_block.clear();
            } else if in_warning && !current_block.is_empty() {
                warning_blocks.push(current_block.join("\n"));
                current_block.clear();
            }
            in_warning = true;
            in_error = false;
            current_block.push(line.to_string());
            continue;
        }

        if (in_error || in_warning) && (line.starts_with(' ') || line.starts_with("   ") || line.starts_with('|')) {
            current_block.push(line.to_string());
            continue;
        }

        if (in_error || in_warning) && line.trim().is_empty() && current_block.len() > 2 {
            if in_error {
                error_blocks.push(current_block.join("\n"));
            } else {
                warning_blocks.push(current_block.join("\n"));
            }
            current_block.clear();
            in_error = false;
            in_warning = false;
        }
    }

    if in_error && !current_block.is_empty() {
        error_blocks.push(current_block.join("\n"));
    } else if in_warning && !current_block.is_empty() {
        warning_blocks.push(current_block.join("\n"));
    }

    let mut out = Vec::new();

    let has_errors = !error_blocks.is_empty();
    if has_errors {
        out.push(format!("cargo {}: {} errors", subcommand, error_blocks.len()));
        out.extend(error_blocks);
    }

    if !warning_blocks.is_empty() {
        if !has_errors {
            out.push(format!("cargo {}: {} warnings", subcommand, warning_blocks.len()));
            out.extend(warning_blocks);
        } else {
            out.push(format!("(... and {} warnings suppressed)", warning_blocks.len()));
        }
    }

    if let Some(fin) = finished_line {
        out.push(fin);
    } else if out.is_empty() {
        if exit_code == 0 {
            out.push(format!("cargo {}: ok (compiled {} crates)", subcommand, compiled_count));
        } else {
            out.push(format!("cargo {}: failed (exit code {})", subcommand, exit_code));
        }
    }

    out.join("\n")
}

/// Filter `tsc` (TypeScript compiler) outputs.
pub fn filter_tsc(raw: &str, exit_code: i32) -> String {
    let mut file_errors: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total_errors = 0;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains(": error TS") {
            total_errors += 1;
            if let Some((loc, msg)) = trimmed.split_once(": error TS") {
                let err = format!("TS{}", msg);
                file_errors.entry(loc.to_string()).or_default().push(err);
            }
        }
    }

    if total_errors == 0 {
        if exit_code == 0 {
            "tsc: 0 errors".to_string()
        } else {
            raw.lines().take(30).collect::<Vec<&str>>().join("\n")
        }
    } else {
        let mut out = vec![format!("tsc: {} errors in {} files", total_errors, file_errors.len())];
        for (file, errs) in file_errors {
            out.push(format!("{}:", file));
            for e in errs {
                out.push(format!("  {}", e));
            }
        }
        out.join("\n")
    }
}

/// Filter `go build` and `go vet` outputs.
pub fn filter_go_build(raw: &str, exit_code: i32) -> String {
    let lines: Vec<&str> = raw
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .collect();

    if lines.is_empty() {
        if exit_code == 0 {
            "go build: ok".to_string()
        } else {
            format!("go build: failed (exit code {})", exit_code)
        }
    } else {
        lines.join("\n")
    }
}

/// Filter `dotnet build` outputs.
pub fn filter_dotnet_build(raw: &str, exit_code: i32) -> String {
    let mut diagnostics = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains(": error CS") || trimmed.contains(": warning CS") {
            diagnostics.push(trimmed.to_string());
        }
    }

    if diagnostics.is_empty() {
        if exit_code == 0 {
            "dotnet build: ok".to_string()
        } else {
            raw.lines().take(25).collect::<Vec<&str>>().join("\n")
        }
    } else {
        diagnostics.join("\n")
    }
}

/// Filter `mvn` (Maven) build outputs.
pub fn filter_mvn_build(raw: &str, exit_code: i32) -> String {
    let mut errors = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[ERROR]") {
            errors.push(trimmed.to_string());
        }
    }

    if errors.is_empty() {
        if exit_code == 0 {
            "mvn: BUILD SUCCESS".to_string()
        } else {
            format!("mvn: BUILD FAILURE (exit code {})", exit_code)
        }
    } else {
        errors.join("\n")
    }
}

/// Filter `gradle` / `gradlew` build outputs.
pub fn filter_gradle_build(raw: &str, exit_code: i32) -> String {
    let mut failures = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("FAILURE:") || trimmed.starts_with("* What went wrong:") || trimmed.starts_with("> ") {
            failures.push(trimmed.to_string());
        }
    }

    if failures.is_empty() {
        if exit_code == 0 {
            "gradle: BUILD SUCCESSFUL".to_string()
        } else {
            format!("gradle: BUILD FAILED (exit code {})", exit_code)
        }
    } else {
        failures.join("\n")
    }
}

/// Filter `mypy` Python type checking outputs.
pub fn filter_mypy(raw: &str, exit_code: i32) -> String {
    let mut errors = Vec::new();
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains(": error:") {
            errors.push(trimmed.to_string());
        } else if trimmed.starts_with("Found ") && trimmed.contains("error") {
            summary = trimmed.to_string();
        } else if trimmed.starts_with("Success: no issues found") {
            return "mypy: 0 errors".to_string();
        }
    }

    if errors.is_empty() {
        if exit_code == 0 {
            "mypy: ok".to_string()
        } else {
            format!("mypy: failed (exit code {})", exit_code)
        }
    } else {
        if !summary.is_empty() {
            errors.push(summary);
        }
        errors.join("\n")
    }
}

/// Filter `ruff` Python linter outputs.
pub fn filter_ruff(raw: &str, exit_code: i32) -> String {
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        if exit_code == 0 {
            "ruff: ok".to_string()
        } else {
            format!("ruff: failed (exit code {})", exit_code)
        }
    } else {
        lines.join("\n")
    }
}

/// Filter `make` outputs.
pub fn filter_make(raw: &str, exit_code: i32) -> String {
    let errors: Vec<&str> = raw
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.contains(": error:") || t.contains("make: ***") || t.starts_with("Error ")
        })
        .collect();

    if errors.is_empty() {
        if exit_code == 0 {
            "make: ok".to_string()
        } else {
            raw.lines().take(20).collect::<Vec<&str>>().join("\n")
        }
    } else {
        errors.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_cargo_build_success() {
        let raw = "   Compiling foo v0.1.0\n   Compiling bar v0.1.0\n    Finished dev [unoptimized + debuginfo] target(s) in 1.2s\n";
        let out = filter_cargo_build(raw, "build", 0);
        assert!(out.contains("Finished dev [unoptimized + debuginfo] target(s) in 1.2s"));
        assert!(!out.contains("Compiling foo"));
    }

    #[test]
    fn test_filter_cargo_build_errors() {
        let raw = "   Compiling foo v0.1.0\nerror[E0308]: mismatched types\n  --> src/main.rs:10:5\n   |\n10 |     let x: u32 = \"str\";\n   |                  ^^^^^ expected u32, found &str\n\nerror: could not compile `foo` due to previous error\n";
        let out = filter_cargo_build(raw, "build", 101);
        assert!(out.contains("cargo build: 1 errors"));
        assert!(out.contains("error[E0308]: mismatched types"));
    }

    #[test]
    fn test_filter_tsc() {
        let raw = "src/main.ts(12,5): error TS2322: Type 'string' is not assignable to type 'number'.\n";
        let out = filter_tsc(raw, 2);
        assert!(out.contains("tsc: 1 errors in 1 files"));
        assert!(out.contains("src/main.ts(12,5):"));
        assert!(out.contains("TS2322: Type 'string' is not assignable to type 'number'."));
    }
}
