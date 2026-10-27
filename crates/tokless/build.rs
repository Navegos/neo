// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/build.rs
// created: 2026-10-16
// lastModified: 2026-10-27

use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn compile_builtin_filters() {
    let filters_dir = Path::new("src/rtk/filters/builtin");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo");
    let dest = Path::new(&out_dir).join("builtin_filters.toml");

    println!("cargo:rerun-if-changed=src/rtk/filters/builtin");

    if filters_dir.exists() {
        let mut files: Vec<_> = fs::read_dir(filters_dir)
            .expect("src/rtk/filters/builtin directory must exist")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
            .collect();

        files.sort_by_key(|e| e.file_name());

        let mut combined = String::from("schema_version = 1\n\n");

        for entry in &files {
            let content = fs::read_to_string(entry.path())
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", entry.path(), e));
            combined.push_str(&format!(
                "# --- {} ---\n",
                entry.file_name().to_string_lossy()
            ));
            combined.push_str(&content);
            combined.push_str("\n\n");
        }

        let parsed: toml::Value = combined.parse().unwrap_or_else(|e| {
            panic!(
                "TOML validation failed for combined filters:\n{}\n\nCheck src/rtk/filters/builtin/*.toml files",
                e
            )
        });

        if let Some(filters) = parsed.get("filters").and_then(|f| f.as_table()) {
            let mut seen: HashSet<String> = HashSet::new();
            for key in filters.keys() {
                if !seen.insert(key.clone()) {
                    panic!("Duplicate filter name '{}' found across builtin toml files", key);
                }
            }
        }

        fs::write(&dest, combined).expect("Failed to write combined builtin_filters.toml");
    } else {
        fs::write(&dest, "schema_version = 1\n\n").expect("Failed to write empty builtin_filters.toml");
    }
}

fn main() {
    compile_builtin_filters();

    #[cfg(target_os = "windows")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let major: u64 = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0);
        let minor: u64 = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0);
        let patch: u64 = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0);
        let version_u64 = (major << 48) | (minor << 32) | (patch << 16);

        let mut res = winres::WindowsResource::new();
        res.set_version_info(winres::VersionInfo::FILEVERSION, version_u64);
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, version_u64);

        res.set("CompanyName", "Navegos");
        res.set("FileDescription", env!("CARGO_PKG_DESCRIPTION"));
        res.set("FileVersion", &format!("{major}.{minor}.{patch}.0"));
        res.set("LegalCopyright", "Copyright (c) 2026 Navegos. @DevelVitorF");
        res.set("LegalTrademarks", "");
        res.set("OriginalFilename", "tokless.exe");
        res.set("ProductName", "tokless");
        res.set("ProductVersion", &format!("{major}.{minor}.{patch}.0"));
        res.set("PrivateBuild", "");
        res.set("SpecialBuild", "");

        res.set_language(0x0000); // Neutral

        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resources: {e}");
            std::process::exit(1);
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
