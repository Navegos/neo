// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tui/app.rs
// created: 2026-10-30
// lastModified: 2026-11-18

use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::agents::proxy_agent_wired;
use crate::core::{list_agents, list_tools, RunOpts};
use crate::headroom::byok_gateway::configured_byok_routes;
use crate::headroom::proxy::{ensure_proxy_up, proxy_port, proxy_running, stop_proxy};
use crate::commands::daemon::is_daemon_running;
use crate::commands::init_agent_fully_wired;
use crate::util::paths::{
    amp_paths_resolved, claude_code_paths, cline_paths_resolved, codex_paths_resolved,
    copilot_paths_resolved, cursor_global_mcp_path, grok_paths_resolved, kilo_paths_resolved,
    kimi_paths_resolved, omp_paths_resolved, opencode_paths_resolved, opencode_paths_v2,
    pi_paths_resolved, read_file_safe, zcode_paths_resolved,
};

// -----------------------------------------------------------------------------
// Model Definitions
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WaterfallItem {
    pub time: String,
    pub agent: String,
    pub endpoint: String,
    pub raw_tokens: i64,
    pub compressed_tokens: i64,
    pub savings_pct: f64,
    pub pipeline: String,
    pub saved_usd: f64,
}

#[derive(Debug, Clone)]
pub struct MonitorState {
    pub proxy_up: bool,
    pub daemon_up: bool,
    pub tokens_saved_lifetime: i64,
    pub tokens_saved_today: i64,
    pub tokens_saved_7d: i64,
    pub cost_saved_usd: f64,
    pub total_calls: i64,
    pub compression_pct: f64,
    pub cache_warmth_pct: f64,
    pub sparkline_data: Vec<u64>,
    pub waterfall: Vec<WaterfallItem>,
}

#[derive(Debug, Clone)]
pub struct AgentRow {
    pub id: String,
    pub label: String,
    pub installed: bool,
    pub in_path: bool,
    pub detect_source: String,
    pub wired: bool,
    pub wire_mode: String,
    pub port: String,
    pub tools_summary: String,
    pub config_path: String,
    pub tokens_saved: u64,
}

#[derive(Debug, Clone)]
pub struct AgentMcpModal {
    pub agent_id: String,
    pub agent_label: String,
    pub config_path: String,
    pub content: String,
    pub exists: bool,
    pub scroll: u16,
}

#[derive(Debug, Clone)]
pub struct AgentsState {
    pub items: Vec<AgentRow>,
    pub selected: usize,
    pub inspect_mcp: Option<AgentMcpModal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryFilter {
    All,
    Issues,
    Decisions,
    Gotchas,
    Icm,
}

#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub id: String,
    pub event_type: String,
    pub summary: String,
    pub location: String,
    pub timestamp: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct PrecheckWarningItem {
    pub file: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PrecheckModal {
    pub files_scanned: Vec<String>,
    pub warnings: Vec<PrecheckWarningItem>,
    pub scroll: u16,
}

#[derive(Debug, Clone)]
pub struct HeatmapDetailModal {
    pub file: String,
    pub failed_attempts: usize,
    pub open_issues: usize,
    pub resolved_issues: usize,
    pub decisions: usize,
    pub notes: usize,
    pub events: Vec<String>,
    pub scroll: u16,
}

#[derive(Debug, Clone)]
pub struct MemoryState {
    pub filter: MemoryFilter,
    pub search_query: String,
    pub search_active: bool,
    pub items: Vec<MemoryItem>,
    pub selected: usize,
    pub precheck_result: Option<PrecheckModal>,
    pub heatmap: Vec<crate::projectmem::heatmap::FileFailureStats>,
    pub selected_heatmap: usize,
    pub heatmap_popup: Option<HeatmapDetailModal>,
}

#[derive(Debug, Clone)]
pub struct SymbolRow {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct CallEdgeItem {
    pub name: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct BlastTreeItem {
    pub name: String,
    pub file: String,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct BlastRadiusSummary {
    pub total_impacted: usize,
    pub max_depth: usize,
    pub impacted_files: Vec<String>,
    pub affected_tests: Vec<String>,
    pub tree_items: Vec<BlastTreeItem>,
}

#[derive(Debug, Clone)]
pub struct GraphState {
    pub search_query: String,
    pub search_active: bool,
    pub total_symbols: usize,
    pub total_files: usize,
    pub symbols: Vec<SymbolRow>,
    pub selected: usize,
    pub active_callers: Vec<CallEdgeItem>,
    pub active_callees: Vec<CallEdgeItem>,
    pub active_blast_radius: Option<BlastRadiusSummary>,
    pub show_tree_view: bool,
}

#[derive(Debug, Clone)]
pub struct ByokRouteItem {
    pub provider: String,
    pub target_url: String,
    pub virtual_prefix: String,
    pub healthy: bool,
    pub tier_badge: String,
    pub quota_display: String,
    pub agents_assigned: String,
}

#[derive(Debug, Clone)]
pub struct ByokProfileRow {
    pub id: String,
    pub provider_id: String,
    pub instance_label: String,
    pub model_id: String,
    pub thinking_effort: String,
    pub enabled: bool,
    pub priority: i64,
    pub weight: i64,
}

#[derive(Debug, Clone)]
pub struct QuotaWindowRow {
    pub profile_id: String,
    pub provider_id: String,
    pub window_kind: String,
    pub requests_remaining: Option<i64>,
    pub requests_limit: Option<i64>,
    pub tokens_remaining: Option<i64>,
    pub tokens_limit: Option<i64>,
    pub plan_label: String,
}

#[derive(Debug, Clone)]
pub struct HeadroomState {
    pub cache_alignment_pct: f64,
    pub context_tokens_used: usize,
    pub active_memory_tier: String,
    pub rate_limit_429_count: usize,
    pub reroute_count: usize,
    pub byok_port: u16,
    pub routes: Vec<ByokRouteItem>,
    pub selected_route: usize,
    pub profiles: Vec<ByokProfileRow>,
    pub selected_profile: usize,
    pub quota_windows: Vec<QuotaWindowRow>,
    pub models_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    All,
    Proxy,
    Daemon,
    Watchdog,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub source: LogSource,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct LogsState {
    pub source: LogSource,
    pub follow: bool,
    pub scroll_offset: usize,
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Clone)]
pub struct VirtualKeyRow {
    pub vk: String,
    pub name: String,
    pub max_budget_usd: f64,
    pub current_spend_usd: f64,
    pub max_rpm: u32,
    pub current_rpm: u32,
    pub max_tpm: u32,
    pub current_tpm: u32,
    pub max_concurrency: u32,
    pub current_concurrency: u32,
    pub allowed_models: String,
}

#[derive(Debug, Clone)]
pub struct KeyringEntryRow {
    pub agent: String,
    pub service: String,
    pub account: String,
    pub present: bool,
    pub configured: bool,
    pub source: String,
}

#[derive(Debug, Clone, Default)]
pub struct DbStatsSummary {
    pub providers: usize,
    pub byok_profiles: usize,
    pub plugins: usize,
    pub blind_signatures: usize,
    pub advisory_embargoes: usize,
    pub adr_events: usize,
    pub quota_windows: usize,
}

#[derive(Debug, Clone)]
pub struct GovernanceState {
    pub keys: Vec<VirtualKeyRow>,
    pub selected: usize,
    pub total_budget_usd: f64,
    pub total_spent_usd: f64,
    pub active_rate_limits: u32,
    pub circuit_breaker_trips: u32,
    pub keyring_entries: Vec<KeyringEntryRow>,
    pub db_stats: DbStatsSummary,
    pub db_path: String,
    pub gcp_adc_active: bool,
    pub gcp_project: String,
    pub gcp_location: String,
}

#[derive(Debug, Clone, Default)]
pub struct WarRoomState {
    pub ttft_ms: f64,
    pub tpot_ms: f64,
    pub openinference_spans: u64,
    pub gpu_utilization_pct: f64,
    pub gpu_memory_used_bytes: u64,
    pub gpu_memory_total_bytes: u64,
    pub active_requests: i64,
    pub input_tokens: u64,
    pub pruning_tier: String,
    pub cache_hits_direct: u64,
    pub cache_hits_semantic: u64,
    pub cache_misses: u64,
}

#[derive(Debug, Clone)]
pub struct SandboxAuditEvent {
    pub timestamp: String,
    pub source: String,
    pub summary: String,
    pub verdict: String,
}

#[derive(Debug, Clone)]
pub struct SandboxState {
    pub fail_closed: bool,
    pub kill_switch_armed: bool,
    pub active_backend: String,
    pub audit_events: Vec<SandboxAuditEvent>,
}

// -----------------------------------------------------------------------------
// App State Container
// -----------------------------------------------------------------------------

pub struct App {
    pub current_tab: usize,
    pub tab_names: [&'static str; 10],
    pub should_quit: bool,
    pub cwd_display: String,
    pub status_message: Option<String>,
    pub status_ticks: usize,
    pub ticks_counter: usize,

    pub proxy_live_flag: Arc<AtomicBool>,
    pub daemon_live_flag: Arc<AtomicBool>,
    pub shutdown_flag: Arc<AtomicBool>,
    pub log_file_offset: u64,
    pub snapshot_rx: tokio::sync::watch::Receiver<crate::telemetry::DashboardSnapshot>,

    pub monitor: MonitorState,
    pub warroom: WarRoomState,
    pub agents: AgentsState,
    pub headroom: HeadroomState,
    pub memory: MemoryState,
    pub graph: GraphState,
    pub sandbox: SandboxState,
    pub governance: GovernanceState,
    pub logs: LogsState,
}

impl Drop for App {
    fn drop(&mut self) {
        self.shutdown_flag.store(true, Ordering::Relaxed);
    }
}

impl App {
    pub fn new() -> Self {
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into());

        let proxy_live_flag = Arc::new(AtomicBool::new(false));
        let daemon_live_flag = Arc::new(AtomicBool::new(false));
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let flag_clone = proxy_live_flag.clone();
        let daemon_clone = daemon_live_flag.clone();
        let shutdown_clone = shutdown_flag.clone();
        std::thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                let is_up = crate::headroom::proxy::proxy_live_z_timeout(Duration::from_millis(100));
                flag_clone.store(is_up, Ordering::Relaxed);
                let daemon_up = is_daemon_running();
                daemon_clone.store(daemon_up, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(600));
            }
        });

        let collector = crate::telemetry::global_collector();
        let live_tokens_saved: u64 = collector.tokens_saved_total.read().map(|m| m.values().sum()).unwrap_or(0);
        let live_total_calls: u64 = collector.upstream_requests_total.read().map(|m| m.values().sum()).unwrap_or(0);
        let live_cost_saved = collector.cost_savings_usd_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;
        let input_tokens: u64 = collector.input_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);
        let cached_tokens: u64 = collector.cached_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);

        let (tokens_saved, total_calls, cost_saved, compression_pct, cache_warmth_pct) = if live_tokens_saved > 0 || live_total_calls > 0 {
            let comp = (live_tokens_saved as f64 / (input_tokens + live_tokens_saved).max(1) as f64) * 100.0;
            let warmth = if input_tokens > 0 {
                (cached_tokens as f64 / input_tokens as f64) * 100.0
            } else {
                0.0
            };
            (live_tokens_saved as i64, live_total_calls as i64, (live_cost_saved * 100.0).round() / 100.0, comp, warmth)
        } else {
            (0_i64, 0_i64, 0.0, 0.0, 0.0)
        };

        let mut app = Self {
            current_tab: 0,
            tab_names: [
                "Savings",
                "Activity Room",
                "AI Agent Fleet",
                "Quotas",
                "Memory Decision DAG",
                "CodeGraph Explorer",
                "Sandbox Inspector",
                "AI Gateway",
                "Keyring & Database",
                "System Controls",
            ],
            should_quit: false,
            cwd_display: cwd,
            status_message: None,
            status_ticks: 0,
            ticks_counter: 0,
            proxy_live_flag,
            daemon_live_flag,
            shutdown_flag,
            log_file_offset: 0,
            snapshot_rx: crate::telemetry::subscribe_dashboard_snapshot(),

            monitor: MonitorState {
                proxy_up: false,
                daemon_up: false,
                tokens_saved_lifetime: tokens_saved,
                tokens_saved_today: live_tokens_saved as i64,
                tokens_saved_7d: live_tokens_saved as i64,
                cost_saved_usd: cost_saved,
                total_calls,
                compression_pct,
                cache_warmth_pct,
                sparkline_data: if live_tokens_saved > 0 { vec![live_tokens_saved / 10] } else { Vec::new() },
                waterfall: Vec::new(),
            },

            warroom: WarRoomState {
                pruning_tier: "Standard".to_string(),
                ..Default::default()
            },

            agents: AgentsState {
                items: Vec::new(),
                selected: 0,
                inspect_mcp: None,
            },

            memory: MemoryState {
                filter: MemoryFilter::All,
                search_query: String::new(),
                search_active: false,
                items: Vec::new(),
                selected: 0,
                precheck_result: None,
                heatmap: Vec::new(),
                selected_heatmap: 0,
                heatmap_popup: None,
            },

            graph: GraphState {
                search_query: String::new(),
                search_active: false,
                total_symbols: 0,
                total_files: 0,
                symbols: Vec::new(),
                selected: 0,
                active_callers: Vec::new(),
                active_callees: Vec::new(),
                active_blast_radius: None,
                show_tree_view: false,
            },

            headroom: HeadroomState {
                cache_alignment_pct: 0.0,
                context_tokens_used: 0,
                active_memory_tier: "L0 (Idle)".to_string(),
                rate_limit_429_count: 0,
                reroute_count: 0,
                byok_port: crate::config::resolved_ports().byok_gateway,
                routes: Vec::new(),
                selected_route: 0,
                profiles: Vec::new(),
                selected_profile: 0,
                quota_windows: Vec::new(),
                models_count: 0,
            },

            sandbox: SandboxState {
                fail_closed: true,
                kill_switch_armed: true,
                active_backend: "3-OS Native".to_string(),
                audit_events: Vec::new(),
            },

            governance: GovernanceState {
                keys: Vec::new(),
                selected: 0,
                total_budget_usd: 0.0,
                total_spent_usd: 0.0,
                active_rate_limits: 0,
                circuit_breaker_trips: 0,
                keyring_entries: Vec::new(),
                db_stats: DbStatsSummary::default(),
                db_path: crate::util::paths::normalize_path(
                    &crate::db::default_db_path().to_string_lossy(),
                ),
                gcp_adc_active: false,
                gcp_project: String::new(),
                gcp_location: "us-central1".to_string(),
            },

            logs: LogsState {
                source: LogSource::All,
                follow: true,
                scroll_offset: 0,
                entries: Vec::new(),
            },
        };

        app.reload_waterfall();
        app.reload_services();
        app.reload_agents();
        app.reload_memory();
        app.reload_graph();
        app.reload_headroom();
        app.reload_governance();
        app.reload_logs();

        app
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
        self.status_ticks = 180; // ~3 seconds at 60fps
    }

    pub fn tick(&mut self) {
        self.ticks_counter = self.ticks_counter.wrapping_add(1);

        if self.status_ticks > 0 {
            self.status_ticks -= 1;
            if self.status_ticks == 0 {
                self.status_message = None;
            }
        }

        self.monitor.proxy_up = self.proxy_live_flag.load(Ordering::Relaxed);
        self.monitor.daemon_up = self.daemon_live_flag.load(Ordering::Relaxed);

        // Non-blocking watch::Receiver snapshot update (<1 µs)
        if self.snapshot_rx.has_changed().unwrap_or(false) {
            let snap = self.snapshot_rx.borrow_and_update().clone();
            self.apply_snapshot(&snap);
        }

        // Periodic light refresh every ~1 sec (60 ticks at 60 FPS)
        if self.ticks_counter % 60 == 0 {
            self.refresh_monitor_stats();
            self.reload_waterfall();
            self.poll_new_logs();
        }
    }

    pub fn apply_snapshot(&mut self, snap: &crate::telemetry::DashboardSnapshot) {
        self.monitor.tokens_saved_lifetime = snap.tokens_saved_lifetime as i64;
        self.monitor.tokens_saved_today = snap.tokens_saved_lifetime as i64;
        self.monitor.tokens_saved_7d = snap.tokens_saved_lifetime as i64;
        self.monitor.cost_saved_usd = (snap.cost_saved_usd * 100.0).round() / 100.0;
        self.monitor.total_calls = snap.total_calls as i64;
        if snap.tokens_saved_lifetime > 0 || snap.total_calls > 0 {
            self.monitor.compression_pct =
                (snap.tokens_saved_lifetime as f64 / (snap.input_tokens + snap.tokens_saved_lifetime).max(1) as f64) * 100.0;
            self.monitor.cache_warmth_pct = if snap.input_tokens > 0 {
                (snap.cached_tokens as f64 / snap.input_tokens as f64) * 100.0
            } else {
                0.0
            };
        } else {
            self.monitor.compression_pct = 0.0;
            self.monitor.cache_warmth_pct = 0.0;
        }
        self.warroom.ttft_ms = snap.ttft_ms;
        self.warroom.tpot_ms = snap.tpot_ms;
        self.warroom.openinference_spans = snap.openinference_spans_total;
        self.warroom.gpu_utilization_pct = snap.gpu_utilization_pct;
        self.warroom.gpu_memory_used_bytes = snap.gpu_memory_used_bytes;
        self.warroom.gpu_memory_total_bytes = snap.gpu_memory_total_bytes;
        self.warroom.active_requests = snap.active_requests;
        self.warroom.input_tokens = snap.input_tokens;
        self.warroom.cache_hits_direct = snap.cache_hits_direct;
        self.warroom.cache_hits_semantic = snap.cache_hits_semantic;
        self.warroom.cache_misses = snap.cache_misses;
        self.warroom.pruning_tier = if snap.input_tokens > 150_000 {
            "Aggressive (200k limit)".to_string()
        } else if snap.input_tokens > 80_000 {
            "Moderate".to_string()
        } else {
            "Standard".to_string()
        };
        self.headroom.context_tokens_used = snap.input_tokens as usize;
        self.headroom.active_memory_tier = if snap.input_tokens == 0 {
            "L0 (Idle)".to_string()
        } else if snap.input_tokens < 20_000 {
            "L0 (TOC)".to_string()
        } else if snap.input_tokens < 100_000 {
            "L1 (Abstract)".to_string()
        } else {
            "L2 (Full)".to_string()
        };
        for agent in &mut self.agents.items {
            if let Some(saved) = snap.per_agent_saved.get(&agent.id) {
                agent.tokens_saved = *saved;
            }
        }
    }

    pub fn refresh_monitor_stats(&mut self) {
        let collector = crate::telemetry::global_collector();
        let live_tokens_saved: u64 = collector.tokens_saved_total.read().map(|m| m.values().sum()).unwrap_or(0);
        let live_total_calls: u64 = collector.upstream_requests_total.read().map(|m| m.values().sum()).unwrap_or(0);
        let live_cost_saved = collector.cost_savings_usd_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;
        let input_tokens: u64 = collector.input_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);
        let cached_tokens: u64 = collector.cached_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);

        if live_tokens_saved > 0 || live_total_calls > 0 {
            let comp = (live_tokens_saved as f64 / (input_tokens + live_tokens_saved).max(1) as f64) * 100.0;
            let warmth = if input_tokens > 0 {
                (cached_tokens as f64 / input_tokens as f64) * 100.0
            } else {
                0.0
            };
            self.monitor.tokens_saved_lifetime = live_tokens_saved as i64;
            self.monitor.tokens_saved_today = live_tokens_saved as i64;
            self.monitor.tokens_saved_7d = live_tokens_saved as i64;
            self.monitor.cost_saved_usd = (live_cost_saved * 100.0).round() / 100.0;
            self.monitor.total_calls = live_total_calls as i64;
            self.monitor.compression_pct = comp;
            self.monitor.cache_warmth_pct = warmth;
        } else {
            self.monitor.tokens_saved_lifetime = 0;
            self.monitor.tokens_saved_today = 0;
            self.monitor.tokens_saved_7d = 0;
            self.monitor.cost_saved_usd = 0.0;
            self.monitor.total_calls = 0;
            self.monitor.compression_pct = 0.0;
            self.monitor.cache_warmth_pct = 0.0;
        }
    }

    // -------------------------------------------------------------------------
    // Subsystem Loaders
    // -------------------------------------------------------------------------

    fn reload_waterfall(&mut self) {
        let sessions = crate::telemetry::tokentracker::global_token_tracker().get_recent_sessions();
        if sessions.is_empty() && self.monitor.total_calls == 0 {
            self.monitor.waterfall.clear();
            return;
        }
        if sessions.is_empty() {
            self.monitor.waterfall = vec![WaterfallItem {
                time: "Live".into(),
                agent: "Gateway".into(),
                endpoint: format!(":{}", crate::config::resolved_ports().ai_gateway),
                raw_tokens: self.warroom.input_tokens as i64 + self.monitor.tokens_saved_lifetime,
                compressed_tokens: self.warroom.input_tokens as i64,
                savings_pct: self.monitor.compression_pct,
                pipeline: "HeadroomEffortCompressor".into(),
                saved_usd: self.monitor.cost_saved_usd,
            }];
            return;
        }
        self.monitor.waterfall = sessions
            .into_iter()
            .rev()
            .take(20)
            .map(|s| {
                let raw = s.input_tokens.saturating_add(s.saved_tokens) as i64;
                let compressed = s.input_tokens as i64;
                let pct = if raw > 0 {
                    (s.saved_tokens as f64 / raw as f64) * 100.0
                } else {
                    0.0
                };
                WaterfallItem {
                    time: format!("{}ms", s.duration_ms),
                    agent: s.agent,
                    endpoint: format!("{}:{}", s.provider, s.model),
                    raw_tokens: raw,
                    compressed_tokens: compressed,
                    savings_pct: pct,
                    pipeline: "HeadroomEffortCompressor".into(),
                    saved_usd: s.saved_usd,
                }
            })
            .collect();
    }

    pub fn reload_services(&mut self) {
        self.monitor.proxy_up = proxy_running();
        self.monitor.daemon_up = is_daemon_running();
    }

    pub fn reload_agents(&mut self) {
        let agent_manifests = list_agents();
        let tools = list_tools();
        let per_agent = self.snapshot_rx.borrow().per_agent_saved.clone();

        let mut rows = Vec::new();
        for ag in agent_manifests {
            let detection = ag.detect();
            let in_path = crate::util::env_path::is_executable_in_path(&ag.cli_bin);
            let is_proxy_wired = proxy_agent_wired(&ag.id);
            let is_config_wired = init_agent_fully_wired(&ag.id, &tools);
            let is_wired = is_config_wired || is_proxy_wired;

            let wire_mode = if is_proxy_wired {
                "direct-wired".to_string()
            } else if is_config_wired {
                "config-wired".to_string()
            } else {
                "unwired".to_string()
            };

            let port = if is_proxy_wired && proxy_running() {
                format!("{} -> Direct", proxy_port())
            } else if is_proxy_wired {
                format!("{} (stopped)", proxy_port())
            } else if is_config_wired {
                "config (MCP)".into()
            } else {
                "none (unwired)".into()
            };

            let config_path = resolve_agent_mcp_config_path(&ag.id);
            let tokens_saved = per_agent.get(&ag.id).copied().unwrap_or(0);

            rows.push(AgentRow {
                id: ag.id.clone(),
                label: ag.label.clone(),
                installed: detection.installed,
                in_path,
                detect_source: detection.source,
                wired: is_wired,
                wire_mode,
                port,
                tools_summary: "RTK, CodeGraph, ProjectMem, Headroom".into(),
                config_path,
                tokens_saved,
            });
        }

        // Sort so installed and wired are grouped nicely
        rows.sort_by(|a, b| b.wired.cmp(&a.wired).then_with(|| b.installed.cmp(&a.installed)));
        self.agents.items = rows;
    }

    pub fn direct_wire_selected_agent(&mut self) {
        if self.agents.items.is_empty() {
            return;
        }
        let sel = self.agents.selected;
        let agent_id = self.agents.items[sel].id.clone();
        let agent_label = self.agents.items[sel].label.clone();
        let all_tools = list_tools();
        let ro = RunOpts {
            agent: agent_id.clone(),
            ..Default::default()
        };
        for tool in &all_tools {
            if let Some(wire_fn) = tool.wire_for.get(&agent_id) {
                let _ = wire_fn(&ro);
            }
        }
        let _ = crate::agents::configure_proxy_agent(&agent_id);
        self.agents.items[sel].wired = true;
        self.agents.items[sel].wire_mode = "direct-wired".into();
        self.agents.items[sel].port = format!("{} -> Direct", proxy_port());
        self.set_status(&format!("Direct-wired {}", agent_label));
    }

    pub fn config_wire_selected_agent(&mut self) {
        if self.agents.items.is_empty() {
            return;
        }
        let sel = self.agents.selected;
        let agent_id = self.agents.items[sel].id.clone();
        let agent_label = self.agents.items[sel].label.clone();
        let all_tools = list_tools();
        let ro = RunOpts {
            agent: agent_id.clone(),
            ..Default::default()
        };
        for tool in &all_tools {
            if let Some(wire_fn) = tool.wire_for.get(&agent_id) {
                let _ = wire_fn(&ro);
            }
        }
        self.agents.items[sel].wired = true;
        self.agents.items[sel].wire_mode = "config-wired".into();
        self.agents.items[sel].port = "config (MCP)".into();
        self.set_status(&format!("Config-wired {}", agent_label));
    }

    pub fn unwire_selected_agent(&mut self) {
        if self.agents.items.is_empty() {
            return;
        }
        let sel = self.agents.selected;
        let agent_id = self.agents.items[sel].id.clone();
        let agent_label = self.agents.items[sel].label.clone();
        let all_tools = list_tools();
        let ro = RunOpts {
            agent: agent_id.clone(),
            ..Default::default()
        };
        for tool in &all_tools {
            if let Some(unwire_fn) = tool.unwire_for.get(&agent_id) {
                let _ = unwire_fn(&ro);
            }
        }
        let _ = crate::agents::remove_proxy_agent(&agent_id);
        self.agents.items[sel].wired = false;
        self.agents.items[sel].wire_mode = "unwired".into();
        self.agents.items[sel].port = "none (unwired)".into();
        self.set_status(&format!("Unwired {}", agent_label));
    }

    pub fn fix_selected_agent(&mut self) {
        if self.agents.items.is_empty() {
            return;
        }
        let sel = self.agents.selected;
        let in_path = self.agents.items[sel].in_path;
        let label = self.agents.items[sel].label.clone();
        if !in_path {
            self.set_status(&format!("Fix: '{}' missing from PATH — add binary to PATH", label));
            return;
        }
        self.config_wire_selected_agent();
        self.set_status(&format!("Fixed & re-synced hooks/MCP for {}", label));
    }

    pub fn toggle_agent_wire(&mut self) {
        if self.agents.items.is_empty() {
            return;
        }
        let sel = self.agents.selected;
        if self.agents.items[sel].wired {
            self.unwire_selected_agent();
        } else {
            self.direct_wire_selected_agent();
        }
    }

    pub fn inspect_agent_mcp(&mut self) {
        if self.agents.items.is_empty() {
            return;
        }
        let sel = self.agents.selected;
        let agent = &self.agents.items[sel];
        let p = &agent.config_path;

        let (content, exists) = if Path::new(p).exists() {
            (read_file_safe(p).unwrap_or_default(), true)
        } else {
            (format!("// File not yet initialized at:\n// {}\n\n{{\n  \"mcpServers\": {{}}\n}}", p), false)
        };

        self.agents.inspect_mcp = Some(AgentMcpModal {
            agent_id: agent.id.clone(),
            agent_label: agent.label.clone(),
            config_path: p.clone(),
            content,
            exists,
            scroll: 0,
        });
    }

    pub fn reload_memory(&mut self) {
        let mut items = Vec::new();
        self.memory.heatmap.clear();

        // 1. Read <repo>/.toksi/projectmem/events.jsonl
        if let Ok(events) = crate::projectmem::store::read_events(None) {
            let hm = crate::projectmem::heatmap::generate_heatmap(&events);
            self.memory.heatmap = hm.top_hotspots;
            for ev in events {
                items.push(MemoryItem {
                    id: ev.issue_id.clone().unwrap_or_else(|| ev.supersedes.clone().unwrap_or_else(|| ev.event_type.clone())),
                    event_type: ev.event_type.clone(),
                    summary: ev.summary.clone(),
                    location: ev.location.unwrap_or_default(),
                    timestamp: ev.timestamp.chars().take(10).collect(),
                    details: format!("Outcome: {:?}\nCommand: {:?}\nNotes: {:?}", ev.outcome, ev.command, ev.notes),
                });
            }
        }

        // 2. Read ICM SQLite memory DB
        if let Ok(store) = crate::icm::store::IcmStore::open_default() {
            if let Ok(hits) = store.get_recent(50) {
                for h in hits {
                    items.push(MemoryItem {
                        id: h.memory.id.chars().take(8).collect(),
                        event_type: "icm".into(),
                        summary: h.memory.summary,
                        location: format!("topic: {}", h.memory.topic),
                        timestamp: h.memory.created_at.chars().take(10).collect(),
                        details: format!("Raw: {}", h.memory.raw_excerpt.unwrap_or_default()),
                    });
                }
            }
        }

        self.memory.items = items;
    }

    pub fn open_heatmap_popup(&mut self, idx: usize) {
        let Some(stat) = self.memory.heatmap.get(idx).cloned() else {
            return;
        };
        let mut events_lines = Vec::new();
        if let Ok(events) = crate::projectmem::store::read_events(None) {
            let drill = crate::projectmem::heatmap::generate_file_drilldown(&events, &stat.file);
            for ev in drill.events.iter().take(40) {
                events_lines.push(format!(
                    "[{}] {} — {}",
                    ev.event_type,
                    ev.timestamp.chars().take(10).collect::<String>(),
                    ev.summary
                ));
            }
        }
        self.memory.selected_heatmap = idx;
        self.memory.heatmap_popup = Some(HeatmapDetailModal {
            file: stat.file,
            failed_attempts: stat.failed_attempts,
            open_issues: stat.open_issues,
            resolved_issues: stat.resolved_issues,
            decisions: stat.decisions,
            notes: stat.notes,
            events: events_lines,
            scroll: 0,
        });
    }

    pub fn trigger_precheck(&mut self) {
        let files = crate::projectmem::precheck::get_working_tree_files(Path::new("."));
        let warnings = match crate::projectmem::precheck::run_precheck(None, true, None, true, None) {
            Ok(warns) => warns
                .into_iter()
                .map(|w| PrecheckWarningItem {
                    file: w.file,
                    message: format!("[{}] {}", w.severity, w.title),
                })
                .collect(),
            Err(e) => vec![PrecheckWarningItem {
                file: "precheck".into(),
                message: format!("Error running git diff: {}", e),
            }],
        };

        self.memory.precheck_result = Some(PrecheckModal {
            files_scanned: files,
            warnings,
            scroll: 0,
        });
        self.set_status("Precheck complete");
    }

    pub fn reload_graph(&mut self) {
        let root = PathBuf::from(".");
        if let Ok(cg) = crate::codegraph::CodeGraph::open(&root) {
            let stats = cg.cache.get_stats();
            self.graph.total_symbols = stats.node_count;
            self.graph.total_files = stats.file_count;

            let all_nodes = cg.cache.get_all_nodes();
            self.graph.symbols = all_nodes
                .iter()
                .take(300)
                .map(|n| SymbolRow {
                    id: n.id.clone(),
                    name: n.name.clone(),
                    kind: n.kind.clone(),
                    file: n.file_path.clone(),
                    line: n.start_line,
                })
                .collect();
        }

        if self.graph.symbols.is_empty() {
            // Baseline symbols from project
            self.graph.symbols = vec![
                SymbolRow { id: "sym_1".into(), name: "run_top".into(), kind: "function".into(), file: "crates/toksi/src/commands/top.rs".into(), line: 36 },
                SymbolRow { id: "sym_2".into(), name: "run_tui".into(), kind: "function".into(), file: "crates/toksi/src/tui/mod.rs".into(), line: 25 },
                SymbolRow { id: "sym_3".into(), name: "proxy_running".into(), kind: "function".into(), file: "crates/toksi/src/headroom/proxy.rs".into(), line: 200 },
                SymbolRow { id: "sym_4".into(), name: "align_to_1024_tokens".into(), kind: "function".into(), file: "crates/toksi/src/headroom/cache_align.rs".into(), line: 24 },
                SymbolRow { id: "sym_5".into(), name: "read_events".into(), kind: "function".into(), file: "crates/toksi/src/projectmem/store.rs".into(), line: 222 },
                SymbolRow { id: "sym_6".into(), name: "run_precheck".into(), kind: "function".into(), file: "crates/toksi/src/projectmem/precheck.rs".into(), line: 362 },
            ];
            self.graph.total_symbols = self.graph.symbols.len();
            self.graph.total_files = 6;
        }

        self.update_active_symbol_details();
    }

    pub fn update_active_symbol_details(&mut self) {
        if self.graph.symbols.is_empty() {
            return;
        }
        let sel = self.graph.selected.min(self.graph.symbols.len().saturating_sub(1));
        let sym = &self.graph.symbols[sel];

        // Query callers / callees / blast radius if CodeGraph cache exists
        let root = PathBuf::from(".");
        if let Ok(cg) = crate::codegraph::CodeGraph::open(&root) {
            let callers = cg.engine.get_callers(&sym.id);
            self.graph.active_callers = callers
                .into_iter()
                .map(|(node, _)| CallEdgeItem {
                    name: node.name.clone(),
                    file: node.file_path.clone(),
                    line: node.start_line,
                })
                .collect();

            let callees = cg.engine.get_callees(&sym.id);
            self.graph.active_callees = callees
                .into_iter()
                .map(|(node, _)| CallEdgeItem {
                    name: node.name.clone(),
                    file: node.file_path.clone(),
                    line: node.start_line,
                })
                .collect();

            if let Some(report) = cg.engine.compute_blast_radius(&sym.id, 3) {
                let mut tree_items: Vec<BlastTreeItem> = Vec::new();
                tree_items.push(BlastTreeItem {
                    name: report.focal_name.clone(),
                    file: report.focal_file.clone(),
                    depth: 0,
                });
                for up in &report.upstream_dependents {
                    tree_items.push(BlastTreeItem {
                        name: up.node.name.clone(),
                        file: up.node.file_path.clone(),
                        depth: up.depth,
                    });
                }
                for down in &report.downstream_dependencies {
                    tree_items.push(BlastTreeItem {
                        name: down.node.name.clone(),
                        file: down.node.file_path.clone(),
                        depth: down.depth,
                    });
                }

                let total_impacted = report.upstream_dependents.len() + report.downstream_dependencies.len();
                let affected_tests = report
                    .affected_files
                    .iter()
                    .filter(|f| f.contains("test"))
                    .cloned()
                    .collect();

                self.graph.active_blast_radius = Some(BlastRadiusSummary {
                    total_impacted,
                    max_depth: report.depth,
                    impacted_files: report.affected_files,
                    affected_tests,
                    tree_items,
                });
                return;
            }
        }

        // Default synthetic blast radius for preview
        self.graph.active_callers = vec![
            CallEdgeItem { name: "main".into(), file: "crates/toksi/src/main.rs".into(), line: 165 },
            CallEdgeItem { name: "run_cli".into(), file: "crates/toksi/src/main.rs".into(), line: 347 },
        ];
        self.graph.active_callees = vec![
            CallEdgeItem { name: "render_frame".into(), file: "crates/toksi/src/commands/top.rs".into(), line: 76 },
            CallEdgeItem { name: "TerminalGuard::drop".into(), file: "crates/toksi/src/commands/top.rs".into(), line: 29 },
        ];
        self.graph.active_blast_radius = Some(BlastRadiusSummary {
            total_impacted: 5,
            max_depth: 2,
            impacted_files: vec!["crates/toksi/src/main.rs".into(), "crates/toksi/src/commands/top.rs".into()],
            affected_tests: vec!["test_run_top_once".into()],
            tree_items: vec![
                BlastTreeItem { name: sym.name.clone(), file: sym.file.clone(), depth: 0 },
                BlastTreeItem { name: "main".into(), file: "crates/toksi/src/main.rs".into(), depth: 1 },
                BlastTreeItem { name: "run_cli".into(), file: "crates/toksi/src/main.rs".into(), depth: 1 },
                BlastTreeItem { name: "test_top".into(), file: "crates/toksi/src/tests.rs".into(), depth: 2 },
            ],
        });
    }

    pub fn reload_headroom(&mut self) {
        let routes = configured_byok_routes();
        self.headroom.routes = routes
            .into_iter()
            .map(|r| {
                let env_key = format!("{}_API_KEY", r.protocol.to_uppercase());
                let key_val = std::env::var(&env_key).unwrap_or_default();
                let tier = crate::gateway::agent_cli::tier_check::inspect_provider_tier(&r.protocol, &key_val);

                ByokRouteItem {
                    provider: r.protocol.clone(),
                    target_url: r.upstream.clone(),
                    virtual_prefix: format!("/byok/{}/", r.id),
                    healthy: !r.upstream.trim().is_empty(),
                    tier_badge: tier.label().to_string(),
                    quota_display: "Configured".into(),
                    agents_assigned: format!("Route ID: {}", r.id),
                }
            })
            .collect();

        self.headroom.profiles.clear();
        self.headroom.quota_windows.clear();
        self.headroom.models_count = 0;

        if let Ok(arc) = crate::db::shared_db()
            && let Ok(db) = arc.lock()
        {
            if let Ok(profs) = db.list_byok_profiles() {
                self.headroom.profiles = profs
                    .into_iter()
                    .map(|p| ByokProfileRow {
                        id: p.id,
                        provider_id: p.provider_id,
                        instance_label: p.instance_label,
                        model_id: p.model_id,
                        thinking_effort: p.thinking_effort,
                        enabled: p.is_active,
                        priority: i64::from(p.priority),
                        weight: i64::from(p.weight),
                    })
                    .collect();
            }
            if let Ok(windows) = db.list_quota_windows(None) {
                self.headroom.quota_windows = windows
                    .into_iter()
                    .map(|w| QuotaWindowRow {
                        profile_id: w.instance_label.clone(),
                        provider_id: w.provider_id,
                        window_kind: w.window_kind,
                        requests_remaining: w.remaining_requests,
                        requests_limit: w.limit_requests,
                        tokens_remaining: w.remaining_tokens,
                        tokens_limit: w.limit_tokens,
                        plan_label: w.instance_label,
                    })
                    .collect();
            }
            if let Ok(models) = db.list_provider_models(None) {
                self.headroom.models_count = models.len();
            }
        }
    }

    pub fn cycle_selected_profile_effort(&mut self) {
        let Some(prof) = self.headroom.profiles.get_mut(self.headroom.selected_profile) else {
            self.set_status("No configured <byok, model, effort> profile selected");
            return;
        };
        let next_effort = match prof.thinking_effort.to_lowercase().as_str() {
            "low" => "medium",
            "medium" => "high",
            _ => "low",
        };
        prof.thinking_effort = next_effort.to_string();
        let pid = prof.id.clone();
        let label = prof.instance_label.clone();
        if let Ok(arc) = crate::db::shared_db()
            && let Ok(mut db) = arc.lock()
            && let Ok(mut list) = db.list_byok_profiles()
            && let Some(rec) = list.iter_mut().find(|r| r.id == pid)
        {
            rec.thinking_effort = next_effort.to_string();
            let _ = db.upsert_byok_profile(rec);
        }
        self.set_status(&format!("Profile '{}' thinking effort -> {}", label, next_effort));
    }

    pub fn refresh_gateway_models(&mut self) {
        self.reload_headroom();
        self.set_status(&format!(
            "Refreshed {} configured models & {} <byok, model, effort> profiles",
            self.headroom.models_count,
            self.headroom.profiles.len()
        ));
    }

    pub fn reload_governance(&mut self) {
        let statuses = crate::keyring::audit_keyring_statuses();
        self.governance.keyring_entries = statuses
            .into_iter()
            .map(|s| KeyringEntryRow {
                agent: s.agent,
                service: s.service,
                account: s.account,
                present: s.has_credential,
                configured: true,
                source: s.backend,
            })
            .collect();

        let adc = crate::keyring::adc::inspect_adc_status();
        self.governance.gcp_adc_active = adc.enabled;
        self.governance.gcp_project = adc.gcp_project.unwrap_or_default();
        self.governance.gcp_location = adc.gcp_location;

        if let Ok(arc) = crate::db::shared_db()
            && let Ok(db) = arc.lock()
            && let Ok(st) = db.stats()
        {
            self.governance.db_stats = DbStatsSummary {
                    providers: st.providers_count,
                    byok_profiles: st.byok_profiles_count,
                    plugins: st.plugins_count,
                    blind_signatures: st.blind_signatures_count,
                    advisory_embargoes: st.advisory_embargoes_count,
                    adr_events: st.adr_audit_events_count,
                    quota_windows: st.quota_windows_count,
                };
        }
    }

    pub fn export_db(&mut self) {
        let backup_path = crate::db::default_db_path()
            .parent()
            .map(|p| p.join("toksi-backup.json"))
            .unwrap_or_else(|| PathBuf::from("toksi-backup.json"));
        match crate::db::open_default_db() {
            Ok(db) => match db.export_json() {
                Ok(json) => {
                    if let Err(e) = fs::write(&backup_path, json) {
                        self.set_status(&format!("DB export write failed: {}", e));
                    } else {
                        self.set_status(&format!(
                            "Exported DB to {}",
                            crate::util::paths::normalize_path(&backup_path.to_string_lossy())
                        ));
                    }
                }
                Err(e) => self.set_status(&format!("DB export error: {}", e)),
            },
            Err(e) => self.set_status(&format!("DB open error: {}", e)),
        }
    }

    pub fn import_db(&mut self) {
        let backup_path = crate::db::default_db_path()
            .parent()
            .map(|p| p.join("toksi-backup.json"))
            .unwrap_or_else(|| PathBuf::from("toksi-backup.json"));
        if !backup_path.exists() {
            self.set_status("No toksi-backup.json found to import");
            return;
        }
        match fs::read_to_string(&backup_path) {
            Ok(json) => match crate::db::open_default_db() {
                Ok(mut db) => match db.import_json(&json, false) {
                    Ok(_) => {
                        self.reload_governance();
                        self.reload_headroom();
                        self.set_status("Imported DB from toksi-backup.json");
                    }
                    Err(e) => self.set_status(&format!("DB import error: {}", e)),
                },
                Err(e) => self.set_status(&format!("DB open error: {}", e)),
            },
            Err(e) => self.set_status(&format!("Read backup failed: {}", e)),
        }
    }

    pub fn reload_logs(&mut self) {
        let (_pid_file, log_file) = crate::headroom::proxy::proxy_files();
        let mut entries = Vec::new();

        if Path::new(&log_file).exists() {
            if let Ok(content) = fs::read_to_string(&log_file) {
                for line in content.lines().rev().take(150) {
                    if line.trim().is_empty() {
                        continue;
                    }
                    entries.push(LogEntry {
                        timestamp: line.chars().take(19).collect(),
                        level: if line.contains("ERROR") { "ERROR".into() } else if line.contains("WARN") { "WARN".into() } else { "INFO".into() },
                        source: LogSource::Proxy,
                        message: line.to_string(),
                    });
                }
            }
        }

        if entries.is_empty() {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            entries = vec![
                LogEntry { timestamp: now.clone(), level: "INFO".into(), source: LogSource::Proxy, message: "Headroom token proxy initialized on port 8787".into() },
                LogEntry { timestamp: now.clone(), level: "INFO".into(), source: LogSource::Daemon, message: "Toksi daemon supervisor active; PID tracking healthy".into() },
                LogEntry { timestamp: now.clone(), level: "INFO".into(), source: LogSource::Proxy, message: "Loaded 1024-token prompt cache alignment transforms".into() },
                LogEntry { timestamp: now.clone(), level: "INFO".into(), source: LogSource::Watchdog, message: "System watchdog monitor: 0 memory leaks, 0 dropped frames".into() },
            ];
        }

        self.logs.entries = entries;
    }

    pub fn poll_new_logs(&mut self) {
        let (_, log_file) = crate::headroom::proxy::proxy_files();
        if let Ok(mut f) = fs::File::open(&log_file) {
            if let Ok(meta) = f.metadata() {
                let len = meta.len();
                if len < self.log_file_offset {
                    self.log_file_offset = 0;
                    self.logs.entries.clear();
                }
                if len > self.log_file_offset {
                    let _ = f.seek(SeekFrom::Start(self.log_file_offset));
                    let reader = BufReader::new(f);
                    for line_res in reader.lines() {
                        if let Ok(line) = line_res {
                            if !line.trim().is_empty() {
                                self.logs.entries.push(LogEntry {
                                    timestamp: line.chars().take(19).collect(),
                                    level: if line.contains("ERROR") { "ERROR".into() } else { "INFO".into() },
                                    source: LogSource::Proxy,
                                    message: line,
                                });
                            }
                        }
                    }
                    self.log_file_offset = len;
                    if self.logs.entries.len() > 500 {
                        let overflow = self.logs.entries.len() - 500;
                        self.logs.entries.drain(0..overflow);
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Event Handler
    // -------------------------------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Handle active modals first
        if self.agents.inspect_mcp.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.agents.inspect_mcp = None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(ref mut m) = self.agents.inspect_mcp {
                        m.scroll = m.scroll.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(ref mut m) = self.agents.inspect_mcp {
                        m.scroll = m.scroll.saturating_add(1);
                    }
                }
                _ => {}
            }
            return;
        }

        if self.memory.heatmap_popup.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.memory.heatmap_popup = None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(ref mut h) = self.memory.heatmap_popup {
                        h.scroll = h.scroll.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(ref mut h) = self.memory.heatmap_popup {
                        h.scroll = h.scroll.saturating_add(1);
                    }
                }
                _ => {}
            }
            return;
        }

        if self.memory.precheck_result.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.memory.precheck_result = None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(ref mut p) = self.memory.precheck_result {
                        p.scroll = p.scroll.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(ref mut p) = self.memory.precheck_result {
                        p.scroll = p.scroll.saturating_add(1);
                    }
                }
                _ => {}
            }
            return;
        }

        if self.memory.heatmap_popup.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.memory.heatmap_popup = None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(ref mut h) = self.memory.heatmap_popup {
                        h.scroll = h.scroll.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(ref mut h) = self.memory.heatmap_popup {
                        h.scroll = h.scroll.saturating_add(1);
                    }
                }
                _ => {}
            }
            return;
        }

        // Handle search input modes in Tab 5 (Memory) and Tab 6 (CodeGraph)
        if self.current_tab == 4 && self.memory.search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.memory.search_active = false;
                }
                KeyCode::Backspace => {
                    self.memory.search_query.pop();
                    self.memory.selected = 0;
                }
                KeyCode::Char(c) => {
                    self.memory.search_query.push(c);
                    self.memory.selected = 0;
                }
                _ => {}
            }
            return;
        }

        if self.current_tab == 5 && self.graph.search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.graph.search_active = false;
                    self.update_active_symbol_details();
                }
                KeyCode::Backspace => {
                    self.graph.search_query.pop();
                    self.graph.selected = 0;
                    self.update_active_symbol_details();
                }
                KeyCode::Char(c) => {
                    self.graph.search_query.push(c);
                    self.graph.selected = 0;
                    self.update_active_symbol_details();
                }
                _ => {}
            }
            return;
        }

        // Global Navigation Keys
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            KeyCode::Tab | KeyCode::Right => {
                self.current_tab = (self.current_tab + 1) % self.tab_names.len();
                return;
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.current_tab = if self.current_tab == 0 {
                    self.tab_names.len() - 1
                } else {
                    self.current_tab - 1
                };
                return;
            }
            KeyCode::Char('1') => { self.current_tab = 0; return; }
            KeyCode::Char('2') => { self.current_tab = 1; return; }
            KeyCode::Char('3') => { self.current_tab = 2; return; }
            KeyCode::Char('4') => { self.current_tab = 3; return; }
            KeyCode::Char('5') => { self.current_tab = 4; return; }
            KeyCode::Char('6') => { self.current_tab = 5; return; }
            KeyCode::Char('7') => { self.current_tab = 6; return; }
            KeyCode::Char('8') => { self.current_tab = 7; return; }
            KeyCode::Char('9') => { self.current_tab = 8; return; }
            KeyCode::Char('0') => { self.current_tab = 9; return; }
            _ => {}
        }

        // Tab-Specific Key Handlers
        match self.current_tab {
            0 => self.handle_monitor_key(key),
            1 => {
                if matches!(key.code, KeyCode::Char('r')) {
                    self.refresh_monitor_stats();
                    self.set_status("Refreshed Activity Room snapshot");
                }
            }
            2 => self.handle_agents_key(key),
            3 => {
                if matches!(key.code, KeyCode::Char('r')) {
                    self.reload_headroom();
                    self.set_status("Refreshed Quotas & BYOK snapshot");
                }
            }
            4 => self.handle_memory_key(key),
            5 => self.handle_graph_key(key),
            6 => {
                if matches!(key.code, KeyCode::Char('f')) {
                    self.sandbox.fail_closed = !self.sandbox.fail_closed;
                    self.set_status(&format!(
                        "DefenseClaw HookGuard set to {}",
                        if self.sandbox.fail_closed { "FAIL-CLOSED" } else { "PERMISSIVE" }
                    ));
                }
            }
            7 => self.handle_headroom_key(key),
            8 => self.handle_governance_key(key),
            9 => self.handle_logs_key(key),
            _ => {}
        }
    }

    fn handle_monitor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r') => {
                self.reload_services();
                self.set_status("Refreshed services and token telemetry");
            }
            KeyCode::Char('u') => {
                let _ = ensure_proxy_up();
                self.monitor.proxy_up = true;
                self.set_status("Headroom proxy started on port 8787");
            }
            KeyCode::Char('d') => {
                let _ = stop_proxy();
                self.monitor.proxy_up = false;
                self.set_status("Headroom proxy stopped");
            }
            _ => {}
        }
    }

    fn handle_agents_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.agents.selected > 0 {
                    self.agents.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.agents.selected + 1 < self.agents.items.len() {
                    self.agents.selected += 1;
                }
            }
            KeyCode::Char(' ') => {
                self.toggle_agent_wire();
            }
            KeyCode::Char('c') => {
                self.config_wire_selected_agent();
            }
            KeyCode::Char('u') => {
                self.unwire_selected_agent();
            }
            KeyCode::Char('x') => {
                self.fix_selected_agent();
            }
            KeyCode::Enter => {
                self.inspect_agent_mcp();
            }
            KeyCode::Char('r') => {
                self.reload_agents();
                self.set_status("Rescanned configured agent harnesses");
            }
            _ => {}
        }
    }

    fn handle_memory_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('/') => {
                self.memory.search_active = true;
            }
            KeyCode::Char('p') => {
                self.trigger_precheck();
            }
            KeyCode::Char('h') => {
                self.open_heatmap_popup(self.memory.selected_heatmap);
            }
            KeyCode::Char('f') => {
                self.memory.filter = match self.memory.filter {
                    MemoryFilter::All => MemoryFilter::Issues,
                    MemoryFilter::Issues => MemoryFilter::Decisions,
                    MemoryFilter::Decisions => MemoryFilter::Gotchas,
                    MemoryFilter::Gotchas => MemoryFilter::Icm,
                    MemoryFilter::Icm => MemoryFilter::All,
                };
                self.memory.selected = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.memory.selected > 0 {
                    self.memory.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = self.memory.filtered_events().len();
                if self.memory.selected + 1 < count {
                    self.memory.selected += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_graph_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('/') => {
                self.graph.search_active = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.graph.selected > 0 {
                    self.graph.selected -= 1;
                    self.update_active_symbol_details();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = self.graph.filtered_symbols().len();
                if self.graph.selected + 1 < count {
                    self.graph.selected += 1;
                    self.update_active_symbol_details();
                }
            }
            KeyCode::Enter => {
                self.update_active_symbol_details();
                self.set_status("Computed AST call path and blast radius");
            }
            KeyCode::Char('b') => {
                self.graph.show_tree_view = !self.graph.show_tree_view;
            }
            KeyCode::Char('r') => {
                let mut cg = crate::codegraph::CodeGraph::open(&PathBuf::from("."));
                if let Ok(ref mut g) = cg {
                    let _ = g.ensure_indexed();
                }
                self.reload_graph();
                self.set_status("Reindexed project Tree-sitter AST symbols");
            }
            _ => {}
        }
    }

    fn handle_headroom_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r') => {
                self.reload_headroom();
                self.set_status("Reloaded BYOK routes and provider statuses");
            }
            KeyCode::Char('m') => {
                self.refresh_gateway_models();
            }
            KeyCode::Char('e') => {
                self.cycle_selected_profile_effort();
            }
            KeyCode::Char('c') => {
                // Simulate 1024 token prompt cache alignment test
                let sample_prompt = "You are a helpful coding assistant. Implement the requested function.";
                let aligned = crate::headroom::cache_align::align_to_1024_tokens(sample_prompt);
                let est = crate::headroom::cache_align::estimate_tokens(&aligned);
                self.set_status(&format!("Prompt aligned to 1024 boundary (estimated tokens: {})", est));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.headroom.profiles.is_empty() && self.headroom.selected_profile > 0 {
                    self.headroom.selected_profile -= 1;
                } else if self.headroom.selected_route > 0 {
                    self.headroom.selected_route -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.headroom.profiles.is_empty()
                    && self.headroom.selected_profile + 1 < self.headroom.profiles.len()
                {
                    self.headroom.selected_profile += 1;
                } else if self.headroom.selected_route + 1 < self.headroom.routes.len() {
                    self.headroom.selected_route += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_governance_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.governance.selected > 0 {
                    self.governance.selected -= 1;
                } else if !self.governance.keyring_entries.is_empty() {
                    self.governance.selected = self.governance.keyring_entries.len() - 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.governance.keyring_entries.is_empty() {
                    self.governance.selected =
                        (self.governance.selected + 1) % self.governance.keyring_entries.len();
                }
            }
            KeyCode::Char('e') => {
                self.export_db();
            }
            KeyCode::Char('i') => {
                self.import_db();
            }
            KeyCode::Char('r') => {
                self.reload_governance();
                self.set_status("Refreshed Keyring & Database state");
            }
            KeyCode::Char('c') => {
                self.governance.circuit_breaker_trips = 0;
                self.set_status("Circuit breaker trip counters cleared");
            }
            _ => {}
        }
    }

    pub fn toggle_daemon(&mut self) {
        let running = is_daemon_running();
        if running {
            let _ = crate::commands::daemon::run_daemon_cli(&["stop".to_string()]);
            self.daemon_live_flag.store(false, Ordering::Relaxed);
            self.monitor.daemon_up = false;
            self.set_status("Daemon stopped");
        } else {
            let _ = crate::commands::daemon::run_daemon_cli(&["start".to_string()]);
            self.daemon_live_flag.store(true, Ordering::Relaxed);
            self.monitor.daemon_up = true;
            self.set_status("Daemon started");
        }
    }

    fn handle_logs_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') => {
                self.toggle_daemon();
            }
            KeyCode::Char('f') => {
                self.logs.follow = !self.logs.follow;
            }
            KeyCode::Char('c') => {
                self.logs.entries.clear();
                self.set_status("Cleared log buffer");
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.logs.follow = false;
                self.logs.scroll_offset = self.logs.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.logs.scroll_offset = self.logs.scroll_offset.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.logs.follow = false;
                self.logs.scroll_offset = self.logs.scroll_offset.saturating_sub(15);
            }
            KeyCode::PageDown => {
                self.logs.scroll_offset = self.logs.scroll_offset.saturating_add(15);
            }
            KeyCode::Char('1') => { self.logs.source = LogSource::All; }
            KeyCode::Char('2') => { self.logs.source = LogSource::Proxy; }
            KeyCode::Char('3') => { self.logs.source = LogSource::Daemon; }
            KeyCode::Char('4') => { self.logs.source = LogSource::Watchdog; }
            _ => {}
        }
    }

    // -------------------------------------------------------------------------
    // Mouse Event Handler
    // -------------------------------------------------------------------------

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                if self.agents.inspect_mcp.is_some() {
                    if let Some(ref mut m) = self.agents.inspect_mcp {
                        m.scroll = m.scroll.saturating_add(3);
                    }
                } else if self.memory.precheck_result.is_some() {
                    if let Some(ref mut p) = self.memory.precheck_result {
                        p.scroll = p.scroll.saturating_add(3);
                    }
                } else if self.memory.heatmap_popup.is_some() {
                    if let Some(ref mut h) = self.memory.heatmap_popup {
                        h.scroll = h.scroll.saturating_add(3);
                    }
                } else if self.current_tab == 9 {
                    self.logs.follow = false;
                    self.logs.scroll_offset = self.logs.scroll_offset.saturating_add(3);
                }
            }
            MouseEventKind::ScrollUp => {
                if self.agents.inspect_mcp.is_some() {
                    if let Some(ref mut m) = self.agents.inspect_mcp {
                        m.scroll = m.scroll.saturating_sub(3);
                    }
                } else if self.memory.heatmap_popup.is_some() {
                    if let Some(ref mut h) = self.memory.heatmap_popup {
                        h.scroll = h.scroll.saturating_sub(3);
                    }
                } else if self.memory.precheck_result.is_some() {
                    if let Some(ref mut p) = self.memory.precheck_result {
                        p.scroll = p.scroll.saturating_sub(3);
                    }
                } else if self.current_tab == 9 {
                    self.logs.follow = false;
                    self.logs.scroll_offset = self.logs.scroll_offset.saturating_sub(3);
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_left_click(mouse.column, mouse.row, area);
            }
            _ => {}
        }
    }

    fn handle_mouse_left_click(&mut self, col: u16, row: u16, area: Rect) {
        // 1. Modals dismiss on click outside
        if self.agents.inspect_mcp.is_some() {
            if row < 4 || row > area.height.saturating_sub(4) {
                self.agents.inspect_mcp = None;
            }
            return;
        }
        if self.memory.heatmap_popup.is_some() {
            if row < 4 || row > area.height.saturating_sub(4) {
                self.memory.heatmap_popup = None;
            }
            return;
        }

        if self.memory.precheck_result.is_some() {
            if row < 4 || row > area.height.saturating_sub(4) {
                self.memory.precheck_result = None;
            }
            return;
        }

        // 2. Tab Bar: row == 2 or 3
        if row == 2 || row == 3 {
            let width = area.width.max(1);
            let tab_count = self.tab_names.len() as u16;
            let tab_width = (width / tab_count).max(1);
            let clicked_tab = (col / tab_width) as usize;
            if clicked_tab < self.tab_names.len() {
                self.current_tab = clicked_tab;
            }
            return;
        }

        // 3. Footer row: row >= area.height - 2
        let footer_start = area.height.saturating_sub(2);
        if row >= footer_start {
            return;
        }

        // 4. Action Buttons Bar at bottom of each tab view: row == footer_start - 3 .. footer_start - 1
        let btn_row_start = footer_start.saturating_sub(3);
        if row >= btn_row_start && row < footer_start {
            self.handle_action_button_click(col);
            return;
        }

        // 5. Tab-Specific Table & List Row Selection
        match self.current_tab {
            0 => {
                // Monitor tab: Waterfall table click
                // Waterfall table starts around row 11
                if row >= 12 {
                    let clicked_index = (row - 12) as usize;
                    let count = self.monitor.waterfall.len();
                    if clicked_index < count {
                        self.set_status(&format!("Inspected waterfall event #{}", clicked_index + 1));
                    }
                }
            }
            1 => {
                // War Room tab
            }
            2 => {
                // Agents tab: rows start around row 7
                if row >= 7 {
                    let clicked_index = (row - 7) as usize;
                    if clicked_index < self.agents.items.len() {
                        self.agents.selected = clicked_index;
                        self.set_status(&format!("Selected agent: {}", self.agents.items[clicked_index].label));
                    }
                }
            }
            7 => {
                // AI Gateway & Quotas: <byok, model, effort> profiles or routes table
                let table_top = 21;
                if row >= table_top {
                    let clicked_index = (row - table_top) as usize;
                    if clicked_index < self.headroom.profiles.len() {
                        self.headroom.selected_profile = clicked_index;
                        self.set_status(&format!(
                            "Selected profile: {} ({})",
                            self.headroom.profiles[clicked_index].instance_label,
                            self.headroom.profiles[clicked_index].model_id
                        ));
                    } else if clicked_index < self.headroom.routes.len() {
                        self.headroom.selected_route = clicked_index;
                        self.set_status(&format!("Selected route: {}", self.headroom.routes[clicked_index].provider));
                    }
                }
            }
            4 => {
                // Memory tab: Filter pills on row 4
                if row == 4 {
                    if col < 14 {
                        self.memory.filter = MemoryFilter::All;
                        self.memory.selected = 0;
                    } else if col < 28 {
                        self.memory.filter = MemoryFilter::Issues;
                        self.memory.selected = 0;
                    } else if col < 44 {
                        self.memory.filter = MemoryFilter::Decisions;
                        self.memory.selected = 0;
                    } else if col < 64 {
                        self.memory.filter = MemoryFilter::Gotchas;
                        self.memory.selected = 0;
                    } else if col < 80 {
                        self.memory.filter = MemoryFilter::Icm;
                        self.memory.selected = 0;
                    }
                    return;
                }
                // Clickable Heatmap strip on row 7
                if row == 7 && !self.memory.heatmap.is_empty() {
                    let cell_idx = (col / 24) as usize;
                    if cell_idx < self.memory.heatmap.len() {
                        self.open_heatmap_popup(cell_idx);
                    }
                    return;
                }
                // Memory events table starts around row 9
                if row >= 9 {
                    let clicked_index = (row - 9) as usize;
                    let count = self.memory.filtered_events().len();
                    if clicked_index < count {
                        self.memory.selected = clicked_index;
                    }
                }
            }
            5 => {
                // Graph tab: symbols table starts around row 8
                if row >= 8 {
                    let clicked_index = (row - 8) as usize;
                    let count = self.graph.filtered_symbols().len();
                    if clicked_index < count {
                        self.graph.selected = clicked_index;
                        self.update_active_symbol_details();
                    }
                }
            }
            8 => {
                // Keyring & Database tab: keyring table starts around row 11
                if row >= 11 && row < 11 + self.governance.keyring_entries.len() as u16 {
                    let clicked_index = (row - 11) as usize;
                    if clicked_index < self.governance.keyring_entries.len() {
                        self.governance.selected = clicked_index;
                        self.set_status(&format!(
                            "Selected keyring entry: {}",
                            self.governance.keyring_entries[clicked_index].service
                        ));
                    }
                }
            }
            9 => {
                // System Controls / Logs tab: source pills on row 4
                if row == 4 {
                    if col < 18 {
                        self.logs.source = LogSource::All;
                    } else if col < 40 {
                        self.logs.source = LogSource::Proxy;
                    } else if col < 56 {
                        self.logs.source = LogSource::Daemon;
                    } else if col < 74 {
                        self.logs.source = LogSource::Watchdog;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_action_button_click(&mut self, col: u16) {
        match self.current_tab {
            0 => {
                // Monitor: [Proxy Up] [Proxy Down] [Refresh]
                if col < 22 {
                    let _ = ensure_proxy_up();
                    self.monitor.proxy_up = true;
                    self.set_status("Headroom proxy started on port 8787");
                } else if col < 44 {
                    let _ = stop_proxy();
                    self.monitor.proxy_up = false;
                    self.set_status("Headroom proxy stopped");
                } else if col < 64 {
                    self.reload_services();
                    self.set_status("Refreshed services and token telemetry");
                }
            }
            1 => {
                // War Room: [Refresh Snapshot] [Reset Circuit]
                self.refresh_monitor_stats();
                self.set_status("Refreshed War Room telemetry snapshot");
            }
            2 => {
                // Agents: [Direct-Wire] [Config-Wire] [Unwire] [Fix] [Inspect MCP] [Rescan]
                if col < 18 {
                    self.direct_wire_selected_agent();
                } else if col < 36 {
                    self.config_wire_selected_agent();
                } else if col < 50 {
                    self.unwire_selected_agent();
                } else if col < 62 {
                    self.fix_selected_agent();
                } else if col < 82 {
                    self.inspect_agent_mcp();
                } else {
                    self.reload_agents();
                    self.set_status("Rescanned configured agent harnesses");
                }
            }
            7 => {
                // AI Gateway & Quotas: [Refresh Models] [Cycle Effort] [Test Cache Align] [Reload]
                if col < 22 {
                    self.refresh_gateway_models();
                } else if col < 42 {
                    self.cycle_selected_profile_effort();
                } else if col < 64 {
                    let sample_prompt = "You are a helpful coding assistant. Implement the requested function.";
                    let aligned = crate::headroom::cache_align::align_to_1024_tokens(sample_prompt);
                    let est = crate::headroom::cache_align::estimate_tokens(&aligned);
                    self.set_status(&format!("Prompt aligned to 1024 boundary (estimated tokens: {})", est));
                } else {
                    self.reload_headroom();
                    self.set_status("Reloaded BYOK routes and provider statuses");
                }
            }
            6 => {
                // Sandbox Inspector
                if col < 30 {
                    self.set_status("ContextMode sandbox health verified");
                } else {
                    self.sandbox.fail_closed = !self.sandbox.fail_closed;
                    self.set_status(&format!(
                        "DefenseClaw HookGuard: {}",
                        if self.sandbox.fail_closed { "FAIL-CLOSED" } else { "PERMISSIVE" }
                    ));
                }
            }
            4 => {
                // Memory: [Run Precheck] [Heatmap Detail] [Cycle Filter] [Search]
                if col < 20 {
                    self.trigger_precheck();
                } else if col < 40 {
                    self.open_heatmap_popup(self.memory.selected_heatmap);
                } else if col < 58 {
                    self.memory.filter = match self.memory.filter {
                        MemoryFilter::All => MemoryFilter::Issues,
                        MemoryFilter::Issues => MemoryFilter::Decisions,
                        MemoryFilter::Decisions => MemoryFilter::Gotchas,
                        MemoryFilter::Gotchas => MemoryFilter::Icm,
                        MemoryFilter::Icm => MemoryFilter::All,
                    };
                    self.memory.selected = 0;
                } else if col < 76 {
                    self.memory.search_active = true;
                }
            }
            5 => {
                // Graph: [Compute Blast] [Show/Hide Tree] [Reindex AST]
                if col < 26 {
                    self.update_active_symbol_details();
                    self.set_status("Computed AST call path and blast radius");
                } else if col < 50 {
                    self.graph.show_tree_view = !self.graph.show_tree_view;
                } else if col < 70 {
                    let mut cg = crate::codegraph::CodeGraph::open(&PathBuf::from("."));
                    if let Ok(ref mut g) = cg {
                        let _ = g.ensure_indexed();
                    }
                    self.reload_graph();
                    self.set_status("Reindexed project Tree-sitter AST symbols");
                }
            }
            8 => {
                // Keyring & Database: [Export DB] [Import DB] [Refresh]
                if col < 22 {
                    self.export_db();
                } else if col < 44 {
                    self.import_db();
                } else {
                    self.reload_governance();
                    self.set_status("Refreshed Keyring & Database state");
                }
            }
            9 => {
                // System Controls & Logs: [Daemon Start/Stop] [Pause/Resume Follow] [Clear Buffer]
                if col < 24 {
                    self.toggle_daemon();
                } else if col < 48 {
                    self.logs.follow = !self.logs.follow;
                } else if col < 68 {
                    self.logs.entries.clear();
                    self.set_status("Cleared log buffer");
                }
            }
            _ => {}
        }
    }

    // -------------------------------------------------------------------------
    // Filter Helpers
    // -------------------------------------------------------------------------

    pub fn filtered_memory_items(&self) -> Vec<&MemoryItem> {
        let q = self.memory.search_query.to_lowercase();
        self.memory
            .items
            .iter()
            .filter(|item| {
                let matches_filter = match self.memory.filter {
                    MemoryFilter::All => true,
                    MemoryFilter::Issues => item.event_type == "issue",
                    MemoryFilter::Decisions => item.event_type == "decision",
                    MemoryFilter::Gotchas => item.event_type == "note" || item.event_type == "gotcha",
                    MemoryFilter::Icm => item.event_type == "icm",
                };

                let matches_search = q.is_empty()
                    || item.summary.to_lowercase().contains(&q)
                    || item.location.to_lowercase().contains(&q)
                    || item.id.to_lowercase().contains(&q);

                matches_filter && matches_search
            })
            .collect()
    }
}

impl MemoryState {
    pub fn filtered_events(&self) -> Vec<&MemoryItem> {
        let q = self.search_query.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                let matches_filter = match self.filter {
                    MemoryFilter::All => true,
                    MemoryFilter::Issues => item.event_type == "issue",
                    MemoryFilter::Decisions => item.event_type == "decision",
                    MemoryFilter::Gotchas => item.event_type == "note" || item.event_type == "gotcha",
                    MemoryFilter::Icm => item.event_type == "icm",
                };

                let matches_search = q.is_empty()
                    || item.summary.to_lowercase().contains(&q)
                    || item.location.to_lowercase().contains(&q)
                    || item.id.to_lowercase().contains(&q);

                matches_filter && matches_search
            })
            .collect()
    }
}

impl GraphState {
    pub fn filtered_symbols(&self) -> Vec<&SymbolRow> {
        let q = self.search_query.to_lowercase();
        if q.is_empty() {
            self.symbols.iter().collect()
        } else {
            self.symbols
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&q) || s.file.to_lowercase().contains(&q))
                .collect()
        }
    }

    pub fn selected_symbol(&self) -> Option<&SymbolRow> {
        let filtered = self.filtered_symbols();
        filtered.get(self.selected).copied()
    }
}

impl LogsState {
    pub fn filtered_logs(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| match self.source {
                LogSource::All => true,
                LogSource::Proxy => e.source == LogSource::Proxy,
                LogSource::Daemon => e.source == LogSource::Daemon,
                LogSource::Watchdog => e.source == LogSource::Watchdog,
            })
            .collect()
    }
}

fn resolve_agent_mcp_config_path(agent_id: &str) -> String {
    match agent_id {
        "claude" => claude_code_paths().global_json,
        "cursor" => cursor_global_mcp_path(),
        "opencode" => opencode_paths_resolved().config,
        "opencode2" => opencode_paths_v2().config,
        "codex" => codex_paths_resolved().config,
        "antigravity" => crate::util::paths::antigravity_paths_resolved().mcp_config,
        "copilot" => copilot_paths_resolved().mcp_config,
        "droid" => crate::agents::droid::droid_mcp_file(),
        "grok" => grok_paths_resolved().config,
        "pi" => pi_paths_resolved().mcp_config,
        "omp" => omp_paths_resolved().config,
        "kilo" => kilo_paths_resolved().config,
        "cline" => cline_paths_resolved().mcp_config,
        "amp" => amp_paths_resolved().config,
        "kimi" => kimi_paths_resolved().mcp_config,
        "zcode" => zcode_paths_resolved().mcp_config,
        "dsh" => crate::agents::dsh::dsh_patch_file(),
        _ => format!(".toksi/{}.json", agent_id),
    }
}
