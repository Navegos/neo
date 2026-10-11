use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::agents::{copilot_shim_digest_path, copilot_transaction_file_owned, ide_project_root};
use crate::core::DynError;
use crate::util::paths::{copilot_paths_resolved, ensure_dir, write_file_atomic};

pub struct CopilotFileSnapshot {
    pub path: String,
    pub existed: bool,
    pub data: Vec<u8>,
}

pub fn with_copilot_transaction<F>(extra_paths: &[String], f: F) -> Result<bool, DynError>
where
    F: FnOnce() -> Result<bool, DynError>,
{
    let paths = copilot_transaction_paths(extra_paths);
    let mut snapshots = Vec::with_capacity(paths.len());

    for p in &paths {
        reject_copilot_symlink_path(p)?;
        let existed = Path::new(p).exists();
        let data = if existed {
            fs::read(p).map_err(|e| -> DynError { e.to_string().into() })?
        } else {
            Vec::new()
        };
        snapshots.push(CopilotFileSnapshot {
            path: p.clone(),
            existed,
            data,
        });
    }

    match f() {
        Ok(true) => Ok(true),
        Ok(false) => {
            let _ = restore_copilot_snapshot(&snapshots);
            Ok(false)
        }
        Err(err) => {
            if let Err(rb_err) = restore_copilot_snapshot(&snapshots) {
                return Err(format!("{}; rollback failed: {}", err, rb_err).into());
            }
            Err(err)
        }
    }
}

pub fn copilot_transaction_paths(extra_paths: &[String]) -> Vec<String> {
    let p = copilot_paths_resolved();
    let root = ide_project_root();

    let mut raw = vec![
        p.settings,
        p.mcp_config,
        p.instructions,
        Path::new(&p.hooks_dir).join("tokless-rtk.json").to_string_lossy().to_string(),
        Path::new(&p.hooks_dir).join("tokless-codegraph-index.json").to_string_lossy().to_string(),
        copilot_shim_digest_path(),
    ];

    if !root.is_empty() {
        let r = Path::new(&root);
        raw.push(r.join(".vscode").join("mcp.json").to_string_lossy().to_string());
        raw.push(r.join(".vscode").join("settings.json").to_string_lossy().to_string());
        raw.push(r.join(".github").join("copilot-instructions.md").to_string_lossy().to_string());
        raw.push(r.join(".github").join("hooks").join("tokless-rtk.json").to_string_lossy().to_string());
        raw.push(r.join(".github").join("hooks").join("tokless-codegraph-index.json").to_string_lossy().to_string());
    }

    for ep in extra_paths {
        if !ep.is_empty() {
            raw.push(ep.clone());
        }
    }

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for item in raw {
        if !item.is_empty() && seen.insert(item.clone()) {
            out.push(item);
        }
    }
    out
}

pub fn reject_copilot_symlink_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    let p = Path::new(path);
    let mut acc = PathBuf::new();
    for comp in p.components() {
        acc.push(comp);
        if matches!(comp, Component::Prefix(_) | Component::RootDir) {
            continue;
        }
        if let Ok(meta) = fs::symlink_metadata(&acc) {
            if meta.file_type().is_symlink() {
                return Err(format!("refusing to follow symlink in copilot path: {}", acc.display()));
            }
        }
    }
    Ok(())
}

pub fn restore_copilot_snapshot(snapshots: &[CopilotFileSnapshot]) -> Result<(), String> {
    let mut first_err: Option<String> = None;
    for snap in snapshots {
        if let Err(e) = reject_copilot_symlink_path(&snap.path) {
            if first_err.is_none() {
                first_err = Some(e);
            }
            continue;
        }
        if snap.existed {
            if let Some(parent) = Path::new(&snap.path).parent() {
                let _ = ensure_dir(parent);
            }
            let content = String::from_utf8_lossy(&snap.data);
            if let Err(e) = write_file_atomic(&snap.path, &content) {
                if first_err.is_none() {
                    first_err = Some(e.to_string());
                }
            }
        } else if Path::new(&snap.path).exists() {
            if copilot_transaction_file_owned(&snap.path) || snap.path.ends_with(".json") || snap.path.ends_with(".md") {
                if let Err(e) = fs::remove_file(&snap.path) {
                    if first_err.is_none() {
                        first_err = Some(e.to_string());
                    }
                }
            }
        }
    }
    if let Some(err) = first_err {
        Err(err)
    } else {
        Ok(())
    }
}
