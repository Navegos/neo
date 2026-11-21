// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/validator/shard_builder.rs
// created: 2026-11-21

//! Symmetrical 3-tree GitHub Pages shard builder (`advisories/`, `domains/` <= 500 rows/page,
//! `lookup/` prefix maps + paired `index.json` / `index.html`) for `Navegos/advisory`.

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::util::paths::write_file_atomic;

pub const MAX_ROWS_PER_PAGE: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactAdvisoryRow {
    pub id: String,
    pub taxonomy: String,
    pub year: String,
    pub severity: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShardBuildReport {
    pub advisories_written: usize,
    pub domain_pages_written: usize,
    pub lookup_shards_written: usize,
}

fn write_paired_index(dir: &Path, title: &str, items: &[String]) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("Failed to create shard dir '{}': {e}", dir.display()))?;
    let json_val = json!({ "title": title, "entries": items });
    let json_str = serde_json::to_string_pretty(&json_val)
        .map_err(|e| format!("Failed to serialize index.json: {e}"))?;
    write_file_atomic(&dir.join("index.json"), &json_str)
        .map_err(|e| format!("Failed to write index.json: {e}"))?;

    let list_html: String = items
        .iter()
        .map(|it| format!("<li><a href=\"{it}\">{it}</a></li>"))
        .collect::<Vec<_>>()
        .join("\n");
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head><body><h1>{title}</h1><ul>{list_html}</ul></body></html>"
    );
    write_file_atomic(&dir.join("index.html"), &html)
        .map_err(|e| format!("Failed to write index.html: {e}"))?;
    Ok(())
}

/// Builds the symmetrical 3-tree GitHub Pages shard layout under `advisory_root`.
pub fn build_three_tree_shards(
    advisory_root: &Path,
    rows: &[CompactAdvisoryRow],
) -> Result<ShardBuildReport, String> {
    write_file_atomic(&advisory_root.join(".nojekyll"), "")
        .map_err(|e| format!("Failed to write .nojekyll: {e}"))?;
    write_paired_index(
        advisory_root,
        "Navegos/advisory",
        &[
            "advisories/".to_string(),
            "domains/".to_string(),
            "lookup/".to_string(),
        ],
    )?;

    let mut by_tax_year: BTreeMap<(String, String), Vec<&CompactAdvisoryRow>> = BTreeMap::new();
    for r in rows {
        by_tax_year
            .entry((r.taxonomy.clone(), r.year.clone()))
            .or_default()
            .push(r);
    }

    let mut advisories_written = 0usize;
    let mut domain_pages_written = 0usize;
    let mut lookup_shards_written = 0usize;

    let mut tax_names: Vec<String> = by_tax_year
        .keys()
        .map(|(t, _)| format!("{t}/"))
        .collect();
    tax_names.sort();
    tax_names.dedup();
    write_paired_index(&advisory_root.join("advisories"), "Advisories", &tax_names)?;
    write_paired_index(&advisory_root.join("domains"), "Domains", &tax_names)?;
    write_paired_index(
        &advisory_root.join("lookup"),
        "Lookup",
        &["_global/".to_string()],
    )?;

    let mut lookup_groups: BTreeMap<(String, String, String), Vec<&CompactAdvisoryRow>> =
        BTreeMap::new();

    for ((tax, year), group) in &by_tax_year {
        // Tree 1: advisories/<taxonomy>/<year>/<ID>/<ID>.json
        let adv_year_dir = advisory_root.join("advisories").join(tax).join(year);
        let mut id_dirs = Vec::new();
        for row in group {
            let id_dir = adv_year_dir.join(&row.id);
            write_paired_index(&id_dir, &row.id, &[format!("{}.json", row.id)])?;
            let adv_json = serde_json::to_string_pretty(row)
                .map_err(|e| format!("Failed to serialize advisory row: {e}"))?;
            write_file_atomic(&id_dir.join(format!("{}.json", row.id)), &adv_json)
                .map_err(|e| format!("Failed to write advisory JSON: {e}"))?;
            id_dirs.push(format!("{}/", row.id));
            advisories_written += 1;

            let kind = if row.id.starts_with("CVE-") {
                "cve"
            } else if row.id.starts_with("GHSA-") {
                "ghsa"
            } else if row.id.starts_with("TOKSI-") {
                "toksi"
            } else {
                "vuln"
            };
            let prefix: String = row.id.chars().take(8).collect();
            lookup_groups
                .entry((kind.to_string(), year.clone(), prefix))
                .or_default()
                .push(row);
        }
        write_paired_index(&adv_year_dir, &format!("{tax}/{year}"), &id_dirs)?;

        // Tree 2: domains/<taxonomy>/<year>/page-NNN.json (<= 500 rows/page)
        let dom_year_dir = advisory_root.join("domains").join(tax).join(year);
        let mut page_files = Vec::new();
        for (page_idx, chunk) in group.chunks(MAX_ROWS_PER_PAGE).enumerate() {
            let page_name = format!("page-{:03}.json", page_idx + 1);
            let page_json = serde_json::to_string_pretty(&chunk)
                .map_err(|e| format!("Failed to serialize domain page: {e}"))?;
            write_file_atomic(&dom_year_dir.join(&page_name), &page_json)
                .map_err(|e| format!("Failed to write domain page: {e}"))?;
            page_files.push(page_name);
            domain_pages_written += 1;
        }
        write_paired_index(&dom_year_dir, &format!("domains/{tax}/{year}"), &page_files)?;
    }

    // Tree 3: lookup/_global/{cve,vuln,ghsa,toksi}/<year>/<prefix>.json
    for ((kind, year, prefix), entries) in &lookup_groups {
        let dir = advisory_root
            .join("lookup")
            .join("_global")
            .join(kind)
            .join(year);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create lookup dir: {e}"))?;
        let fname = format!("{prefix}.json");
        let body = serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("Failed to serialize lookup shard: {e}"))?;
        write_file_atomic(&dir.join(&fname), &body)
            .map_err(|e| format!("Failed to write lookup shard: {e}"))?;
        write_paired_index(&dir, &format!("lookup/{kind}/{year}"), &[fname])?;
        lookup_shards_written += 1;
    }

    Ok(ShardBuildReport {
        advisories_written,
        domain_pages_written,
        lookup_shards_written,
    })
}
