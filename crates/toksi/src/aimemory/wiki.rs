// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/wiki.rs
// created: 2026-10-28
// lastModified: 2026-11-08

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::aimemory::tiered::TieredEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub slug: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub summary_text: Option<String>,
    pub tags: Vec<String>,
    pub body: String,
}

pub struct WikiStore {
    root_dir: PathBuf,
}

impl WikiStore {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        let norm = crate::util::paths::normalize_path(&root_dir.as_ref().to_string_lossy());
        let dir = PathBuf::from(norm);
        let _ = fs::create_dir_all(&dir);
        Self { root_dir: dir }
    }

    pub fn open_default() -> Self {
        Self::new(crate::aimemory::default_wiki_dir())
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn page_path(&self, slug: &str) -> PathBuf {
        let norm_slug = crate::util::paths::normalize_path(slug);
        let cleaned_slug = norm_slug.trim_matches('/').replace('/', "_");
        self.root_dir.join(format!("{}.md", cleaned_slug))
    }

    pub fn write_page(&self, page: &WikiPage) -> Result<(), String> {
        let path = self.page_path(&page.slug);
        let content = format!(
            "---\ntitle: {}\nabstract: {}\nsummary: {}\ntags: {}\n---\n\n{}",
            page.title,
            page.abstract_text.as_deref().unwrap_or(""),
            page.summary_text.as_deref().unwrap_or(""),
            page.tags.join(", "),
            page.body
        );
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn read_page(&self, slug: &str) -> Result<Option<WikiPage>, String> {
        let path = self.page_path(slug);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        Ok(Some(parse_markdown_page(slug, &content)))
    }

    pub fn list_pages(&self) -> Result<Vec<WikiPage>, String> {
        let mut pages = Vec::new();
        if !self.root_dir.exists() {
            return Ok(pages);
        }

        let entries = fs::read_dir(&self.root_dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                if let Ok(content) = fs::read_to_string(&path) {
                    pages.push(parse_markdown_page(&slug, &content));
                }
            }
        }
        pages.sort_by(|a, b| a.slug.cmp(&b.slug));
        Ok(pages)
    }

    pub fn to_tiered_entries(&self) -> Result<Vec<TieredEntry>, String> {
        let pages = self.list_pages()?;
        Ok(pages
            .into_iter()
            .map(|p| TieredEntry {
                key: p.slug,
                title: p.title,
                abstract_text: p.abstract_text,
                summary_text: p.summary_text,
                full_content: p.body,
            })
            .collect())
    }

    pub fn delete_page(&self, slug: &str) -> Result<bool, String> {
        let path = self.page_path(slug);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Dynamic purge: delete wiki pages referencing or generated from ignored files/directories
    pub fn purge_ignored_pages(&self, root: &Path) -> Result<usize, String> {
        let ignore = crate::codegraph::walker::build_project_ignore(root);
        let pages = self.list_pages()?;
        let mut purged = 0;

        for page in pages {
            let norm_slug = crate::util::paths::normalize_path(&page.slug);
            let restored_path = norm_slug.replace('_', "/");
            let full = if Path::new(&norm_slug).is_absolute() {
                PathBuf::from(&norm_slug)
            } else {
                root.join(&restored_path)
            };
            let is_dir = full.is_dir();
            let is_ignored = ignore.matched_path_or_any_parents(&full, is_dir).is_ignore()
                || crate::codegraph::walker::is_build_or_vendor_artifact(&norm_slug)
                || crate::codegraph::walker::is_build_or_vendor_artifact(&restored_path)
                || restored_path.split('/').any(crate::codegraph::walker::is_build_or_vendor_artifact)
                || norm_slug.split('/').any(crate::codegraph::walker::is_build_or_vendor_artifact);

            if is_ignored && self.delete_page(&page.slug)? {
                purged += 1;
            }
        }
        Ok(purged)
    }
}

fn parse_markdown_page(slug: &str, content: &str) -> WikiPage {
    let mut title = slug.to_string();
    let mut abstract_text = None;
    let mut summary_text = None;
    let mut tags = Vec::new();
    let mut body = content.to_string();

    if content.starts_with("---") {
        if let Some(end_idx) = content[3..].find("---") {
            let frontmatter = &content[3..3 + end_idx];
            body = content[3 + end_idx + 3..].trim_start().to_string();

            for line in frontmatter.lines() {
                let trimmed = line.trim();
                if let Some(stripped) = trimmed.strip_prefix("title:") {
                    title = stripped.trim().to_string();
                } else if let Some(stripped) = trimmed.strip_prefix("abstract:") {
                    let s = stripped.trim();
                    if !s.is_empty() {
                        abstract_text = Some(s.to_string());
                    }
                } else if let Some(stripped) = trimmed.strip_prefix("summary:") {
                    let s = stripped.trim();
                    if !s.is_empty() {
                        summary_text = Some(s.to_string());
                    }
                } else if let Some(stripped) = trimmed.strip_prefix("tags:") {
                    tags = stripped
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
            }
        }
    }

    WikiPage {
        slug: slug.to_string(),
        title,
        abstract_text,
        summary_text,
        tags,
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wiki_purge_ignored_pages() {
        let temp_dir = std::env::temp_dir().join("toksi_wiki_purge_test");
        let _ = fs::create_dir_all(&temp_dir);
        let store = WikiStore::new(temp_dir.join("wiki"));

        let page1 = WikiPage {
            slug: "src/engine.rs".to_string(),
            title: "Engine Page".to_string(),
            abstract_text: None,
            summary_text: None,
            tags: Vec::new(),
            body: "docs".to_string(),
        };

        let page2 = WikiPage {
            slug: "target/temp_out.md".to_string(),
            title: "Temp Out".to_string(),
            abstract_text: None,
            summary_text: None,
            tags: Vec::new(),
            body: "temp docs".to_string(),
        };

        store.write_page(&page1).unwrap();
        store.write_page(&page2).unwrap();
        assert_eq!(store.list_pages().unwrap().len(), 2);

        // Add gitignore
        let gitignore = temp_dir.join(".gitignore");
        fs::write(&gitignore, "target/\n").unwrap();

        let purged = store.purge_ignored_pages(&temp_dir).unwrap();
        assert_eq!(purged, 1);
        let remaining = store.list_pages().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].slug, "src_engine.rs");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

