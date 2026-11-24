// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/skills_repo.rs
// created: 2026-11-16
// lastModified: 2026-11-18

//! Externalized `<skills>` (`Navegos/skills`) & staged `<neo>/rules/`, `<neo>/agents/`
//! repository loader and synchronizer (Zero Binary Bloat Policy).
//!
//! Reads `[repositories]` from `~/.toksi/config.toml` (falling back to workspace
//! ancestors or `CARGO_MANIFEST_DIR`) so zero `.md`/`.yaml`/`.json`/`.toml` skill
//! or rule files are compiled into the `toksi` binary via `include_str!`.

#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::rules::skill_validator::{analyze_content, heal_skill_content, FileKind};
use crate::rules::skills_catalog::{SkillOrigin, SkillsCatalog};
use crate::rules::{has_enforce_guard, wrap_enforce_guard, RulesError};
use crate::util::paths::{home, normalize_path};
use crate::util::ports::toksi_config_toml_path;

pub const DEFAULT_SKILLS_REMOTE: &str = "https://github.com/Navegos/skills.git";
pub const DEFAULT_SKILLS_PAGES_URL: &str = "https://navegos.github.io/skills/";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
struct ConfigRepositoriesToml {
    repositories: RepositoriesSection,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
struct RepositoriesSection {
    skills_dir: Option<String>,
    skills_remote: Option<String>,
    skills_pages_url: Option<String>,
    user_skills_dir: Option<String>,
    stage_dir_outside_crates: Option<String>,
    neo_dir: Option<String>,
}

/// Resolved externalized repository & staging paths for `<neo>` and `<skills>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillsRepoConfig {
    pub neo_root: Option<PathBuf>,
    pub stage_rules_dir: PathBuf,
    pub stage_agents_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub user_skills_dir: PathBuf,
    pub skills_remote: String,
    pub skills_pages_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkillsRepoSyncReport {
    pub synced_files: Vec<PathBuf>,
    pub healed_files: Vec<PathBuf>,
    pub blocked_files: Vec<(String, String)>,
    pub catalog_count: usize,
    pub collision_count: usize,
}

fn expand_tilde_path(raw: &str, neo_root: Option<&Path>) -> PathBuf {
    let mut s = raw.trim().to_string();
    if let Some(root) = neo_root {
        let root_str = normalize_path(&root.to_string_lossy());
        s = s.replace("<neo>", &root_str);
    }
    if let Some(rest) = s.strip_prefix("~/").or_else(|| s.strip_prefix("~\\")) {
        let h = home();
        if !h.is_empty() {
            return PathBuf::from(normalize_path(&format!("{}/{}", h.trim_end_matches('/'), rest)));
        }
    }
    PathBuf::from(normalize_path(&s))
}

fn has_staged_assets(dir: &Path) -> bool {
    dir.join("rules").join("principles.md").is_file()
        || dir.join("agents").join("skills").join("principles").join("SKILL.md").is_file()
        || dir.join("inspect-vault").join("skills").join("rules").join("principles.md").is_file()
}

/// Locates `<neo>` workspace root by walking environment hints, `current_dir()`,
/// `current_exe()` ancestors, and compile-time `CARGO_MANIFEST_DIR`.
pub fn detect_neo_root() -> Option<PathBuf> {
    if let Ok(env_root) = std::env::var("TOKSI_NEO_ROOT") {
        let p = PathBuf::from(normalize_path(&env_root));
        if has_staged_assets(&p) {
            return Some(p);
        }
    }

    let mut seeds: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        seeds.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        seeds.push(exe);
    }
    if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
        seeds.push(PathBuf::from(manifest));
    }

    for seed in seeds {
        let mut cur = if seed.is_file() {
            seed.parent().map(Path::to_path_buf).unwrap_or(seed)
        } else {
            seed
        };
        loop {
            if has_staged_assets(&cur) {
                return Some(PathBuf::from(normalize_path(&cur.to_string_lossy())));
            }
            if !cur.pop() {
                break;
            }
        }
    }
    None
}

impl SkillsRepoConfig {
    /// Loads repository configuration from `~/.toksi/config.toml` `[repositories]`
    /// combined with detected `<neo>` staging directories.
    pub fn load() -> Self {
        let neo_root = detect_neo_root();
        let toml_cfg = fs::read_to_string(toksi_config_toml_path())
            .ok()
            .and_then(|raw| toml::from_str::<ConfigRepositoriesToml>(&raw).ok())
            .unwrap_or_default()
            .repositories;
        Self::from_repositories_section(toml_cfg, neo_root)
    }

    fn from_repositories_section(toml_cfg: RepositoriesSection, neo_root: Option<PathBuf>) -> Self {
        let effective_neo = toml_cfg
            .neo_dir
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|p| expand_tilde_path(p, neo_root.as_deref()))
            .or(neo_root);

        let home_dir = home();
        let default_user_skills = if home_dir.is_empty() {
            PathBuf::from(".toksi/user-skills")
        } else {
            PathBuf::from(normalize_path(&format!(
                "{}/.toksi/user-skills",
                home_dir.trim_end_matches('/')
            )))
        };

        let stage_agents_dir = toml_cfg
            .stage_dir_outside_crates
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|p| expand_tilde_path(p, effective_neo.as_deref()))
            .unwrap_or_else(|| {
                effective_neo
                    .as_ref()
                    .map(|r| r.join("agents"))
                    .unwrap_or_else(|| PathBuf::from("agents"))
            });

        let stage_rules_dir = effective_neo
            .as_ref()
            .map(|r| r.join("rules"))
            .unwrap_or_else(|| PathBuf::from("rules"));

        let skills_dir = toml_cfg
            .skills_dir
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|p| expand_tilde_path(p, effective_neo.as_deref()))
            .unwrap_or_else(|| {
                effective_neo
                    .as_ref()
                    .map(|r| r.join("inspect-vault").join("skills"))
                    .unwrap_or_else(|| PathBuf::from("inspect-vault/skills"))
            });

        let user_skills_dir = toml_cfg
            .user_skills_dir
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|p| expand_tilde_path(p, effective_neo.as_deref()))
            .unwrap_or(default_user_skills);

        let skills_remote = toml_cfg
            .skills_remote
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SKILLS_REMOTE.to_string());
        let skills_pages_url = toml_cfg
            .skills_pages_url
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SKILLS_PAGES_URL.to_string());

        Self {
            neo_root: effective_neo,
            stage_rules_dir,
            stage_agents_dir,
            skills_dir,
            user_skills_dir,
            skills_remote,
            skills_pages_url,
        }
    }

    fn rule_search_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        roots.push(self.stage_rules_dir.clone());
        roots.push(self.skills_dir.join("rules"));
        let h = home();
        if !h.is_empty() {
            roots.push(PathBuf::from(normalize_path(&format!(
                "{}/.toksi/rules",
                h.trim_end_matches('/')
            ))));
        }
        roots
    }

    fn skill_search_roots(&self) -> Vec<PathBuf> {
        vec![
            self.user_skills_dir.clone(),
            self.skills_dir.join("skills"),
            self.stage_agents_dir.join("skills"),
        ]
    }
}

fn read_first_matching(candidates: &[PathBuf]) -> Option<String> {
    for path in candidates {
        if path.is_file()
            && let Ok(raw) = fs::read_to_string(path)
        {
            return Some(raw.replace("\r\n", "\n"));
        }
    }
    None
}

/// Reads an externalized rule markdown document (`principles`, `caveman`, `ponytail`)
/// from `<neo>/rules/<id>.md` or `<skills>/rules/<id>.md`.
pub fn read_rule_markdown(id: &str) -> String {
    let cfg = SkillsRepoConfig::load();
    let file_name = format!("{}.md", id.trim());
    let candidates: Vec<PathBuf> = cfg
        .rule_search_roots()
        .into_iter()
        .map(|r| r.join(&file_name))
        .collect();
    read_first_matching(&candidates).unwrap_or_else(|| default_rule_fallback(id))
}

/// Reads an externalized Cursor `.mdc` rule (`principles.mdc`, `caveman.mdc`, `ponytail.mdc`).
pub fn read_cursor_rule(id: &str) -> String {
    let cfg = SkillsRepoConfig::load();
    let file_name = format!("{}.mdc", id.trim());
    let candidates: Vec<PathBuf> = cfg
        .rule_search_roots()
        .into_iter()
        .map(|r| r.join(&file_name))
        .collect();
    read_first_matching(&candidates).unwrap_or_else(|| {
        format!(
            "---\ndescription: Toksi {} rule.\nalwaysApply: true\n---\n{}\n",
            id.trim(),
            read_rule_markdown(id)
        )
    })
}

/// Reads an externalized skill `SKILL.md` from `user_skills_dir`, `<skills>/skills/`, or `<neo>/agents/skills/`.
pub fn read_skill_markdown(skill_dir_name: &str) -> String {
    let cfg = SkillsRepoConfig::load();
    let name = skill_dir_name.trim();
    let mut candidates = Vec::new();
    for root in cfg.skill_search_roots() {
        candidates.push(root.join(name).join("SKILL.md"));
        candidates.push(root.join(name).join("skill.md"));
    }
    read_first_matching(&candidates).unwrap_or_else(|| {
        format!(
            "---\nname: {0}\ndescription: Toksi {0} skill loaded via externalized skills repository.\nlicense: Apache-2.0\n---\n\n<enforce>\n# {0}\n\nExternalized skill payload for `{0}`.\n</enforce>\n",
            name
        )
    })
}

/// Reads an externalized Gemini custom `.toml` command template from `<neo>/rules/commands/` or `<skills>/rules/commands/`.
pub fn read_command_toml(filename: &str) -> String {
    let cfg = SkillsRepoConfig::load();
    let name = filename.trim();
    let candidates: Vec<PathBuf> = cfg
        .rule_search_roots()
        .into_iter()
        .map(|r| r.join("commands").join(name))
        .collect();
    read_first_matching(&candidates).unwrap_or_else(|| {
        let stem = Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name);
        format!(
            "description = \"Toksi {} command\"\nprompt = \"Run {} skill rules. Args: {{{{args}}}}\"\n",
            stem, stem
        )
    })
}

/// Reads an externalized subsystem spec (`caveman-shrink.md`, `caveman-browse.md`, `caveman-blocks.md`).
pub fn read_spec_markdown(filename: &str) -> String {
    let cfg = SkillsRepoConfig::load();
    let name = filename.trim();
    let candidates: Vec<PathBuf> = cfg
        .rule_search_roots()
        .into_iter()
        .map(|r| r.join(name))
        .collect();
    read_first_matching(&candidates).unwrap_or_default()
}

/// Reads `<neo>/rules/agent_instructions.md` or `<skills>/rules/agent_instructions.md` if present.
pub fn read_agent_instructions_template() -> Option<String> {
    let cfg = SkillsRepoConfig::load();
    let candidates: Vec<PathBuf> = cfg
        .rule_search_roots()
        .into_iter()
        .map(|r| r.join("agent_instructions.md"))
        .collect();
    read_first_matching(&candidates)
}

fn default_rule_fallback(id: &str) -> String {
    match id.trim().to_ascii_lowercase().as_str() {
        "principles" => "# Super Intelligence Agent Operating Principles\n\n<enforce>\nEpistemic Rigor, First-Principles Simplicity, Surgical Mutation, Deterministic Tool Orchestration, and Closed-Loop Verification.\n</enforce>\n".to_string(),
        "caveman" => "# Response Style (caveman)\n\n<enforce>\nRespond terse like smart caveman using ASD-STE100 rules. Drop filler, keep technical facts and negations exact.\n</enforce>\n".to_string(),
        "ponytail" => "# Build Discipline (ponytail)\n\n<enforce>\nFollow the Ladder: YAGNI, codebase reuse, stdlib/native, installed deps, one line, minimum complete code with runnable verification.\n</enforce>\n".to_string(),
        other => format!("# {}\n\n<enforce>\nRule `{}` staged externally.\n</enforce>\n", other, other),
    }
}

/// Discovers, heals, grades, security-vets, and deduplicates all available skills
/// across `[repositories].user_skills_dir`, `[repositories].skills_dir`, and `<neo>/agents/skills/`.
pub fn discover_vetted_catalog(cfg: &SkillsRepoConfig) -> Result<SkillsCatalog, RulesError> {
    let sources = vec![
        (SkillOrigin::UserCustom, cfg.user_skills_dir.clone()),
        (SkillOrigin::OfficialSkillsRepo, cfg.skills_dir.join("skills")),
        (SkillOrigin::StagedNeo, cfg.stage_agents_dir.join("skills")),
    ];
    SkillsCatalog::build_from_sources(&sources)
}

/// Fetches/imports vetted external skills from a source directory (e.g. `[repositories].user_skills_dir`
/// or an imported vault pack), heals YAML frontmatter (`skilldoctor` + `agentskills`), verifies
/// `<enforce>` security vetting, and installs clean copies into `<skills>/skills/`.
pub fn sync_external_skills_repo(cfg: &SkillsRepoConfig) -> Result<SkillsRepoSyncReport, RulesError> {
    let mut report = SkillsRepoSyncReport::default();

    let dest_rules_dir = cfg.skills_dir.join("rules");
    let dest_cmd_dir = dest_rules_dir.join("commands");
    let dest_skills_dir = cfg.skills_dir.join("skills");
    fs::create_dir_all(&dest_cmd_dir).map_err(|e| RulesError::io(&dest_cmd_dir, e))?;
    fs::create_dir_all(&dest_skills_dir).map_err(|e| RulesError::io(&dest_skills_dir, e))?;

    // 1. Sync staged `<neo>/rules/*.{md,mdc}` & `commands/*.toml` into `<skills>/rules/`
    if cfg.stage_rules_dir.is_dir() {
        let entries = fs::read_dir(&cfg.stage_rules_dir).map_err(|e| RulesError::io(&cfg.stage_rules_dir, e))?;
        for entry in entries.flatten() {
            let src = entry.path();
            if src.is_file() {
                let raw = fs::read_to_string(&src).map_err(|e| RulesError::io(&src, e))?;
                let norm = raw.replace("\r\n", "\n");
                let dst = dest_rules_dir.join(entry.file_name());
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                let guarded = if name.ends_with(".md") && !name.contains("instructions") && !has_enforce_guard(&norm) {
                    wrap_enforce_guard(&norm)
                } else {
                    norm
                };
                fs::write(&dst, guarded).map_err(|e| RulesError::io(&dst, e))?;
                report.synced_files.push(PathBuf::from(normalize_path(&dst.to_string_lossy())));
            }
        }
        let src_cmds = cfg.stage_rules_dir.join("commands");
        if src_cmds.is_dir() {
            let cmd_entries = fs::read_dir(&src_cmds).map_err(|e| RulesError::io(&src_cmds, e))?;
            for entry in cmd_entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    let raw = fs::read_to_string(&src).map_err(|e| RulesError::io(&src, e))?;
                    let dst = dest_cmd_dir.join(entry.file_name());
                    fs::write(&dst, raw.replace("\r\n", "\n")).map_err(|e| RulesError::io(&dst, e))?;
                    report.synced_files.push(PathBuf::from(normalize_path(&dst.to_string_lossy())));
                }
            }
        }
    }

    // 2. Build deduplicated, security-vetted catalog across staged, official, and user dirs
    let catalog = discover_vetted_catalog(cfg)?;
    report.catalog_count = catalog.entries.len();
    report.collision_count = catalog.collisions.len();
    report.blocked_files = catalog.blocked_skills.clone();

    for entry in &catalog.entries {
        let skill_dir = dest_skills_dir.join(&entry.name);
        fs::create_dir_all(&skill_dir).map_err(|e| RulesError::io(&skill_dir, e))?;
        let target_file = skill_dir.join("SKILL.md");
        let target_norm = normalize_path(&target_file.to_string_lossy());

        let healed = heal_skill_content(&target_norm, &entry.content, Some(&entry.name), true);
        let check = analyze_content(&target_norm, &healed.output, Some(FileKind::Skill));
        if !check.vetted_for_install {
            report.blocked_files.push((target_norm, "failed post-heal security gate".to_string()));
            continue;
        }
        fs::write(&target_file, &healed.output).map_err(|e| RulesError::io(&target_file, e))?;
        let pb = PathBuf::from(&target_norm);
        if healed.changed {
            report.healed_files.push(pb.clone());
        }
        report.synced_files.push(pb);
    }

    Ok(report)
}

/// Fetches official vetted skills online from `[repositories].skills_pages_url`
/// (e.g., `https://navegos.github.io/skills/`), heals YAML frontmatter via
/// `skilldoctor` + `agentskills`, verifies `<enforce>` security vetting, and
/// stages clean `SKILL.md` files into `cfg.skills_dir`.
pub async fn fetch_online_skills_from_pages(
    cfg: &SkillsRepoConfig,
    skill_names: &[&str],
) -> Result<SkillsRepoSyncReport, RulesError> {
    let base = cfg.skills_pages_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| RulesError::Http {
            url: base.to_string(),
            message: e.to_string(),
        })?;

    let dest_skills_dir = cfg.skills_dir.join("skills");
    fs::create_dir_all(&dest_skills_dir).map_err(|e| RulesError::io(&dest_skills_dir, e))?;

    let mut report = SkillsRepoSyncReport::default();
    for raw_name in skill_names {
        let name = raw_name.trim();
        if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
            continue;
        }
        let url = format!("{}/skills/{}/SKILL.md", base, name);
        let resp = client.get(&url).send().await.map_err(|e| RulesError::Http {
            url: url.clone(),
            message: e.to_string(),
        })?;
        if !resp.status().is_success() {
            report.blocked_files.push((
                url,
                format!("HTTP {}", resp.status().as_u16()),
            ));
            continue;
        }
        let raw = resp.text().await.map_err(|e| RulesError::Http {
            url: url.clone(),
            message: e.to_string(),
        })?;

        let target_dir = dest_skills_dir.join(name);
        let target_file = target_dir.join("SKILL.md");
        let target_norm = normalize_path(&target_file.to_string_lossy());
        let healed = heal_skill_content(&target_norm, &raw, Some(name), true);
        let check = analyze_content(&target_norm, &healed.output, Some(FileKind::Skill));
        if !check.vetted_for_install {
            report.blocked_files.push((
                target_norm,
                "failed skilldoctor/defenseclaw <enforce> security gate".to_string(),
            ));
            continue;
        }

        fs::create_dir_all(&target_dir).map_err(|e| RulesError::io(&target_dir, e))?;
        fs::write(&target_file, &healed.output).map_err(|e| RulesError::io(&target_file, e))?;
        let pb = PathBuf::from(&target_norm);
        if healed.changed {
            report.healed_files.push(pb.clone());
        }
        report.synced_files.push(pb);
        report.catalog_count += 1;
    }

    Ok(report)
}

/// Canonical skills authored or explicitly adapted for `<neo>`.
pub const CANONICAL_NEO_SKILLS: &[&str] = &[
    "aimemory",
    "codegraph",
    "ctxmode",
    "icm",
    "projectmem",
    "rtk",
    "cavecrew",
    "caveman",
    "caveman-commit",
    "caveman-compress",
    "caveman-discover",
    "caveman-evidence-review",
    "caveman-explore",
    "caveman-help",
    "caveman-learn",
    "caveman-manage",
    "caveman-optimize",
    "caveman-review",
    "caveman-stats",
    "investigate-first",
    "lean-build",
    "megacave",
    "migration",
    "ponytail",
    "ponytail-audit",
    "ponytail-debt",
    "ponytail-gain",
    "ponytail-help",
    "ponytail-review",
    "principles",
    "safe-refactor",
    "surgical-patch",
    "ultracave",
    "verify-and-stop",
    "security",
];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoHygieneAuditReport {
    pub retained_skills: Vec<String>,
    pub pruned_skills: Vec<String>,
    pub renamed_context_to_ctxmode: bool,
    pub healed_skills: Vec<String>,
    pub blocked_skills: Vec<(String, String)>,
    pub raw_script_violations: Vec<String>,
    pub embedded_bloat_violations: Vec<String>,
    pub advisory_plaintext_violations: Vec<String>,
}

fn collect_raw_skill_scripts(dir: &Path, root_label: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            collect_raw_skill_scripts(&p, root_label, out);
        } else if p.is_file() {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if matches!(ext.as_str(), "sh" | "ps1" | "bat" | "cmd") {
                let rel = p
                    .strip_prefix(root_label)
                    .map(|r| normalize_path(&r.to_string_lossy()))
                    .unwrap_or_else(|_| normalize_path(&p.to_string_lossy()));
                out.push(rel);
            }
        }
    }
}

fn scan_crates_src_for_bloat(dir: &Path, base: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            scan_crates_src_for_bloat(&p, base, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs")
            && let Ok(content) = fs::read_to_string(&p)
        {
            let rel = p
                .strip_prefix(base)
                .map(|r| normalize_path(&r.to_string_lossy()))
                .unwrap_or_else(|_| normalize_path(&p.to_string_lossy()));
            for (idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains("include_str!(")
                    && (trimmed.contains(".md\"")
                        || trimmed.contains(".yaml\"")
                        || trimmed.contains(".yml\""))
                {
                    out.push(format!("{}:{}", rel, idx + 1));
                }
            }
        }
    }
}

fn scan_advisory_plaintext(dir: &Path, base: &Path, out: &mut Vec<String>) {
    const LEAK_MARKERS: &[&str] = &[
        "BEGIN PRIVATE KEY",
        "BEGIN RSA PRIVATE KEY",
        "ghp_",
        "sk-ant-",
        "AIzaSy",
        "exploit_payload",
    ];
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name == ".git" {
            continue;
        }
        if p.is_dir() {
            scan_advisory_plaintext(&p, base, out);
        } else if p.is_file() {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if matches!(ext.as_str(), "json" | "md" | "toml" | "txt")
                && let Ok(raw) = fs::read_to_string(&p)
                && LEAK_MARKERS.iter().any(|m| raw.contains(m))
            {
                let rel = p
                    .strip_prefix(base)
                    .map(|r| normalize_path(&r.to_string_lossy()))
                    .unwrap_or_else(|_| normalize_path(&p.to_string_lossy()));
                out.push(rel);
            }
        }
    }
}

/// Audits `<neo>/.agents/skills/`, `crates/toksi/src/` zero-bloat policy, and
/// `<advisory>` zero-plaintext policy. When `clean` is true, renames `context` ->
/// `ctxmode`, deletes unadapted foreign skill dirs, heals kept skills with `<enforce>`,
/// and optionally stages vetted skills into `<neo>/agents/` and `<skills>`.
pub fn audit_neo_skills_hygiene(
    neo_root: &Path,
    clean: bool,
    stage_to_external: bool,
) -> Result<NeoHygieneAuditReport, RulesError> {
    let mut report = NeoHygieneAuditReport::default();
    let agents_skills = neo_root.join(".agents").join("skills");

    if agents_skills.is_dir() {
        let legacy_ctx = agents_skills.join("context");
        let canonical_ctxmode = agents_skills.join("ctxmode");
        if legacy_ctx.is_dir() && clean {
            if !canonical_ctxmode.exists() {
                fs::rename(&legacy_ctx, &canonical_ctxmode)
                    .map_err(|e| RulesError::io(&canonical_ctxmode, e))?;
                let skill_md = canonical_ctxmode.join("SKILL.md");
                if skill_md.is_file()
                    && let Ok(raw) = fs::read_to_string(&skill_md)
                {
                    let updated = raw
                        .replace("name: context", "name: ctxmode")
                        .replace("# context", "# ctxmode")
                        .replace("for context.", "for ctxmode.");
                    fs::write(&skill_md, updated).map_err(|e| RulesError::io(&skill_md, e))?;
                }
            } else {
                fs::remove_dir_all(&legacy_ctx).map_err(|e| RulesError::io(&legacy_ctx, e))?;
            }
            report.renamed_context_to_ctxmode = true;
        }

        let entries = fs::read_dir(&agents_skills).map_err(|e| RulesError::io(&agents_skills, e))?;
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();

        for dir in dirs {
            let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()).map(ToString::to_string)
            else {
                continue;
            };
            if !CANONICAL_NEO_SKILLS.contains(&dir_name.as_str()) {
                report.pruned_skills.push(dir_name.clone());
                if clean {
                    fs::remove_dir_all(&dir).map_err(|e| RulesError::io(&dir, e))?;
                } else {
                    collect_raw_skill_scripts(&dir, neo_root, &mut report.raw_script_violations);
                }
                continue;
            }

            collect_raw_skill_scripts(&dir, neo_root, &mut report.raw_script_violations);

            let skill_file = if dir.join("SKILL.md").is_file() {
                dir.join("SKILL.md")
            } else if dir.join("skill.md").is_file() {
                dir.join("skill.md")
            } else {
                continue;
            };
            let raw = fs::read_to_string(&skill_file).map_err(|e| RulesError::io(&skill_file, e))?;
            let norm = normalize_path(&skill_file.to_string_lossy());
            let initial_scan = crate::security::skill_scanner::scan_skill_or_plugin(&norm, &raw);
            if initial_scan.verdict != crate::security::CelVerdict::Allow {
                report
                    .blocked_skills
                    .push((dir_name.clone(), initial_scan.cel_reason));
                if clean {
                    fs::remove_dir_all(&dir).map_err(|e| RulesError::io(&dir, e))?;
                }
                continue;
            }

            let healed = heal_skill_content(&norm, &raw, Some(&dir_name), true);
            let check = analyze_content(&norm, &healed.output, Some(FileKind::Skill));
            let post_scan =
                crate::security::skill_scanner::scan_skill_or_plugin(&norm, &healed.output);
            if !check.vetted_for_install || post_scan.verdict != crate::security::CelVerdict::Allow
            {
                report
                    .blocked_skills
                    .push((dir_name.clone(), post_scan.cel_reason));
                continue;
            }

            if clean && healed.changed {
                fs::write(&skill_file, &healed.output)
                    .map_err(|e| RulesError::io(&skill_file, e))?;
                report.healed_skills.push(dir_name.clone());
            }
            report.retained_skills.push(dir_name.clone());

            if clean && stage_to_external {
                let staged_dir = neo_root.join("agents").join("skills").join(&dir_name);
                fs::create_dir_all(&staged_dir).map_err(|e| RulesError::io(&staged_dir, e))?;
                let staged_file = staged_dir.join("SKILL.md");
                fs::write(&staged_file, &healed.output)
                    .map_err(|e| RulesError::io(&staged_file, e))?;
            }
        }
    }

    let crates_src = neo_root.join("crates").join("toksi").join("src");
    if crates_src.is_dir() {
        scan_crates_src_for_bloat(&crates_src, neo_root, &mut report.embedded_bloat_violations);
    }

    let advisory_dir = neo_root.join("inspect-vault").join("advisory");
    if advisory_dir.is_dir() {
        scan_advisory_plaintext(&advisory_dir, neo_root, &mut report.advisory_plaintext_violations);
    }

    if clean
        && stage_to_external
        && (neo_root.join("agents").join("skills").is_dir()
            || neo_root.join("inspect-vault").join("skills").is_dir())
    {
        let mut scoped_cfg = SkillsRepoConfig::load();
        scoped_cfg.neo_root = Some(neo_root.to_path_buf());
        scoped_cfg.stage_rules_dir = neo_root.join("rules");
        scoped_cfg.stage_agents_dir = neo_root.join("agents");
        scoped_cfg.skills_dir = neo_root.join("inspect-vault").join("skills");
        scoped_cfg.user_skills_dir = neo_root.join(".agents").join("skills");
        let _ = sync_external_skills_repo(&scoped_cfg)?;
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_external_repo_load_and_sync() -> Result<(), RulesError> {
        let cfg = SkillsRepoConfig::load();
        assert!(!read_rule_markdown("principles").is_empty());
        assert!(has_enforce_guard(&read_rule_markdown("principles")));
        assert!(has_enforce_guard(&read_rule_markdown("caveman")));
        assert!(has_enforce_guard(&read_rule_markdown("ponytail")));

        let tmp = tempdir().map_err(|e| RulesError::io("tmp", e))?;
        let mut isolated = cfg.clone();
        isolated.skills_dir = tmp.path().join("skills-repo");
        isolated.user_skills_dir = tmp.path().join("user-skills");
        let sync_rep = sync_external_skills_repo(&isolated)?;
        assert!(sync_rep.catalog_count >= 28);
        assert!(sync_rep.blocked_files.is_empty());
        assert!(isolated.skills_dir.join("skills/ultracave/SKILL.md").is_file());
        assert!(isolated.skills_dir.join("skills/ponytail-debt/SKILL.md").is_file());
        assert!(isolated.skills_dir.join("skills/investigate-first/SKILL.md").is_file());
        assert!(isolated.skills_dir.join("skills/caveman-explore/SKILL.md").is_file());
        Ok(())
    }

    #[test]
    fn test_empty_repositories_fields_fall_back_to_detected_defaults() {
        let parsed: ConfigRepositoriesToml = toml::from_str(
            "[repositories]\nneo_dir = \"\"\nstage_dir_outside_crates = \"  \"\nskills_dir = \"\"\nuser_skills_dir = \"\"\n",
        )
        .expect("parse repositories table");
        let cfg = SkillsRepoConfig::from_repositories_section(parsed.repositories, detect_neo_root());
        assert!(cfg.neo_root.is_some());
        assert!(cfg.stage_rules_dir.join("principles.md").is_file());
        assert!(cfg.stage_agents_dir.join("skills/principles/SKILL.md").is_file());
        assert!(!cfg.skills_dir.as_os_str().is_empty());
        assert!(!cfg.user_skills_dir.as_os_str().is_empty());
    }
}
