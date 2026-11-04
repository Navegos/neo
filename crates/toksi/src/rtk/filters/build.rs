// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/filters/build.rs
// created: 2026-10-27
// lastModified: 2026-11-04

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
        "npm" | "pnpm" | "yarn" | "bun" | "deno" | "npx" | "pnpx" | "bunx" => {
            filter_npm_like(bin, cmd, &clean, exit_code)
        }
        "pip" | "uv" => filter_pip_uv(bin, cmd, &clean, exit_code),
        "eslint" => filter_eslint(&clean, exit_code),
        "prettier" => filter_prettier(&clean, exit_code),
        "sbt" => filter_sbt_compile(&clean, exit_code),
        "rubocop" => filter_rubocop(&clean, exit_code),
        "phpstan" => filter_phpstan(&clean, exit_code),
        "pint" => filter_pint(&clean, exit_code),
        "golangci-lint" => filter_golangci_lint(&clean, exit_code),
        "sqlfluff" => filter_sqlfluff(&clean, exit_code),
        "next" => filter_next_build(&clean, exit_code),
        "prisma" => filter_prisma(cmd, &clean, exit_code),
        "ng" => filter_ng_build(&clean, exit_code),
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
            if compiled_count > 0 {
                out.push(format!("cargo {}: ok (compiled {} crates)", subcommand, compiled_count));
            } else if raw.starts_with("cargo ") || raw.contains("version") || raw.contains("Usage:") {
                return raw.to_string();
            } else {
                out.push(format!("cargo {}: ok (compiled 0 crates)", subcommand));
            }
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

/// Filter Node/JS package runners (`npm`, `pnpm`, `yarn`, `bun`, `deno`, `npx`).
pub fn filter_npm_like(bin: &str, _cmd: &str, raw: &str, exit_code: i32) -> String {
    let mut out = Vec::new();
    let mut added_line = None;
    let mut error_lines = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if line.starts_with('>') && line.contains('@') {
            continue;
        }
        if trimmed.starts_with("npm WARN") || trimmed.starts_with("npm notice") {
            continue;
        }
        if trimmed.contains("found 0 vulnerabilities") || trimmed.starts_with("Run `npm audit`") {
            continue;
        }
        if trimmed.contains("⸨") || trimmed.contains("⸩") || (trimmed.contains("...") && trimmed.len() < 12) {
            continue;
        }

        if trimmed.starts_with("npm ERR!") || trimmed.starts_with("error ") || trimmed.starts_with("ERR_") {
            error_lines.push(trimmed.to_string());
            continue;
        }

        if trimmed.starts_with("added ")
            || trimmed.starts_with("up to date")
            || trimmed.starts_with("installed ")
            || trimmed.starts_with("Packages: +")
        {
            added_line = Some(trimmed.to_string());
            continue;
        }

        out.push(line);
    }

    if !error_lines.is_empty() {
        return error_lines.join("\n");
    }

    if let Some(added) = added_line {
        return format!("{}: {}", bin, added);
    }

    if out.is_empty() {
        if exit_code == 0 {
            format!("{}: ok", bin)
        } else {
            format!("{}: failed (exit code {})", bin, exit_code)
        }
    } else {
        out.join("\n")
    }
}

/// Filter Python package runners (`pip`, `uv`).
pub fn filter_pip_uv(bin: &str, _cmd: &str, raw: &str, exit_code: i32) -> String {
    let mut out = Vec::new();
    let mut installed_line = None;
    let mut error_lines = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("Collecting ")
            || trimmed.starts_with("Using cached ")
            || trimmed.starts_with("Downloading ")
            || trimmed.starts_with("Requirement already satisfied")
            || (trimmed.starts_with('[') && trimmed.contains('%') && trimmed.contains("kB/s"))
            || (trimmed.starts_with('[') && trimmed.contains('━'))
        {
            continue;
        }

        if trimmed.starts_with("ERROR:") || trimmed.starts_with("error:") {
            error_lines.push(trimmed.to_string());
            continue;
        }

        if trimmed.starts_with("Successfully installed ")
            || trimmed.starts_with("Successfully uninstalled ")
            || trimmed.starts_with("Audited ")
            || trimmed.starts_with("Resolved ")
        {
            installed_line = Some(trimmed.to_string());
            continue;
        }

        out.push(line);
    }

    if !error_lines.is_empty() {
        return error_lines.join("\n");
    }

    if let Some(inst) = installed_line {
        return format!("{}: {}", bin, inst);
    }

    if out.is_empty() {
        if exit_code == 0 {
            format!("{}: ok", bin)
        } else {
            format!("{}: failed (exit code {})", bin, exit_code)
        }
    } else {
        out.join("\n")
    }
}

/// Filter `eslint` output grouping diagnostics by file and rule.
pub fn filter_eslint(raw: &str, exit_code: i32) -> String {
    let mut file_issues: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut current_file = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if (line.starts_with('/') || line.contains(":\\") || line.contains(":/"))
            && !line.contains("error")
            && !line.contains("warning")
        {
            current_file = trimmed.to_string();
            continue;
        }

        if trimmed.contains("error") || trimmed.contains("warning") {
            let is_err = trimmed.contains("error");
            if is_err {
                total_errors += 1;
            } else {
                total_warnings += 1;
            }
            let file = if !current_file.is_empty() {
                current_file.clone()
            } else {
                "issues".to_string()
            };
            file_issues.entry(file).or_default().push(trimmed.to_string());
        }
    }

    if total_errors == 0 && total_warnings == 0 {
        if exit_code == 0 {
            "eslint: 0 issues".to_string()
        } else {
            raw.lines().take(20).collect::<Vec<&str>>().join("\n")
        }
    } else {
        let mut out = vec![format!(
            "eslint: {} errors, {} warnings in {} files",
            total_errors,
            total_warnings,
            file_issues.len()
        )];
        for (file, issues) in file_issues {
            out.push(format!("{}:", file));
            for issue in issues.into_iter().take(10) {
                out.push(format!("  {}", issue));
            }
        }
        out.join("\n")
    }
}

/// Filter `prettier` output: hides unchanged files, shows only formatted or error files.
pub fn filter_prettier(raw: &str, exit_code: i32) -> String {
    let mut formatted = Vec::new();
    let mut errors = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.ends_with("[warn]") || trimmed.contains("Forgot to run Prettier?") {
            formatted.push(trimmed.to_string());
        } else if trimmed.contains("error") || trimmed.contains("SyntaxError") {
            errors.push(trimmed.to_string());
        }
    }

    if !errors.is_empty() {
        return errors.join("\n");
    }

    if !formatted.is_empty() {
        let mut out = vec![format!("prettier: {} files formatted or needing format", formatted.len())];
        out.extend(formatted.into_iter().take(20));
        return out.join("\n");
    }

    if exit_code == 0 {
        "prettier: all files matched format".to_string()
    } else {
        raw.lines().take(20).collect::<Vec<&str>>().join("\n")
    }
}

/// Filter Scala SBT compile output.
pub fn filter_sbt_compile(raw: &str, exit_code: i32) -> String {
    let mut source_count: usize = 0;
    let mut total_time_secs = None;
    let mut error_lines = Vec::new();
    let mut has_success = false;

    for line in raw.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("[info] Compiling ") {
            let rest = trimmed.trim_start_matches("[info] Compiling ");
            if let Some(num_str) = rest.split_whitespace().next() {
                source_count += num_str.parse().unwrap_or(0);
            }
            continue;
        }

        if trimmed.starts_with("[success] Total time: ") {
            let rest = trimmed.trim_start_matches("[success] Total time: ");
            total_time_secs = rest.split_whitespace().next().map(|s| s.to_string());
            has_success = true;
            continue;
        }

        if trimmed.starts_with("[error] ") {
            let err = trimmed.trim_start_matches("[error] ").trim();
            if !err.is_empty() && !err.starts_with("Total time:") && !err.contains("compileIncremental") {
                error_lines.push(err.to_string());
            }
        }
    }

    if !error_lines.is_empty() || exit_code != 0 {
        if !error_lines.is_empty() {
            let mut out = vec![format!("sbt compile: {} errors", error_lines.len())];
            for err in error_lines.iter().take(20) {
                out.push(format!("  [error] {}", err));
            }
            out.join("\n")
        } else {
            format!("sbt compile: FAILED (exit code {})", exit_code)
        }
    } else if has_success || source_count > 0 {
        let mut msg = format!("sbt compile: {} sources compiled", source_count);
        if let Some(t) = total_time_secs {
            msg.push_str(&format!(" ({}s)", t));
        }
        msg
    } else {
        "sbt compile: success".to_string()
    }
}

/// Filter Ruby RuboCop output.
pub fn filter_rubocop(raw: &str, exit_code: i32) -> String {
    let mut offenses = Vec::new();
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains("files inspected,") && (trimmed.contains("offense detected") || trimmed.contains("offenses detected")) {
            summary = trimmed.to_string();
        } else if trimmed.contains(':') && (trimmed.contains(": C: ") || trimmed.contains(": W: ") || trimmed.contains(": E: ") || trimmed.contains(": F: ")) {
            offenses.push(trimmed.to_string());
        }
    }

    if exit_code == 0 && offenses.is_empty() {
        if !summary.is_empty() {
            format!("rubocop: {}", summary)
        } else {
            "rubocop: no offenses detected".to_string()
        }
    } else {
        let mut out = Vec::new();
        if !summary.is_empty() {
            out.push(format!("rubocop: {}", summary));
        } else {
            out.push(format!("rubocop: {} offenses detected", offenses.len()));
        }
        for o in offenses.iter().take(25) {
            out.push(format!("  {}", o));
        }
        if offenses.len() > 25 {
            out.push(format!("  ... +{} more offenses", offenses.len() - 25));
        }
        out.join("\n")
    }
}

/// Filter PHP PHPStan static analysis output.
pub fn filter_phpstan(raw: &str, exit_code: i32) -> String {
    let mut errors = Vec::new();
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[OK]") || trimmed.starts_with("[ERROR]") {
            summary = trimmed.to_string();
        } else if trimmed.contains(".php") || (trimmed.starts_with("Line ") || trimmed.contains("  : ")) {
            errors.push(trimmed.to_string());
        }
    }

    if exit_code == 0 && (errors.is_empty() || summary.starts_with("[OK]")) {
        if !summary.is_empty() {
            format!("phpstan: {}", summary)
        } else {
            "phpstan: [OK] No errors".to_string()
        }
    } else {
        let mut out = Vec::new();
        if !summary.is_empty() {
            out.push(format!("phpstan: {}", summary));
        } else {
            out.push(format!("phpstan: FAILED (exit code {})", exit_code));
        }
        for e in errors.iter().take(25) {
            out.push(format!("  {}", e));
        }
        if errors.len() > 25 {
            out.push(format!("  ... +{} more errors", errors.len() - 25));
        }
        out.join("\n")
    }
}

/// Filter PHP Laravel Pint code style output.
pub fn filter_pint(raw: &str, exit_code: i32) -> String {
    let mut files = Vec::new();
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("PASS") || trimmed.starts_with("FAIL") {
            summary = trimmed.to_string();
        } else if trimmed.contains(".php") {
            files.push(trimmed.to_string());
        }
    }

    if exit_code == 0 && files.is_empty() {
        if !summary.is_empty() {
            format!("pint: {}", summary)
        } else {
            "pint: all files passed".to_string()
        }
    } else {
        let mut out = Vec::new();
        out.push(if !summary.is_empty() { format!("pint: {}", summary) } else { format!("pint: {} files changed/failing", files.len()) });
        for f in files.iter().take(20) {
            out.push(format!("  {}", f));
        }
        out.join("\n")
    }
}

/// Filter Go golangci-lint output.
pub fn filter_golangci_lint(raw: &str, exit_code: i32) -> String {
    let mut issues = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains(".go:") {
            issues.push(trimmed.to_string());
        }
    }

    if exit_code == 0 && issues.is_empty() {
        "golangci-lint: no issues found".to_string()
    } else {
        let mut out = vec![format!("golangci-lint: {} issues found", issues.len())];
        for issue in issues.iter().take(25) {
            out.push(format!("  {}", issue));
        }
        if issues.len() > 25 {
            out.push(format!("  ... +{} more issues", issues.len() - 25));
        }
        out.join("\n")
    }
}

/// Filter SQL sqlfluff lint output.
pub fn filter_sqlfluff(raw: &str, exit_code: i32) -> String {
    let mut issues = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains("|") && (trimmed.contains("L:") || trimmed.contains("AL") || trimmed.contains("LT")) {
            issues.push(trimmed.to_string());
        }
    }

    if exit_code == 0 && issues.is_empty() {
        "sqlfluff: all checks passed".to_string()
    } else {
        let mut out = vec![format!("sqlfluff: {} violations", issues.len())];
        for issue in issues.iter().take(25) {
            out.push(format!("  {}", issue));
        }
        if issues.len() > 25 {
            out.push(format!("  ... +{} more violations", issues.len() - 25));
        }
        out.join("\n")
    }
}

/// Filter Next.js build output.
pub fn filter_next_build(raw: &str, exit_code: i32) -> String {
    let mut routes_static = 0;
    let mut routes_dynamic = 0;
    let mut routes_total = 0;
    let mut errors = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('○') || trimmed.starts_with("(Static)") {
            routes_static += 1;
            routes_total += 1;
        } else if trimmed.starts_with('●') || trimmed.starts_with('◐') || trimmed.starts_with('λ') || trimmed.starts_with("(Dynamic)") {
            routes_dynamic += 1;
            routes_total += 1;
        } else if trimmed.contains("Error:") || trimmed.starts_with("Failed to compile") {
            errors.push(trimmed.to_string());
        }
    }

    if exit_code != 0 || !errors.is_empty() {
        let mut out = vec![format!("next build: FAILED (exit code {})", exit_code)];
        for err in errors.iter().take(15) {
            out.push(format!("  {}", err));
        }
        out.join("\n")
    } else if routes_total > 0 {
        format!("next build: {} routes generated ({} static, {} dynamic)", routes_total, routes_static, routes_dynamic)
    } else {
        "next build: compiled successfully".to_string()
    }
}

/// Filter Prisma CLI output (generate, migrate, db push).
pub fn filter_prisma(cmd: &str, raw: &str, exit_code: i32) -> String {
    let mut clean_lines = Vec::new();
    let mut models = 0;

    for line in raw.lines() {
        // Strip ASCII art and box characters
        if line.contains('█') || line.contains('▀') || line.contains('▄') || line.contains('┌') || line.contains('└') || line.contains('│') {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains("model") && trimmed.contains("generated") {
            models += 1;
        }
        if trimmed.starts_with("✔") || trimmed.starts_with("Prisma schema loaded") || trimmed.contains("The following migration(s)") || trimmed.contains("Database is now in sync") {
            clean_lines.push(trimmed.to_string());
        }
    }

    if exit_code != 0 {
        format!("prisma: failed (exit code {})\n{}", exit_code, raw.lines().take(15).collect::<Vec<_>>().join("\n"))
    } else if !clean_lines.is_empty() {
        let mut out = vec!["prisma: success".to_string()];
        out.extend(clean_lines);
        out.join("\n")
    } else if models > 0 {
        format!("prisma: generated client with {} models", models)
    } else {
        format!("prisma: operation completed ({})", cmd)
    }
}

/// Filter Angular ng build output.
pub fn filter_ng_build(raw: &str, exit_code: i32) -> String {
    let mut bundles = Vec::new();
    let mut errors = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Initial Chunk Files") || trimmed.starts_with("Lazy Chunk Files") || trimmed.starts_with("Build at:") {
            continue;
        }
        if trimmed.ends_with(".js") || trimmed.ends_with(".css") {
            bundles.push(trimmed.to_string());
        } else if trimmed.starts_with("Error:") || trimmed.starts_with("✘ [ERROR]") {
            errors.push(trimmed.to_string());
        }
    }

    if exit_code != 0 || !errors.is_empty() {
        let mut out = vec![format!("ng build: FAILED (exit code {})", exit_code)];
        for e in errors.iter().take(15) {
            out.push(format!("  {}", e));
        }
        out.join("\n")
    } else if !bundles.is_empty() {
        let mut out = vec![format!("ng build: {} chunk files generated", bundles.len())];
        for b in bundles.iter().take(10) {
            out.push(format!("  {}", b));
        }
        out.join("\n")
    } else {
        "ng build: build completed successfully".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_npm_like() {
        let raw = "> project@1.0.0 build\nnpm WARN deprecated\nadded 142 packages in 1.4s\n";
        let out = filter_npm_like("npm", "npm install", raw, 0);
        assert_eq!(out, "npm: added 142 packages in 1.4s");
    }

    #[test]
    fn test_filter_pip_uv() {
        let raw = "Collecting requests\nDownloading requests.whl (50 kB)\nSuccessfully installed requests-2.31.0\n";
        let out = filter_pip_uv("pip", "pip install requests", raw, 0);
        assert_eq!(out, "pip: Successfully installed requests-2.31.0");
    }

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

    #[test]
    fn test_filter_sbt_compile() {
        let raw = "[info] Compiling 12 Scala sources to target/classes ...\n[success] Total time: 3 s\n";
        let out = filter_sbt_compile(raw, 0);
        assert_eq!(out, "sbt compile: 12 sources compiled (3s)");
    }

    #[test]
    fn test_filter_rubocop() {
        let raw = "Inspecting 15 files\n..\napp/models/user.rb:10:5: C: Style/StringLiterals: Prefer single-quoted strings.\n\n15 files inspected, 1 offense detected\n";
        let out = filter_rubocop(raw, 1);
        assert!(out.contains("1 offense detected"));
        assert!(out.contains("Style/StringLiterals"));
    }

    #[test]
    fn test_filter_phpstan() {
        let raw = "Note: Using configuration /path/phpstan.neon.\n 5/5 [▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓] 100%\n\n [OK] No errors\n";
        let out = filter_phpstan(raw, 0);
        assert_eq!(out, "phpstan: [OK] No errors");
    }

    #[test]
    fn test_filter_next_build() {
        let raw = "Route (app)                              Size     First Load JS\n○ /                                      5.2 kB          85 kB\n● /dashboard                             12 kB           92 kB\nλ /api/data                              0 B             80 kB\n";
        let out = filter_next_build(raw, 0);
        assert_eq!(out, "next build: 3 routes generated (1 static, 2 dynamic)");
    }

    #[test]
    fn test_filter_prisma() {
        let raw = "Prisma schema loaded from schema.prisma\n✔ Generated Prisma Client to ./node_modules/@prisma/client in 85ms\n";
        let out = filter_prisma("prisma generate", raw, 0);
        assert!(out.contains("prisma: success"));
        assert!(out.contains("Generated Prisma Client"));
    }
}
