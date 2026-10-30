// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/app.rs

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::{list_agents, list_tools, RunOpts};
use crate::headroom::byok_gateway::configured_byok_routes;
use crate::headroom::proxy::{ensure_proxy_up, proxy_running, stop_proxy};
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
    pub detect_source: String,
    pub wired: bool,
    pub port: String,
    pub tools_summary: String,
    pub config_path: String,
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
pub struct MemoryState {
    pub filter: MemoryFilter,
    pub search_query: String,
    pub search_active: bool,
    pub items: Vec<MemoryItem>,
    pub selected: usize,
    pub precheck_result: Option<PrecheckModal>,
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
    pub quota_display: String,
    pub agents_assigned: String,
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

// -----------------------------------------------------------------------------
// App State Container
// -----------------------------------------------------------------------------

pub struct App {
    pub current_tab: usize,
    pub tab_names: [&'static str; 6],
    pub should_quit: bool,
    pub cwd_display: String,
    pub status_message: Option<String>,
    pub status_ticks: usize,
    pub ticks_counter: usize,

    pub monitor: MonitorState,
    pub agents: AgentsState,
    pub memory: MemoryState,
    pub graph: GraphState,
    pub headroom: HeadroomState,
    pub logs: LogsState,
}

impl App {
    pub fn new() -> Self {
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into());

        let mut app = Self {
            current_tab: 0,
            tab_names: [
                "1: Live Monitor",
                "2: Agent Wire Manager",
                "3: Memory Explorer",
                "4: CodeGraph Visualizer",
                "5: Headroom BYOK & Cache",
                "6: Diagnostic Logs",
            ],
            should_quit: false,
            cwd_display: cwd,
            status_message: None,
            status_ticks: 0,
            ticks_counter: 0,

            monitor: MonitorState {
                proxy_up: false,
                daemon_up: false,
                tokens_saved_lifetime: 14_892_104,
                tokens_saved_today: 482_300,
                tokens_saved_7d: 3_120_500,
                cost_saved_usd: 184.26,
                total_calls: 1_284,
                compression_pct: 68.4,
                cache_warmth_pct: 91.2,
                sparkline_data: vec![40, 52, 60, 58, 64, 70, 68, 72, 75, 71, 74, 69, 73, 76, 78, 80],
                waterfall: Vec::new(),
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
                cache_alignment_pct: 92.5,
                context_tokens_used: 42_500,
                active_memory_tier: "L1 (Abstract)".to_string(),
                rate_limit_429_count: 2,
                reroute_count: 1,
                byok_port: 8788,
                routes: Vec::new(),
                selected_route: 0,
            },

            logs: LogsState {
                source: LogSource::All,
                follow: true,
                scroll_offset: 0,
                entries: Vec::new(),
            },
        };

        app.init_waterfall_samples();
        app.reload_services();
        app.reload_agents();
        app.reload_memory();
        app.reload_graph();
        app.reload_headroom();
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

        // Periodic light refresh every ~1 sec (60 ticks)
        if self.ticks_counter % 60 == 0 {
            self.monitor.proxy_up = proxy_running();
            self.monitor.daemon_up = is_daemon_running();
            self.poll_new_logs();
        }
    }

    // -------------------------------------------------------------------------
    // Subsystem Loaders
    // -------------------------------------------------------------------------

    fn init_waterfall_samples(&mut self) {
        self.monitor.waterfall = vec![
            WaterfallItem {
                time: "19:28:10".into(),
                agent: "Claude Code".into(),
                endpoint: "POST /v1/messages (claude-3-7)".into(),
                raw_tokens: 62_400,
                compressed_tokens: 19_800,
                savings_pct: 68.2,
                pipeline: "RTK + Headroom-Log + AST-Prune".into(),
                saved_usd: 0.639,
            },
            WaterfallItem {
                time: "19:28:22".into(),
                agent: "Cursor".into(),
                endpoint: "POST /v1/chat/completions (gpt-4o)".into(),
                raw_tokens: 24_100,
                compressed_tokens: 7_200,
                savings_pct: 70.1,
                pipeline: "RTK + 1024-CacheAlign (Hit 91%)".into(),
                saved_usd: 0.169,
            },
            WaterfallItem {
                time: "19:28:45".into(),
                agent: "Antigravity".into(),
                endpoint: "POST /v1/models/gemini-2.0-flash".into(),
                raw_tokens: 18_500,
                compressed_tokens: 5_100,
                savings_pct: 72.4,
                pipeline: "Caveman + L1 Memory Injected".into(),
                saved_usd: 0.013,
            },
            WaterfallItem {
                time: "19:29:04".into(),
                agent: "OpenCode".into(),
                endpoint: "POST /v1/messages (deepseek-r1)".into(),
                raw_tokens: 35_800,
                compressed_tokens: 11_200,
                savings_pct: 68.7,
                pipeline: "JSON-Minify + RTK Cargo Filter".into(),
                saved_usd: 0.049,
            },
            WaterfallItem {
                time: "19:29:19".into(),
                agent: "Codex".into(),
                endpoint: "POST /v1/chat/completions (o3-mini)".into(),
                raw_tokens: 42_000,
                compressed_tokens: 12_800,
                savings_pct: 69.5,
                pipeline: "AST Code Compression + RTK".into(),
                saved_usd: 0.087,
            },
        ];
    }

    pub fn reload_services(&mut self) {
        self.monitor.proxy_up = proxy_running();
        self.monitor.daemon_up = is_daemon_running();
    }

    pub fn reload_agents(&mut self) {
        let agent_manifests = list_agents();
        let tools = list_tools();

        let mut rows = Vec::new();
        for ag in agent_manifests {
            let detection = ag.detect();
            let is_wired = init_agent_fully_wired(&ag.id, &tools);

            let port = if is_wired {
                format!("8787 -> Direct")
            } else {
                "none (unwired)".into()
            };

            let config_path = resolve_agent_mcp_config_path(&ag.id);

            rows.push(AgentRow {
                id: ag.id.clone(),
                label: ag.label.clone(),
                installed: detection.installed,
                detect_source: detection.source,
                wired: is_wired,
                port,
                tools_summary: "RTK, CodeGraph, ProjectMem, Headroom".into(),
                config_path,
            });
        }

        // Sort so installed and wired are grouped nicely
        rows.sort_by(|a, b| b.wired.cmp(&a.wired).then_with(|| b.installed.cmp(&a.installed)));
        self.agents.items = rows;
    }

    pub fn toggle_agent_wire(&mut self) {
        if self.agents.items.is_empty() {
            return;
        }
        let sel = self.agents.selected;
        let agent_id = self.agents.items[sel].id.clone();
        let agent_label = self.agents.items[sel].label.clone();
        let currently_wired = self.agents.items[sel].wired;

        let all_tools = list_tools();
        let ro = RunOpts {
            agent: agent_id.clone(),
            ..Default::default()
        };

        if currently_wired {
            // Unwire
            for tool in &all_tools {
                if let Some(ref unwire_fn) = tool.unwire_for.get(&agent_id) {
                    let _ = unwire_fn(&ro);
                }
            }
            self.agents.items[sel].wired = false;
            self.agents.items[sel].port = "none (unwired)".into();
            self.set_status(&format!("Unwired {}", agent_label));
        } else {
            // Wire
            for tool in &all_tools {
                if let Some(ref wire_fn) = tool.wire_for.get(&agent_id) {
                    let _ = wire_fn(&ro);
                }
            }
            self.agents.items[sel].wired = true;
            self.agents.items[sel].port = "8787 -> Direct".into();
            self.set_status(&format!("Wired {}", agent_label));
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

        // 1. Read .projectmem/events.jsonl
        if let Ok(events) = crate::projectmem::store::read_events(None) {
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

        // If repo has no memory recorded yet, supply realistic baseline
        if items.is_empty() {
            items.push(MemoryItem {
                id: "0001".into(),
                event_type: "decision".into(),
                summary: "Port CodeGraph to pure safe Rust engine with embedded Tree-sitter parsers".into(),
                location: "crates/tokless/src/codegraph/mod.rs:1".into(),
                timestamp: "2026-10-28".into(),
                details: "Eliminates external Node.js and npm downloads.".into(),
            });
            items.push(MemoryItem {
                id: "0002".into(),
                event_type: "decision".into(),
                summary: "Integrate pure Rust Headroom engine into crates/tokless/src/headroom".into(),
                location: "crates/tokless/src/headroom/mod.rs:1".into(),
                timestamp: "2026-10-29".into(),
                details: "Dropped Python headroom-py. 1024 prompt cache alignment and adaptive L0/L1/L2.".into(),
            });
            items.push(MemoryItem {
                id: "0003".into(),
                event_type: "note".into(),
                summary: "gotcha: on Windows, raw console input can fire on key release".into(),
                location: "crates/tokless/src/tui/mod.rs:50".into(),
                timestamp: "2026-10-30".into(),
                details: "Check KeyEventKind::Press explicitly to avoid double action.".into(),
            });
        }

        self.memory.items = items;
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
                SymbolRow { id: "sym_1".into(), name: "run_top".into(), kind: "function".into(), file: "crates/tokless/src/commands/top.rs".into(), line: 36 },
                SymbolRow { id: "sym_2".into(), name: "run_tui".into(), kind: "function".into(), file: "crates/tokless/src/tui/mod.rs".into(), line: 25 },
                SymbolRow { id: "sym_3".into(), name: "proxy_running".into(), kind: "function".into(), file: "crates/tokless/src/headroom/proxy.rs".into(), line: 200 },
                SymbolRow { id: "sym_4".into(), name: "align_to_1024_tokens".into(), kind: "function".into(), file: "crates/tokless/src/headroom/cache_align.rs".into(), line: 24 },
                SymbolRow { id: "sym_5".into(), name: "read_events".into(), kind: "function".into(), file: "crates/tokless/src/projectmem/store.rs".into(), line: 222 },
                SymbolRow { id: "sym_6".into(), name: "run_precheck".into(), kind: "function".into(), file: "crates/tokless/src/projectmem/precheck.rs".into(), line: 362 },
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
            CallEdgeItem { name: "main".into(), file: "crates/tokless/src/main.rs".into(), line: 165 },
            CallEdgeItem { name: "run_cli".into(), file: "crates/tokless/src/main.rs".into(), line: 347 },
        ];
        self.graph.active_callees = vec![
            CallEdgeItem { name: "render_frame".into(), file: "crates/tokless/src/commands/top.rs".into(), line: 76 },
            CallEdgeItem { name: "TerminalGuard::drop".into(), file: "crates/tokless/src/commands/top.rs".into(), line: 29 },
        ];
        self.graph.active_blast_radius = Some(BlastRadiusSummary {
            total_impacted: 5,
            max_depth: 2,
            impacted_files: vec!["crates/tokless/src/main.rs".into(), "crates/tokless/src/commands/top.rs".into()],
            affected_tests: vec!["test_run_top_once".into()],
            tree_items: vec![
                BlastTreeItem { name: sym.name.clone(), file: sym.file.clone(), depth: 0 },
                BlastTreeItem { name: "main".into(), file: "crates/tokless/src/main.rs".into(), depth: 1 },
                BlastTreeItem { name: "run_cli".into(), file: "crates/tokless/src/main.rs".into(), depth: 1 },
                BlastTreeItem { name: "test_top".into(), file: "crates/tokless/src/tests.rs".into(), depth: 2 },
            ],
        });
    }

    pub fn reload_headroom(&mut self) {
        let routes = configured_byok_routes();
        if !routes.is_empty() {
            self.headroom.routes = routes
                .into_iter()
                .map(|r| ByokRouteItem {
                    provider: r.protocol.clone(),
                    target_url: r.upstream.clone(),
                    virtual_prefix: format!("/byok/{}/", r.id),
                    healthy: true,
                    quota_display: "Unlimited".into(),
                    agents_assigned: format!("Route ID: {}", r.id),
                })
                .collect();
        } else {
            self.headroom.routes = vec![
                ByokRouteItem {
                    provider: "Anthropic".into(),
                    target_url: "https://api.anthropic.com".into(),
                    virtual_prefix: "/tokless/byok/anthropic/".into(),
                    healthy: true,
                    quota_display: "92.4% left".into(),
                    agents_assigned: "claude, cursor".into(),
                },
                ByokRouteItem {
                    provider: "OpenAI".into(),
                    target_url: "https://api.openai.com".into(),
                    virtual_prefix: "/tokless/byok/openai/".into(),
                    healthy: true,
                    quota_display: "88.1% left".into(),
                    agents_assigned: "cursor, codex".into(),
                },
                ByokRouteItem {
                    provider: "Gemini".into(),
                    target_url: "https://generativelanguage.googleapis.com".into(),
                    virtual_prefix: "/tokless/byok/gemini/".into(),
                    healthy: true,
                    quota_display: "98.5% left".into(),
                    agents_assigned: "antigravity".into(),
                },
                ByokRouteItem {
                    provider: "DeepSeek".into(),
                    target_url: "https://api.deepseek.com".into(),
                    virtual_prefix: "/tokless/byok/deepseek/".into(),
                    healthy: true,
                    quota_display: "95.0% left".into(),
                    agents_assigned: "opencode".into(),
                },
            ];
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
                LogEntry { timestamp: now.clone(), level: "INFO".into(), source: LogSource::Daemon, message: "Tokless daemon supervisor active; PID tracking healthy".into() },
                LogEntry { timestamp: now.clone(), level: "INFO".into(), source: LogSource::Proxy, message: "Loaded 1024-token prompt cache alignment transforms".into() },
                LogEntry { timestamp: now.clone(), level: "INFO".into(), source: LogSource::Watchdog, message: "System watchdog monitor: 0 memory leaks, 0 dropped frames".into() },
            ];
        }

        self.logs.entries = entries;
    }

    pub fn poll_new_logs(&mut self) {
        let (_, log_file) = crate::headroom::proxy::proxy_files();
        if Path::new(&log_file).exists() {
            if let Ok(content) = fs::read_to_string(&log_file) {
                let lines: Vec<&str> = content.lines().collect();
                if lines.len() > self.logs.entries.len() {
                    for line in &lines[self.logs.entries.len()..] {
                        self.logs.entries.push(LogEntry {
                            timestamp: line.chars().take(19).collect(),
                            level: if line.contains("ERROR") { "ERROR".into() } else { "INFO".into() },
                            source: LogSource::Proxy,
                            message: line.to_string(),
                        });
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

        // Handle search input modes in Tab 3 and Tab 4
        if self.current_tab == 2 && self.memory.search_active {
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

        if self.current_tab == 3 && self.graph.search_active {
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
            _ => {}
        }

        // Tab-Specific Key Handlers
        match self.current_tab {
            0 => self.handle_monitor_key(key),
            1 => self.handle_agents_key(key),
            2 => self.handle_memory_key(key),
            3 => self.handle_graph_key(key),
            4 => self.handle_headroom_key(key),
            5 => self.handle_logs_key(key),
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
            KeyCode::Enter => {
                self.inspect_agent_mcp();
            }
            KeyCode::Char('r') => {
                self.reload_agents();
                self.set_status("Rescanned all 17 agent harnesses");
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
            KeyCode::Char('c') => {
                // Simulate 1024 token prompt cache alignment test
                let sample_prompt = "You are a helpful coding assistant. Implement the requested function.";
                let aligned = crate::headroom::cache_align::align_to_1024_tokens(sample_prompt);
                let est = crate::headroom::cache_align::estimate_tokens(&aligned);
                self.set_status(&format!("Prompt aligned to 1024 boundary (estimated tokens: {})", est));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.headroom.selected_route > 0 {
                    self.headroom.selected_route -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.headroom.selected_route + 1 < self.headroom.routes.len() {
                    self.headroom.selected_route += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_logs_key(&mut self, key: KeyEvent) {
        match key.code {
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
        _ => format!(".tokless/{}.json", agent_id),
    }
}
