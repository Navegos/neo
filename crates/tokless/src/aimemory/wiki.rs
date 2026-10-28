// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/aimemory/wiki.rs

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
        let dir = root_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&dir);
        Self { root_dir: dir }
    }

    fn page_path(&self, slug: &str) -> PathBuf {
        self.root_dir.join(format!("{}.md", slug))
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

