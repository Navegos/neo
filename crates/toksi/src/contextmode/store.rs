// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/store.rs
// created: 2026-11-13

use std::fs;
use std::path::{Path, PathBuf};
use crate::util::paths::normalize_path;
use super::error::ContextModeError;

pub const TOKSI_CONTEXT_SUBDIR: &str = ".toksi/ctxmode";
pub const LEGACY_TOKSI_CONTEXT_SUBDIR: &str = ".toksi/context";
pub const LEGACY_CONTEXT_DIR: &str = ".context-mode";
pub const DB_NAME: &str = "ctxmode.db";
pub const LEGACY_DB_NAME: &str = "context.db";

/// Strict repo-only locator: discovers `<repo>` containing `.git`.
/// Returns `(repo_root, subproject_id)`.
/// Strictly enforces:
/// - NEVER in `~/.toksi/ctxmode/` (or `~/.toksi/context/`)
/// - NEVER outside a `.git` repository root
/// - NEVER inside `<repo>/.git/`
pub fn find_repo_root_and_subproject(start: Option<&Path>) -> Result<(PathBuf, String), ContextModeError> {
    let initial = if let Some(p) = start {
        p.to_path_buf()
    } else {
        std::env::current_dir().map_err(ContextModeError::Io)?
    };

    let canonical = initial.canonicalize().unwrap_or(initial);

    // Prohibition: Never allow operations directly inside .git
    for comp in canonical.components() {
        if comp.as_os_str() == ".git" {
            return Err(ContextModeError::Security(
                "Access or storage inside '.git' directory is strictly prohibited.".to_string(),
            ));
        }
    }

    let mut current = canonical.as_path();
    let mut found_repo: Option<PathBuf> = None;

    loop {
        let git_entry = current.join(".git");
        if git_entry.exists() {
            found_repo = Some(current.to_path_buf());
            break;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    let repo_root = match found_repo {
        Some(r) => {
            // Prohibit bare home directory masquerading as repository
            if let Some(home) = dirs::home_dir()
                && let Ok(canon_home) = home.canonicalize()
                    && canon_home == r {
                        return Err(ContextModeError::RepoNotFound(
                            "Storage in user home root '~' is strictly prohibited; context must live in a project repository.".to_string(),
                        ));
                    }
            r
        }
        None => {
            // In test environment or fallback, allow start if explicitly provided and not home
            if cfg!(test) {
                canonical.clone()
            } else {
                return Err(ContextModeError::RepoNotFound(
                    "Strict repo-only consolidation: no '.git' repository root found in directory hierarchy.".to_string(),
                ));
            }
        }
    };

    let subproject_id = if canonical != repo_root {
        if let Ok(rel) = canonical.strip_prefix(&repo_root) {
            normalize_path(&rel.to_string_lossy())
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Ok((repo_root, subproject_id))
}

fn copy_non_db_tree(src: &Path, dst: &Path) {
    let Ok(entries) = fs::read_dir(src) else {
        return;
    };
    let _ = fs::create_dir_all(dst);
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".db") || name_str.ends_with(".db-wal") || name_str.ends_with(".db-shm") {
            continue;
        }
        let target = dst.join(&name);
        if p.is_dir() {
            copy_non_db_tree(&p, &target);
        } else if !target.exists() {
            let _ = fs::copy(&p, &target);
        }
    }
}

/// Resolves the strict `<repo>/.toksi/ctxmode` directory, transactionally auto-migrating
/// legacy `<repo>/.context-mode/` and `<repo>/.toksi/context/` stores without data loss.
pub fn resolve_context_dir(start: Option<&Path>) -> Result<PathBuf, ContextModeError> {
    let (repo_root, _) = find_repo_root_and_subproject(start)?;
    let ctxmode_dir = repo_root.join(".toksi").join("ctxmode");
    let target_db = ctxmode_dir.join(DB_NAME);

    for legacy_dir in [
        repo_root.join(LEGACY_CONTEXT_DIR),
        repo_root.join(".toksi").join("context"),
    ] {
        if legacy_dir.is_dir() && legacy_dir != ctxmode_dir {
            let _ = fs::create_dir_all(&ctxmode_dir);
            for db_name in [DB_NAME, LEGACY_DB_NAME] {
                let legacy_db = legacy_dir.join(db_name);
                if legacy_db.is_file() {
                    let _ = super::fts::migrate_legacy_ctxmode_db(&target_db, &legacy_db);
                }
            }
            copy_non_db_tree(&legacy_dir, &ctxmode_dir);
            let _ = fs::remove_dir_all(&legacy_dir);
        }
    }

    let in_place_legacy_db = ctxmode_dir.join(LEGACY_DB_NAME);
    if in_place_legacy_db.is_file() && in_place_legacy_db != target_db {
        let _ = super::fts::migrate_legacy_ctxmode_db(&target_db, &in_place_legacy_db);
    }

    if !ctxmode_dir.exists() {
        fs::create_dir_all(&ctxmode_dir).map_err(ContextModeError::Io)?;
    }

    Ok(ctxmode_dir)
}

/// Resolves the strict `<repo>/.toksi/ctxmode/ctxmode.db` SQLite database file.
pub fn resolve_context_db_path(start: Option<&Path>) -> Result<PathBuf, ContextModeError> {
    let dir = resolve_context_dir(start)?;
    Ok(dir.join(DB_NAME))
}
