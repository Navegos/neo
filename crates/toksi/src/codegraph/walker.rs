// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/walker.rs
// created: 2026-10-28
// lastModified: 2026-11-16

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::WalkBuilder;

pub const MAX_SOURCE_FILE_SIZE: u64 = 2 * 1024 * 1024; // 2 MB

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkedFile {
    pub path: PathBuf,
    pub rel_path: String,
    pub language: &'static str,
    pub mtime: u64,
    pub size: u64,
}

pub fn detect_language(path: &Path) -> Option<&'static str> {
    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        let lower_name = file_name.to_ascii_lowercase();
        if lower_name == "dockerfile" {
            return Some("dockerfile");
        }
        if lower_name == "makefile" || lower_name == "gnumakefile" {
            return Some("make");
        }
        if lower_name == "cmakelists.txt" {
            return Some("cmake");
        }
    }

    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "rs" => Some("rust"),
        "ts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "jsx" => Some("jsx"),
        "py" | "pyi" => Some("python"),
        "go" => Some("go"),
        "c" | "h" => Some("c"),
        "cpp" | "hpp" | "cc" | "cxx" | "hh" | "hxx" => Some("cpp"),
        "cs" => Some("csharp"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "scala" | "sc" => Some("scala"),
        "php" => Some("php"),
        "rb" => Some("ruby"),
        "swift" => Some("swift"),
        "lua" => Some("lua"),
        "dart" => Some("dart"),
        "r" => Some("r"),
        "zig" => Some("zig"),
        "jl" => Some("julia"),
        "ex" | "exs" => Some("elixir"),
        "erl" | "hrl" => Some("erlang"),
        "ml" | "mli" => Some("ocaml"),
        "hs" => Some("haskell"),
        "html" | "htm" => Some("html"),
        "css" => Some("css"),
        "scss" => Some("scss"),
        "vue" => Some("vue"),
        "svelte" => Some("svelte"),
        "graphql" | "gql" => Some("graphql"),
        "sh" | "bash" | "zsh" => Some("bash"),
        "dockerfile" => Some("dockerfile"),
        "tf" | "hcl" => Some("hcl"),
        "nix" => Some("nix"),
        "mk" => Some("make"),
        "cmake" => Some("cmake"),
        "json" => Some("json"),
        "yaml" | "yml" => Some("yaml"),
        "toml" => Some("toml"),
        "proto" => Some("protobuf"),
        "sql" => Some("sql"),
        "md" | "markdown" => Some("markdown"),
        "tex" | "latex" => Some("latex"),
        _ => None,
    }
}

pub fn is_build_or_vendor_artifact(name: &str) -> bool {
    name == ".git"
        || name == "node_modules"
        || name == "target"
        || name == ".codegraph"
        || name == ".projectmem"
        || name == ".toksi"
        || name == "dist"
        || name == "build"
        || name == "out"
        || name == "vendor"
        || name == "__pycache__"
        || name == ".venv"
        || name == "venv"
        || name.ends_with(".lock")
}

/// Returns true if a normalized relative path falls under a build/vendor artifact
/// directory, preserving `build/` packages nested beneath a `src/` source tree.
pub fn is_skipped_rel_path(rel_path: &str) -> bool {
    let norm = crate::util::paths::normalize_path(rel_path);
    let mut saw_src = false;
    let parts: Vec<&str> = norm.split('/').filter(|s| !s.is_empty()).collect();
    for (idx, seg) in parts.iter().enumerate() {
        let is_dir_seg = idx + 1 < parts.len();
        if *seg == "src" && is_dir_seg {
            saw_src = true;
            continue;
        }
        if seg.eq_ignore_ascii_case("build") && is_dir_seg {
            if !saw_src {
                return true;
            }
            continue;
        }
        if is_build_or_vendor_artifact(seg) {
            return true;
        }
    }
    false
}

/// Detects whether a relative path belongs to a test suite or test-support tree.
pub fn is_test_file(rel_path: &str) -> bool {
    let norm = crate::util::paths::normalize_path(rel_path);
    let file_name = norm.rsplit('/').next().unwrap_or(&norm);
    let lower_name = file_name.to_ascii_lowercase();

    if lower_name.starts_with("test_") || lower_name.starts_with("test.") {
        return true;
    }

    if let Some((stem, ext)) = file_name.rsplit_once('.')
        && !ext.is_empty()
    {
        let lower_stem = stem.to_ascii_lowercase();
        for suffix in [
            "_test", ".test", "-test",
            "_tests", ".tests", "-tests",
            "_unittest", ".unittest", "-unittest",
            "_unittests", ".unittests", "-unittests",
            "_spec", ".spec", "-spec",
            "_specs", ".specs", "-specs",
        ] {
            if lower_stem.ends_with(suffix) {
                return true;
            }
        }
        for camel_suffix in ["Test", "Tests", "TestCase", "Tester", "Spec", "Specs"] {
            if stem.ends_with(camel_suffix) && stem.len() > camel_suffix.len() {
                return true;
            }
        }
    }

    let parts: Vec<&str> = norm.split('/').filter(|s| !s.is_empty()).collect();
    let dir_parts = if parts.len() > 1 { &parts[..parts.len() - 1] } else { &[][..] };
    for seg in dir_parts {
        let seg_lower = seg.to_ascii_lowercase();
        if matches!(
            seg_lower.as_str(),
            "tests"
                | "test"
                | "__tests__"
                | "spec"
                | "specs"
                | "testlib"
                | "testing"
                | "e2e"
                | "testdata"
                | "testutil"
                | "testutils"
                | "test-utils"
                | "test_utils"
                | "fakes"
                | "mocks"
                | "__mocks__"
                | "stubs"
                | "unittests"
        ) {
            return true;
        }
        if seg_lower.ends_with("-test")
            || seg_lower.ends_with("_test")
            || seg_lower.ends_with("-tests")
            || seg_lower.ends_with("_tests")
            || seg_lower.ends_with("-testing")
            || seg_lower.ends_with("_testing")
            || seg_lower.ends_with("-unittest")
            || seg_lower.ends_with("_unittest")
            || seg_lower.ends_with("-unittests")
            || seg_lower.ends_with("_unittests")
        {
            return true;
        }
        if (seg.ends_with("Test") || seg.ends_with("Tests") || seg.ends_with("Spec"))
            && seg.len() > 4
        {
            return true;
        }
    }
    false
}

pub fn build_project_ignore(root: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(root);
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        let _ = builder.add(&gitignore_path);
    }
    let ignore_path = root.join(".ignore");
    if ignore_path.exists() {
        let _ = builder.add(&ignore_path);
    }
    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

pub fn is_inside_git_dir(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == ".git")
}

pub fn find_git_repo_root(start: &Path) -> Option<PathBuf> {
    let canonical = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    if is_inside_git_dir(&canonical) {
        return None;
    }
    let mut cur = canonical.as_path();
    loop {
        if cur.join(".git").exists() {
            return Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => return None,
        }
    }
}

pub fn walk_project_files(root: &Path) -> Vec<WalkedFile> {
    if is_inside_git_dir(root) {
        return Vec::new();
    }
    if !cfg!(test) && find_git_repo_root(root).is_none() {
        return Vec::new();
    }
    let mut files = Vec::new();
    let root_owned = root.to_path_buf();
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .hidden(true)
        .parents(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .add_custom_ignore_filename(".ignore")
        .filter_entry(move |entry| {
            let name = entry.file_name().to_string_lossy();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            if is_dir && entry.depth() > 0 && name.eq_ignore_ascii_case("build") {
                let rel = entry.path().strip_prefix(&root_owned).unwrap_or(entry.path());
                let norm_rel = crate::util::paths::normalize_path(&rel.to_string_lossy());
                return norm_rel
                    .split('/')
                    .rev()
                    .skip(1)
                    .any(|seg| seg.eq_ignore_ascii_case("src"));
            }
            if is_dir && is_build_or_vendor_artifact(&name) {
                return false;
            }
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) && name.ends_with(".lock") {
                return false;
            }
            true
        })
        .build();

    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }

        let p = entry.path();
        let lang = match detect_language(p) {
            Some(l) => l,
            None => continue,
        };

        let metadata = match fs::metadata(p) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let size = metadata.len();
        if size == 0 || size > MAX_SOURCE_FILE_SIZE {
            continue;
        }

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let rel_raw = p
            .strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy();
        let rel_path = crate::util::paths::normalize_path(&rel_raw);

        if crate::codegraph::kernel::textutil::is_generated_file(&rel_path) {
            continue;
        }

        files.push(WalkedFile {
            path: p.to_path_buf(),
            rel_path,
            language: lang,
            mtime,
            size,
        });
    }

    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walker_detect_language() {
        assert_eq!(detect_language(Path::new("main.rs")), Some("rust"));
        assert_eq!(detect_language(Path::new("app.ts")), Some("typescript"));
        assert_eq!(detect_language(Path::new("script.py")), Some("python"));
        assert_eq!(detect_language(Path::new("readme.txt")), None);
    }

    #[test]
    fn test_build_or_vendor_artifact() {
        assert!(is_build_or_vendor_artifact("node_modules"));
        assert!(is_build_or_vendor_artifact("target"));
        assert!(is_build_or_vendor_artifact(".git"));
        assert!(is_build_or_vendor_artifact("Cargo.lock"));
        assert!(!is_build_or_vendor_artifact("src"));
    }

    #[test]
    fn test_build_project_ignore() {
        let temp_dir = std::env::temp_dir().join("toksi_walker_test");
        let _ = fs::create_dir_all(&temp_dir);
        let gitignore = temp_dir.join(".gitignore");
        let _ = fs::write(&gitignore, "*.tmp\nbuild/\n");

        let gi = build_project_ignore(&temp_dir);
        let test_file = temp_dir.join("test.tmp");
        assert!(gi.matched_path_or_any_parents(&test_file, false).is_ignore());

        let ok_file = temp_dir.join("test.rs");
        assert!(!gi.matched_path_or_any_parents(&ok_file, false).is_ignore());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
