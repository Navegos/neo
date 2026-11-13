// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/pathsetup.rs
// created: 2026-10-06
// lastModified: 2026-11-13

use std::path::Path;
use super::paths::{home, exists, IS_WIN};
use super::exec::{run, RunOptions};
use super::logger::L;

pub fn expected_bin_dirs() -> Vec<String> {
    let h = home();
    if IS_WIN {
        let mut dirs = vec![
            Path::new(&h).join(".local").join("bin").to_string_lossy().to_string(),
            Path::new(&h).join(".cargo").join("bin").to_string_lossy().to_string(),
        ];
        if let Ok(la) = std::env::var("LOCALAPPDATA") {
            dirs.push(Path::new(&la).join("Programs").join("toksi").to_string_lossy().to_string());
        }
        dirs
    } else {
        vec![
            Path::new(&h).join(".local").join("bin").to_string_lossy().to_string(),
            Path::new(&h).join(".cargo").join("bin").to_string_lossy().to_string(),
        ]
    }
}

pub fn ensure_process_path() -> Vec<String> {
    let sep = if IS_WIN { ";" } else { ":" };
    let cur_path = std::env::var("PATH").unwrap_or_default();
    let mut in_path = std::collections::HashSet::new();
    for d in cur_path.split(sep) {
        in_path.insert(d.to_string());
    }

    let mut added = Vec::new();
    for dir in expected_bin_dirs() {
        if !in_path.contains(&dir) && exists(&dir) {
            super::exec::prepend_process_path(&dir);
            in_path.insert(dir.clone());
            added.push(dir);
        }
    }
    added
}

pub fn ensure_persistent_path() -> Vec<String> {
    if IS_WIN {
        let mut missing = Vec::new();
        for dir in expected_bin_dirs() {
            if exists(&dir) {
                missing.push(dir);
            }
        }
        persist_windows_path_dirs(&missing)
    } else {
        vec![]
    }
}

pub fn persist_windows_path_dirs(dirs: &[String]) -> Vec<String> {
    if dirs.is_empty() {
        return vec![];
    }
    let quoted: Vec<String> = dirs.iter().map(|d| format!("'{}'", d.replace('\'', "''"))).collect();
    let ps = format!(
        r#"$ErrorActionPreference='Stop'
$k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
$cur = ''
if ($null -ne $k.GetValue('Path')) {{
  $cur = $k.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
}}
$parts = $cur -split ';' | Where-Object {{ $_ -ne '' }}
$expanded = $parts | ForEach-Object {{ [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') }}
$add = @({})
$new = $parts
$changed = $false
foreach ($d in $add) {{
  if ($expanded -notcontains $d.TrimEnd('\')) {{ $new += $d; $changed = $true }}
}}
if ($changed) {{
  $k.SetValue('Path', ($new -join ';'), [Microsoft.Win32.RegistryValueKind]::ExpandString)
  Write-Output 'changed'
}}
$k.Close()"#,
        quoted.join(",")
    );

    let r = run("powershell", &["-NoProfile", "-Command", &ps], RunOptions { capture: true, ..Default::default() });
    if r.code != 0 || !r.stdout.contains("changed") {
        return vec![];
    }
    dirs.to_vec()
}

pub fn self_heal_path() {
    if std::env::var("TOKSI_TEST").as_deref() == Ok("1") {
        return;
    }
    let added = ensure_process_path();
    let patched = ensure_persistent_path();
    if added.is_empty() && patched.is_empty() {
        return;
    }
    let mut msg = Vec::new();
    if !added.is_empty() {
        msg.push("PATH updated for this session");
    }
    L.debug(&msg.join(" · "));
}
