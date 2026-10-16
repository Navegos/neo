// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/build.rs
// created: 2026-10-16
// lastModified: 2026-10-16

fn main() {
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

