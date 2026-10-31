// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/global.rs
// created: 2026-10-30

use std::collections::HashSet;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const PATTERNS_FILE: &str = "patterns.jsonl";
pub const GOTCHAS_FILE: &str = "library_gotchas.jsonl";
pub const PREFERENCES_FILE: &str = "stack_preferences.md";
pub const PROMOTABLE_CACHE_FILE: &str = ".promotable.json";

pub const PROMOTABLE_SEED: &[&str] = &[
    "sqlalchemy", "pydantic", "fastapi", "flask", "django",
    "react", "vue", "next", "express", "vite",
    "pandas", "numpy", "torch", "tensorflow",
    "celery", "redis", "docker", "nginx",
    "pytest", "jest", "vitest",
    "alembic", "prisma", "drizzle",
    "tailwind", "webpack", "esbuild",
    "tokio", "serde", "axum", "actix-web", "actix", "diesel", "sqlx",
    "gin", "fiber", "chi", "cobra",
];

pub const GOTCHA_PREFIXES: &[&str] = &[
    "gotcha:",
    "lesson:",
    "warning:",
    "caution:",
    "pitfall:",
    "avoid:",
    "don't ",
    "do not ",
    "never ",
    "bug:",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPattern {
    pub id: String,
    pub pattern: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_project: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: String,
    #[serde(default = "default_times_encountered")]
    pub times_encountered: u64,
    pub created: String,
}

fn default_confidence() -> String {
    "medium".to_string()
}

fn default_times_encountered() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalGotcha {
    pub id: String,
    pub library: String,
    pub gotcha: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_range: Option<String>,
    pub discovered: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectedStack {
    pub tags: Vec<String>,
    pub libraries: Vec<String>,
    pub frameworks: Vec<String>,
    pub manifest_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalExport {
    pub version: String,
    pub exported_at: String,
    pub patterns: Vec<GlobalPattern>,
    pub gotchas: Vec<GlobalGotcha>,
    pub preferences: String,
}

static GLOBAL_DIR_OVERRIDE: std::sync::RwLock<Option<PathBuf>> = std::sync::RwLock::new(None);

pub fn set_global_dir_override(p: Option<PathBuf>) {
    if let Ok(mut lock) = GLOBAL_DIR_OVERRIDE.write() {
        *lock = p;
    }
}

pub fn global_dir() -> PathBuf {
    if let Ok(lock) = GLOBAL_DIR_OVERRIDE.read() {
        if let Some(ref p) = *lock {
            let _ = fs::create_dir_all(p);
            return p.clone();
        }
    }
    if let Ok(home) = env::var("PROJECTMEM_HOME") {
        if !home.trim().is_empty() {
            let p = PathBuf::from(home.trim()).join("global");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".projectmem");
    let p = base.join("global");
    let _ = fs::create_dir_all(&p);
    p
}

pub fn patterns_path() -> PathBuf {
    global_dir().join(PATTERNS_FILE)
}

pub fn gotchas_path() -> PathBuf {
    global_dir().join(GOTCHAS_FILE)
}

pub fn preferences_path() -> PathBuf {
    global_dir().join(PREFERENCES_FILE)
}

pub fn promotable_cache_path() -> PathBuf {
    global_dir().join(PROMOTABLE_CACHE_FILE)
}

pub fn read_patterns() -> Vec<GlobalPattern> {
    let p = patterns_path();
    if !p.exists() {
        return Vec::new();
    }
    let file = match File::open(&p) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(item) = serde_json::from_str::<GlobalPattern>(trimmed) {
            out.push(item);
        }
    }
    out
}

pub fn read_gotchas() -> Vec<GlobalGotcha> {
    let p = gotchas_path();
    if !p.exists() {
        return Vec::new();
    }
    let file = match File::open(&p) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(item) = serde_json::from_str::<GlobalGotcha>(trimmed) {
            out.push(item);
        }
    }
    out
}

fn write_patterns(patterns: &[GlobalPattern]) -> Result<(), String> {
    let p = patterns_path();
    let tmp = p.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&tmp)
        .map_err(|e| e.to_string())?;

    for pat in patterns {
        let line = serde_json::to_string(pat).map_err(|e| e.to_string())?;
        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&tmp, &p).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_gotchas(gotchas: &[GlobalGotcha]) -> Result<(), String> {
    let p = gotchas_path();
    let tmp = p.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&tmp)
        .map_err(|e| e.to_string())?;

    for g in gotchas {
        let line = serde_json::to_string(g).map_err(|e| e.to_string())?;
        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&tmp, &p).map_err(|e| e.to_string())?;
    Ok(())
}

fn generate_id(prefix: &str) -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}_{:012x}", prefix, nanos & 0xFFFF_FFFF_FFFF)
}

pub fn add_pattern(
    pattern: &str,
    tags: Option<Vec<String>>,
    source_project: Option<String>,
    confidence: Option<&str>,
) -> Result<GlobalPattern, String> {
    let entry = GlobalPattern {
        id: generate_id("pat"),
        pattern: pattern.trim().to_string(),
        tags: tags.unwrap_or_default(),
        source_project,
        confidence: confidence.unwrap_or("medium").to_string(),
        times_encountered: 1,
        created: Utc::now().to_rfc3339(),
    };

    let p = patterns_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|e| e.to_string())?;
    let line = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
    writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    Ok(entry)
}

pub fn add_gotcha(
    library: &str,
    gotcha: &str,
    tags: Option<Vec<String>>,
    source_project: Option<String>,
    version_range: Option<String>,
) -> Result<GlobalGotcha, String> {
    let entry = GlobalGotcha {
        id: generate_id("got"),
        library: library.trim().to_lowercase(),
        gotcha: gotcha.trim().to_string(),
        tags: tags.unwrap_or_default(),
        source_project,
        version_range,
        discovered: Utc::now().to_rfc3339(),
    };

    let p = gotchas_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|e| e.to_string())?;
    let line = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
    writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    Ok(entry)
}

pub fn remove_entry(entry_id: &str) -> bool {
    let patterns = read_patterns();
    let p_len = patterns.len();
    let filtered_p: Vec<GlobalPattern> = patterns.into_iter().filter(|p| p.id != entry_id).collect();
    if filtered_p.len() < p_len {
        let _ = write_patterns(&filtered_p);
        return true;
    }

    let gotchas = read_gotchas();
    let g_len = gotchas.len();
    let filtered_g: Vec<GlobalGotcha> = gotchas.into_iter().filter(|g| g.id != entry_id).collect();
    if filtered_g.len() < g_len {
        let _ = write_gotchas(&filtered_g);
        return true;
    }

    false
}

pub fn prune_entries(older_than_days: i64, confidence: Option<&str>) -> usize {
    let cutoff = Utc::now() - ChronoDuration::days(older_than_days);
    let mut removed = 0;

    let patterns = read_patterns();
    let p_orig = patterns.len();
    let filtered_p: Vec<GlobalPattern> = patterns
        .into_iter()
        .filter(|p| {
            if let Ok(ts) = DateTime::parse_from_rfc3339(&p.created) {
                if ts.with_timezone(&Utc) < cutoff {
                    if confidence.is_none() || confidence == Some(p.confidence.as_str()) {
                        removed += 1;
                        return false;
                    }
                }
            }
            true
        })
        .collect();
    if filtered_p.len() < p_orig {
        let _ = write_patterns(&filtered_p);
    }

    let gotchas = read_gotchas();
    let g_orig = gotchas.len();
    let filtered_g: Vec<GlobalGotcha> = gotchas
        .into_iter()
        .filter(|g| {
            if let Ok(ts) = DateTime::parse_from_rfc3339(&g.discovered) {
                if ts.with_timezone(&Utc) < cutoff {
                    removed += 1;
                    return false;
                }
            }
            true
        })
        .collect();
    if filtered_g.len() < g_orig {
        let _ = write_gotchas(&filtered_g);
    }

    removed
}

pub fn load_promotable_set() -> HashSet<String> {
    let mut set: HashSet<String> = PROMOTABLE_SEED.iter().map(|s| s.to_string()).collect();
    let cache_path = promotable_cache_path();
    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = val.get("libraries").and_then(|v| v.as_array()) {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            set.insert(s.to_lowercase());
                        }
                    }
                }
            }
        }
    }
    set
}

pub fn record_promotable_libraries(libraries: &[String]) {
    let new_libs: HashSet<String> = libraries
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if new_libs.is_empty() {
        return;
    }

    let mut existing = load_promotable_set();
    let old_count = existing.len();
    existing.extend(new_libs);
    if existing.len() == old_count {
        return;
    }

    let mut sorted: Vec<String> = existing.into_iter().collect();
    sorted.sort();

    let json_val = serde_json::json!({
        "libraries": sorted
    });
    if let Ok(s) = serde_json::to_string_pretty(&json_val) {
        let _ = fs::write(promotable_cache_path(), s);
    }
}

pub fn detect_stack(root: &Path) -> DetectedStack {
    let mut tags = HashSet::new();
    let mut libraries = HashSet::new();
    let mut frameworks = HashSet::new();
    let mut manifest_files = Vec::new();

    // Cargo.toml
    let cargo = root.join("Cargo.toml");
    if cargo.is_file() {
        tags.insert("rust".to_string());
        manifest_files.push("Cargo.toml".to_string());
        if let Ok(content) = fs::read_to_string(&cargo) {
            let mut in_deps = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("[dependencies") || trimmed.starts_with("[dev-dependencies") {
                    in_deps = true;
                    continue;
                }
                if in_deps && trimmed.starts_with('[') {
                    in_deps = false;
                    continue;
                }
                if in_deps && trimmed.contains('=') {
                    if let Some(name) = trimmed.split('=').next() {
                        let dep = name.trim().to_lowercase();
                        if !dep.is_empty() {
                            libraries.insert(dep);
                        }
                    }
                }
            }
        }
    }

    // package.json
    let pkg = root.join("package.json");
    if pkg.is_file() {
        tags.insert("javascript".to_string());
        manifest_files.push("package.json".to_string());
        if let Ok(content) = fs::read_to_string(&pkg) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                for key in ["dependencies", "devDependencies"] {
                    if let Some(obj) = val.get(key).and_then(|v| v.as_object()) {
                        for dep in obj.keys() {
                            let dep_lower = dep.to_lowercase();
                            libraries.insert(dep_lower.clone());
                            if dep_lower == "react" {
                                tags.insert("react".to_string());
                            } else if dep_lower == "vue" {
                                tags.insert("vue".to_string());
                            } else if dep_lower == "next" {
                                tags.insert("nextjs".to_string());
                            } else if dep_lower == "express" {
                                tags.insert("express".to_string());
                            } else if dep_lower == "typescript" {
                                tags.insert("typescript".to_string());
                            } else if dep_lower == "tailwindcss" {
                                tags.insert("tailwind".to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // tsconfig.json
    if root.join("tsconfig.json").is_file() {
        tags.insert("typescript".to_string());
        manifest_files.push("tsconfig.json".to_string());
    }

    // pyproject.toml
    let pyproj = root.join("pyproject.toml");
    if pyproj.is_file() {
        tags.insert("python".to_string());
        manifest_files.push("pyproject.toml".to_string());
        if let Ok(content) = fs::read_to_string(&pyproj) {
            let known = [
                "sqlalchemy", "pydantic", "fastapi", "flask", "django",
                "pytest", "typer", "click", "celery", "redis", "alembic",
            ];
            let lower = content.to_lowercase();
            for k in known {
                if lower.contains(k) {
                    tags.insert(k.to_string());
                    libraries.insert(k.to_string());
                }
            }
        }
    }

    // requirements.txt
    let reqs = root.join("requirements.txt");
    if reqs.is_file() {
        tags.insert("python".to_string());
        manifest_files.push("requirements.txt".to_string());
        if let Ok(content) = fs::read_to_string(&reqs) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('-') {
                    let dep = trimmed.split(&['=', '<', '>', '~', ';', '@', '['][..]).next().unwrap_or("").trim().to_lowercase();
                    if !dep.is_empty() {
                        libraries.insert(dep);
                    }
                }
            }
        }
    }

    // go.mod
    let gmod = root.join("go.mod");
    if gmod.is_file() {
        tags.insert("go".to_string());
        manifest_files.push("go.mod".to_string());
        if let Ok(content) = fs::read_to_string(&gmod) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("require ") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let lib = parts[1].split('/').last().unwrap_or("").to_lowercase();
                        if !lib.is_empty() {
                            libraries.insert(lib);
                        }
                    }
                }
            }
        }
    }

    // Dockerfile & docker-compose.yml
    if root.join("Dockerfile").is_file() {
        tags.insert("docker".to_string());
        manifest_files.push("Dockerfile".to_string());
    }
    if root.join("docker-compose.yml").is_file() {
        tags.insert("docker".to_string());
        manifest_files.push("docker-compose.yml".to_string());
    }

    // .github/workflows
    if root.join(".github").join("workflows").is_dir() {
        tags.insert("github-actions".to_string());
        manifest_files.push(".github/workflows".to_string());
    }

    // Framework detection from libraries
    let fw_map = [
        ("fastapi", "fastapi"),
        ("flask", "flask"),
        ("django", "django"),
        ("react", "react"),
        ("vue", "vue"),
        ("next", "nextjs"),
        ("express", "express"),
        ("actix", "actix"),
        ("axum", "axum"),
        ("gin", "gin"),
        ("fiber", "fiber"),
    ];
    for (key, fw) in fw_map {
        if libraries.iter().any(|lib| lib.contains(key)) {
            frameworks.insert(fw.to_string());
            tags.insert(fw.to_string());
        }
    }

    let mut tag_list: Vec<String> = tags.into_iter().collect();
    tag_list.sort();
    let mut lib_list: Vec<String> = libraries.into_iter().collect();
    lib_list.sort();
    let mut fw_list: Vec<String> = frameworks.into_iter().collect();
    fw_list.sort();

    record_promotable_libraries(&lib_list);

    DetectedStack {
        tags: tag_list,
        libraries: lib_list,
        frameworks: fw_list,
        manifest_files,
    }
}

fn is_gotcha_signal(summary: &str, event_type: &str, outcome: Option<&str>) -> bool {
    if event_type == "attempt" {
        return matches!(outcome, Some("failed") | Some("partial"));
    }
    if event_type == "decision" || event_type == "note" {
        let head = summary.trim_start().to_lowercase();
        return GOTCHA_PREFIXES.iter().any(|prefix| head.starts_with(prefix));
    }
    false
}

fn strings_similar(a: &str, b: &str) -> bool {
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();
    if words_a.is_empty() || words_b.is_empty() {
        return false;
    }
    let overlap = words_a.intersection(&words_b).count();
    let min_len = words_a.len().min(words_b.len());
    (overlap as f64 / min_len as f64) > 0.6
}

pub fn auto_promote_event(
    event_summary: &str,
    event_type: &str,
    project_name: &str,
    tags: Option<Vec<String>>,
    project_libraries: Option<&[String]>,
    outcome: Option<&str>,
) -> Option<GlobalGotcha> {
    if !matches!(event_type, "attempt" | "decision" | "note") {
        return None;
    }
    if !is_gotcha_signal(event_summary, event_type, outcome) {
        return None;
    }

    let summary_lower = event_summary.to_lowercase();
    let promotable = load_promotable_set();

    let mut matched_libs = Vec::new();
    for lib in &promotable {
        let pattern = format!(r"\b{}\b", regex::escape(lib));
        if let Ok(re) = Regex::new(&pattern) {
            if re.is_match(&summary_lower) {
                matched_libs.push(lib.clone());
            }
        }
    }

    if matched_libs.is_empty() {
        return None;
    }

    if let Some(proj_libs) = project_libraries {
        let mut proj_set: HashSet<String> = proj_libs.iter().map(|s| s.to_lowercase()).collect();
        let aliases = [
            ("nextjs", "next"),
            ("tailwind", "tailwindcss"),
            ("actix-web", "actix"),
        ];
        for (tag, pkg) in aliases {
            if proj_set.contains(tag) {
                proj_set.insert(pkg.to_string());
            }
            if proj_set.contains(pkg) {
                proj_set.insert(tag.to_string());
            }
        }
        matched_libs.retain(|lib| proj_set.contains(lib));
        if matched_libs.is_empty() {
            return None;
        }
    }

    matched_libs.sort_by_key(|b| std::cmp::Reverse(b.len()));
    let target_lib = matched_libs[0].clone();

    // Deduplicate against existing gotchas
    let existing = read_gotchas();
    for g in existing {
        if g.library == target_lib && strings_similar(&g.gotcha, event_summary) {
            return None;
        }
    }

    add_gotcha(
        &target_lib,
        event_summary,
        tags,
        Some(project_name.to_string()),
        None,
    ).ok()
}

pub fn get_relevant_entries(
    stack: &DetectedStack,
    filter_tags: Option<&[String]>,
) -> (Vec<GlobalPattern>, Vec<GlobalGotcha>) {
    let mut project_tags: HashSet<String> = stack.tags.iter().cloned().collect();
    let project_libs: HashSet<String> = stack.libraries.iter().cloned().collect();

    if let Some(ft) = filter_tags {
        project_tags = ft.iter().cloned().collect();
    }

    let patterns = read_patterns();
    let relevant_patterns: Vec<GlobalPattern> = patterns
        .into_iter()
        .filter(|p| {
            if p.tags.is_empty() {
                true
            } else {
                p.tags.iter().any(|t| project_tags.contains(t))
            }
        })
        .collect();

    let gotchas = read_gotchas();
    let relevant_gotchas: Vec<GlobalGotcha> = gotchas
        .into_iter()
        .filter(|g| {
            let lib = g.library.to_lowercase();
            project_libs.contains(&lib) || project_tags.contains(&lib)
        })
        .collect();

    (relevant_patterns, relevant_gotchas)
}

pub fn build_inherited_instructions(patterns: &[GlobalPattern], gotchas: &[GlobalGotcha]) -> String {
    if patterns.is_empty() && gotchas.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        "## Global Memory — Inherited Knowledge\n".to_string(),
        "The following patterns and gotchas were inherited from your global memory".to_string(),
        "(~/.projectmem/global/). They represent lessons learned from other projects.\n".to_string(),
    ];

    if !gotchas.is_empty() {
        lines.push("### Library Gotchas\n".to_string());
        for g in gotchas {
            let src = g.source_project.as_deref().map(|s| format!(" (from {})", s)).unwrap_or_default();
            let ver = g.version_range.as_deref().map(|v| format!(" [{}]", v)).unwrap_or_default();
            lines.push(format!("- **{}**{}: {}{}", g.library, ver, g.gotcha, src));
        }
        lines.push(String::new());
    }

    if !patterns.is_empty() {
        lines.push("### Cross-Project Patterns\n".to_string());
        for p in patterns {
            let src = p.source_project.as_deref().map(|s| format!(" (from {})", s)).unwrap_or_default();
            lines.push(format!("- {}{}", p.pattern, src));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

pub fn export_all() -> GlobalExport {
    let prefs = if preferences_path().exists() {
        fs::read_to_string(preferences_path()).unwrap_or_default()
    } else {
        String::new()
    };

    GlobalExport {
        version: "1.0".to_string(),
        exported_at: Utc::now().to_rfc3339(),
        patterns: read_patterns(),
        gotchas: read_gotchas(),
        preferences: prefs,
    }
}

pub fn import_all(data: &GlobalExport, merge: bool) -> (usize, usize) {
    if !merge {
        let _ = write_patterns(&data.patterns);
        let _ = write_gotchas(&data.gotchas);
        if !data.preferences.is_empty() {
            let _ = fs::write(preferences_path(), &data.preferences);
        }
        return (data.patterns.len(), data.gotchas.len());
    }

    let existing_p_ids: HashSet<String> = read_patterns().into_iter().map(|p| p.id).collect();
    let existing_g_ids: HashSet<String> = read_gotchas().into_iter().map(|g| g.id).collect();

    let mut new_p = 0;
    for p in &data.patterns {
        if !existing_p_ids.contains(&p.id) {
            let path = patterns_path();
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
                if let Ok(line) = serde_json::to_string(p) {
                    let _ = writeln!(f, "{}", line);
                    new_p += 1;
                }
            }
        }
    }

    let mut new_g = 0;
    for g in &data.gotchas {
        if !existing_g_ids.contains(&g.id) {
            let path = gotchas_path();
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
                if let Ok(line) = serde_json::to_string(g) {
                    let _ = writeln!(f, "{}", line);
                    new_g += 1;
                }
            }
        }
    }

    if !data.preferences.is_empty() && !preferences_path().exists() {
        let _ = fs::write(preferences_path(), &data.preferences);
    }

    (new_p, new_g)
}
