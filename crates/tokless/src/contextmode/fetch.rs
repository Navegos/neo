// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/fetch.rs
// created: 2026-10-27

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use regex::Regex;
use reqwest::header::USER_AGENT;
use url::Url;

use super::fts::ContentStore;
use super::stats::{record_cache_hit, record_cache_miss};

pub const DEFAULT_FETCH_TTL_MS: u64 = 24 * 60 * 60 * 1000; // 24 hours
pub const FETCH_PREVIEW_LIMIT: usize = 1200;

#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub url: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FetchOneOutcome {
    Cached {
        label: String,
        chunk_count: usize,
        age_str: String,
        ttl_str: String,
    },
    Fetched {
        label: String,
        total_chunks: usize,
        total_bytes: usize,
        preview: String,
    },
    Error {
        url: String,
        error: String,
    },
}

pub fn compose_fetch_cache_key(source: Option<&str>, url: &str) -> String {
    match source {
        Some(s) if !s.trim().is_empty() => format!("{}::{}", s.trim(), url),
        _ => url.to_string(),
    }
}

/// SSRF security guard. Blocks private, link-local, loopback, and metadata endpoints.
pub fn ssrf_guard(target_url: &str) -> Result<Url, String> {
    let parsed = Url::parse(target_url).map_err(|e| format!("Invalid URL: {}", e))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!("SSRF block: unsupported scheme '{}', only http/https allowed", scheme));
    }

    let host_str = match parsed.host_str() {
        Some(h) => h,
        None => return Err("SSRF block: missing hostname".to_string()),
    };

    if host_str.eq_ignore_ascii_case("localhost") {
        return Err("SSRF block: 'localhost' access is forbidden".to_string());
    }

    // Check IP addresses
    if let Ok(ip) = host_str.parse::<IpAddr>() {
        if is_private_or_restricted_ip(ip) {
            return Err(format!("SSRF block: private/internal IP address '{}' is forbidden", ip));
        }
    }

    Ok(parsed)
}

fn is_private_or_restricted_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 127.0.0.0/8 loopback
            octets[0] == 127
                // 10.0.0.0/8 private
                || octets[0] == 10
                // 172.16.0.0/12 private
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                // 192.168.0.0/16 private
                || (octets[0] == 192 && octets[1] == 168)
                // 169.254.0.0/16 link-local (IMDS)
                || (octets[0] == 169 && octets[1] == 254)
                // 0.0.0.0/8
                || octets[0] == 0
                // Broadcast
                || octets == [255, 255, 255, 255]
        }
        IpAddr::V6(v6) => {
            v6.is_loopback() || (v6.segments()[0] & 0xfe00) == 0xfc00 // Unique Local Address
        }
    }
}

/// Convert HTML content to clean Markdown text, stripping non-content tags.
pub fn html_to_markdown(html: &str) -> String {
    // 1. Strip script, style, noscript, svg, nav, footer, header
    let tags_to_strip = ["script", "style", "noscript", "svg", "nav", "footer", "header"];
    let mut clean = html.to_string();

    for tag in tags_to_strip {
        let pattern = format!(r"(?is)<{tag}[^>]*>.*?</{tag}>", tag = tag);
        if let Ok(re) = Regex::new(&pattern) {
            clean = re.replace_all(&clean, "").to_string();
        }
    }

    // Strip comments <!-- ... -->
    if let Ok(re) = Regex::new(r"(?s)<!--.*?-->") {
        clean = re.replace_all(&clean, "").to_string();
    }

    // 2. Convert headings h1-h6
    for h in 1..=6 {
        let hashes = "#".repeat(h);
        let pattern = format!(r"(?is)<h{h}[^>]*>(.*?)</h{h}>", h = h);
        if let Ok(re) = Regex::new(&pattern) {
            clean = re.replace_all(&clean, |caps: &regex::Captures| {
                format!("\n\n{} {}\n\n", hashes, &caps[1].trim())
            }).to_string();
        }
    }

    // 3. Convert links: <a href="url">text</a> -> [text](url)
    if let Ok(re) = Regex::new(r#"(?is)<a\s+[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#) {
        clean = re.replace_all(&clean, |caps: &regex::Captures| {
            let href = &caps[1];
            let text = &caps[2].trim();
            if text.is_empty() {
                href.to_string()
            } else {
                format!("[{}]({})", text, href)
            }
        }).to_string();
    }

    // 4. Convert code blocks: <pre><code>...</code></pre> or <pre>...</pre>
    if let Ok(re) = Regex::new(r"(?is)<pre[^>]*><code[^>]*>(.*?)</code></pre>") {
        clean = re.replace_all(&clean, "\n\n```\n$1\n```\n\n").to_string();
    }
    if let Ok(re) = Regex::new(r"(?is)<pre[^>]*>(.*?)</pre>") {
        clean = re.replace_all(&clean, "\n\n```\n$1\n```\n\n").to_string();
    }
    if let Ok(re) = Regex::new(r"(?is)<code[^>]*>(.*?)</code>") {
        clean = re.replace_all(&clean, "`$1`").to_string();
    }

    // 5. Convert lists: <li>...</li> -> * ...
    if let Ok(re) = Regex::new(r"(?is)<li[^>]*>(.*?)</li>") {
        clean = re.replace_all(&clean, "\n* $1").to_string();
    }

    // 6. Paragraphs and breaks
    if let Ok(re) = Regex::new(r"(?is)<p[^>]*>(.*?)</p>") {
        clean = re.replace_all(&clean, "\n\n$1\n\n").to_string();
    }
    if let Ok(re) = Regex::new(r"(?is)<br\s*/?>") {
        clean = re.replace_all(&clean, "\n").to_string();
    }

    // 7. Strip all remaining HTML tags
    if let Ok(re) = Regex::new(r"<[^>]+>") {
        clean = re.replace_all(&clean, "").to_string();
    }

    // 8. Decode HTML entities
    clean = clean
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");

    // 9. Normalize whitespace and blank lines
    let mut normalized = String::with_capacity(clean.len());
    let mut consecutive_newlines = 0;

    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                normalized.push('\n');
            }
        } else {
            consecutive_newlines = 0;
            normalized.push_str(trimmed);
            normalized.push('\n');
        }
    }

    normalized.trim().to_string()
}

pub struct WebFetcher {
    client: reqwest::blocking::Client,
}

impl Default for WebFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl WebFetcher {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self { client }
    }

    pub fn fetch_one(
        &self,
        store: &mut ContentStore,
        req: &FetchRequest,
        force: bool,
        ttl_ms: Option<u64>,
    ) -> FetchOneOutcome {
        let target_url = &req.url;
        if let Err(e) = ssrf_guard(target_url) {
            return FetchOneOutcome::Error {
                url: target_url.clone(),
                error: e,
            };
        }

        let cache_key = compose_fetch_cache_key(req.source.as_deref(), target_url);
        let effective_ttl = ttl_ms.unwrap_or(DEFAULT_FETCH_TTL_MS);

        // Check cache unless forced or ttl is 0
        if !force && effective_ttl > 0 {
            if let Some(meta) = store.get_source_meta(&cache_key) {
                if let Ok(parsed_time) = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", meta.indexed_at.replace(' ', "T"))) {
                    let now = chrono::Utc::now();
                    let age_ms = (now - parsed_time.to_utc()).num_milliseconds();
                    if age_ms >= 0 && (age_ms as u64) < effective_ttl {
                        let age_min = age_ms / (60 * 1000);
                        let age_hours = age_min / 60;
                        let age_str = if age_hours > 0 {
                            format!("{}h ago", age_hours)
                        } else if age_min > 0 {
                            format!("{}m ago", age_min)
                        } else {
                            "just now".to_string()
                        };
                        let ttl_str = format!("{}h", effective_ttl / (3600 * 1000));
                        let estimated_bytes = meta.chunk_count * 1600;
                        record_cache_hit(estimated_bytes);

                        return FetchOneOutcome::Cached {
                            label: meta.label,
                            chunk_count: meta.chunk_count,
                            age_str,
                            ttl_str,
                        };
                    }
                }
            }
        }

        record_cache_miss();

        // Perform HTTP GET
        let response = match self
            .client
            .get(target_url)
            .header(USER_AGENT, "Mozilla/5.0 (compatible; tokless-contextmode/1.0)")
            .send()
        {
            Ok(r) => r,
            Err(e) => {
                return FetchOneOutcome::Error {
                    url: target_url.clone(),
                    error: format!("HTTP request failed: {}", e),
                };
            }
        };

        if !response.status().is_success() {
            return FetchOneOutcome::Error {
                url: target_url.clone(),
                error: format!("HTTP error: {}", response.status()),
            };
        }

        let content_type_header = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let raw_body = match response.text() {
            Ok(t) => t,
            Err(e) => {
                return FetchOneOutcome::Error {
                    url: target_url.clone(),
                    error: format!("Failed to read response body: {}", e),
                };
            }
        };

        if raw_body.trim().is_empty() {
            return FetchOneOutcome::Error {
                url: target_url.clone(),
                error: "Empty response body".to_string(),
            };
        }

        let is_json = content_type_header.contains("application/json") || raw_body.trim_start().starts_with('{') || raw_body.trim_start().starts_with('[');
        let is_html = content_type_header.contains("text/html") || raw_body.contains("<html") || raw_body.contains("<!DOCTYPE");

        let (processed_content, ctype) = if is_json {
            (raw_body, Some("json"))
        } else if is_html {
            (html_to_markdown(&raw_body), Some("markdown"))
        } else {
            (raw_body, Some("prose"))
        };

        let total_bytes = processed_content.len();
        let preview = if processed_content.len() > FETCH_PREVIEW_LIMIT {
            let mut cut = processed_content.chars().take(FETCH_PREVIEW_LIMIT).collect::<String>();
            cut.push_str("\n\n... [truncated — use ctx_search() for full content]");
            cut
        } else {
            processed_content.clone()
        };

        let index_res = match store.index(&processed_content, &cache_key, ctype, None) {
            Ok(ir) => ir,
            Err(e) => {
                return FetchOneOutcome::Error {
                    url: target_url.clone(),
                    error: format!("Failed to index content: {}", e),
                };
            }
        };

        FetchOneOutcome::Fetched {
            label: index_res.label,
            total_chunks: index_res.total_chunks,
            total_bytes,
            preview,
        }
    }

    pub fn fetch_batch(
        &self,
        store: &mut ContentStore,
        requests: &[FetchRequest],
        concurrency: usize,
        force: bool,
        ttl_ms: Option<u64>,
    ) -> Vec<FetchOneOutcome> {
        if requests.is_empty() {
            return Vec::new();
        }

        // To ensure thread-safe single writer SQLite, we fetch content in parallel,
        // then index serially into ContentStore.
        let pool_size = concurrency.clamp(1, 8).min(requests.len());

        let mut results = Vec::with_capacity(requests.len());
        // For simple robust parallel fetching, chunk by pool size or process with threads
        let client_arc = Arc::new(self.client.clone());

        for req in requests {
            // First check if cached in store
            let cache_key = compose_fetch_cache_key(req.source.as_deref(), &req.url);
            let effective_ttl = ttl_ms.unwrap_or(DEFAULT_FETCH_TTL_MS);
            let mut cached_outcome = None;

            if !force && effective_ttl > 0 {
                if let Some(meta) = store.get_source_meta(&cache_key) {
                    if let Ok(parsed_time) = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", meta.indexed_at.replace(' ', "T"))) {
                        let now = chrono::Utc::now();
                        let age_ms = (now - parsed_time.to_utc()).num_milliseconds();
                        if age_ms >= 0 && (age_ms as u64) < effective_ttl {
                            let age_min = age_ms / (60 * 1000);
                            let age_hours = age_min / 60;
                            let age_str = if age_hours > 0 {
                                format!("{}h ago", age_hours)
                            } else if age_min > 0 {
                                format!("{}m ago", age_min)
                            } else {
                                "just now".to_string()
                            };
                            let ttl_str = format!("{}h", effective_ttl / (3600 * 1000));
                            record_cache_hit(meta.chunk_count * 1600);
                            cached_outcome = Some(FetchOneOutcome::Cached {
                                label: meta.label,
                                chunk_count: meta.chunk_count,
                                age_str,
                                ttl_str,
                            });
                        }
                    }
                }
            }

            if let Some(cached) = cached_outcome {
                results.push(Some(cached));
            } else {
                record_cache_miss();
                results.push(None);
            }
        }

        // Fetch uncached items
        let uncached_indices: Vec<usize> = results
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_none())
            .map(|(i, _)| i)
            .collect();

        if !uncached_indices.is_empty() {
            let chunks: Vec<Vec<usize>> = uncached_indices
                .chunks(pool_size.max(1))
                .map(|c| c.to_vec())
                .collect();

            for chunk_indices in chunks {
                let mut handles = Vec::new();
                for idx in chunk_indices {
                    let req = requests[idx].clone();
                    let client = Arc::clone(&client_arc);

                    handles.push((
                        idx,
                        std::thread::spawn(move || {
                            if let Err(e) = ssrf_guard(&req.url) {
                                return Err(e);
                            }
                            let resp = client
                                .get(&req.url)
                                .header(USER_AGENT, "Mozilla/5.0 (compatible; tokless-contextmode/1.0)")
                                .send()
                                .map_err(|e| format!("HTTP request error: {}", e))?;

                            if !resp.status().is_success() {
                                return Err(format!("HTTP status error: {}", resp.status()));
                            }

                            let ctype = resp
                                .headers()
                                .get(reqwest::header::CONTENT_TYPE)
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("")
                                .to_string();

                            let body = resp.text().map_err(|e| format!("Body read error: {}", e))?;
                            Ok((body, ctype))
                        }),
                    ));
                }

                for (idx, handle) in handles {
                    let req = &requests[idx];
                    let cache_key = compose_fetch_cache_key(req.source.as_deref(), &req.url);

                    let thread_res = match handle.join() {
                        Ok(res) => res,
                        Err(_) => Err("Fetch thread panicked".to_string()),
                    };

                    match thread_res {
                        Ok((raw_body, content_type_header)) => {
                            let is_json = content_type_header.contains("application/json") || raw_body.trim_start().starts_with('{') || raw_body.trim_start().starts_with('[');
                            let is_html = content_type_header.contains("text/html") || raw_body.contains("<html") || raw_body.contains("<!DOCTYPE");

                            let (processed_content, ctype) = if is_json {
                                (raw_body, Some("json"))
                            } else if is_html {
                                (html_to_markdown(&raw_body), Some("markdown"))
                            } else {
                                (raw_body, Some("prose"))
                            };

                            let total_bytes = processed_content.len();
                            let preview = if processed_content.len() > FETCH_PREVIEW_LIMIT {
                                let mut cut = processed_content.chars().take(FETCH_PREVIEW_LIMIT).collect::<String>();
                                cut.push_str("\n\n... [truncated — use ctx_search() for full content]");
                                cut
                            } else {
                                processed_content.clone()
                            };

                            match store.index(&processed_content, &cache_key, ctype, None) {
                                Ok(ir) => {
                                    results[idx] = Some(FetchOneOutcome::Fetched {
                                        label: ir.label,
                                        total_chunks: ir.total_chunks,
                                        total_bytes,
                                        preview,
                                    });
                                }
                                Err(e) => {
                                    results[idx] = Some(FetchOneOutcome::Error {
                                        url: req.url.clone(),
                                        error: format!("Index error: {}", e),
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            results[idx] = Some(FetchOneOutcome::Error {
                                url: req.url.clone(),
                                error: e,
                            });
                        }
                    }
                }
            }
        }

        results.into_iter().flatten().collect()
    }
}
