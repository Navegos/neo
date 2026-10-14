// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/pathsetup.rs
// created: 2026-10-06
// lastModified: 2026-10-14

use std::path::Path;
use super::paths::{home, exists, IS_WIN};
use super::exec::{run, RunOptions};
use super::logger::L;

fn node_install_dir() -> String {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        Path::new(&home()).join("AppData").join("Local").to_string_lossy().to_string()
    });
    Path::new(&base).join("tokless").join("node").to_string_lossy().to_string()
}

pub fn expected_bin_dirs() -> Vec<String> {
    let h = home();
    if IS_WIN {
        let mut dirs = vec![
            Path::new(&h).join(".local").join("bin").to_string_lossy().to_string(),
            Path::new(&h).join(".bun").join("bin").to_string_lossy().to_string(),
        ];
        if let Ok(la) = std::env::var("LOCALAPPDATA") {
            dirs.push(Path::new(&la).join("Programs").join("tokless").to_string_lossy().to_string());
        }
        dirs
    } else {
        vec![
            Path::new(&h).join(".local").join("bin").to_string_lossy().to_string(),
            Path::new(&h).join(".bun").join("bin").to_string_lossy().to_string(),
            Path::new(&h).join(".cargo").join("bin").to_string_lossy().to_string(),
        ]
    }
}

fn runtime_bin_dirs() -> Vec<String> {
    if !IS_WIN {
        return vec![];
    }
    let mut dirs = vec![node_install_dir()];
    if let Ok(ad) = std::env::var("APPDATA") {
        dirs.push(Path::new(&ad).join("npm").to_string_lossy().to_string());
    }
    if let Ok(pf) = std::env::var("ProgramFiles") {
        dirs.push(Path::new(&pf).join("nodejs").to_string_lossy().to_string());
    }
    if let Ok(la) = std::env::var("LOCALAPPDATA") {
        dirs.push(Path::new(&la).join("Programs").join("nodejs").to_string_lossy().to_string());
    }
    dirs
}

pub fn ensure_process_path() -> Vec<String> {
    let sep = if IS_WIN { ";" } else { ":" };
    let cur_path = std::env::var("PATH").unwrap_or_default();
    let mut current: Vec<String> = cur_path.split(sep).map(|s| s.to_string()).collect();
    let mut in_path = std::collections::HashSet::new();
    for d in &current {
        in_path.insert(d.clone());
    }

    let mut added = Vec::new();
    let mut all_expected = expected_bin_dirs();
    all_expected.extend(runtime_bin_dirs());

    for dir in all_expected {
        if !in_path.contains(&dir) && exists(&dir) {
            current.insert(0, dir.clone());
            in_path.insert(dir.clone());
            added.push(dir);
        }
    }

    if !added.is_empty() {
        unsafe {
            std::env::set_var("PATH", current.join(sep));
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
    if std::env::var("TOKLESS_TEST").as_deref() == Ok("1") {
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
