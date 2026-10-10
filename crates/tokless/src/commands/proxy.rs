use crate::agents::proxy_agent::{configure_proxy_agent, detect_proxy, proxy_state_label, remove_proxy_agent, ProxyState};
use crate::commands::helpers::*;
use crate::headroom::autostart::{disable_proxy_autostart, enable_proxy_autostart, proxy_autostart_enabled, start_proxy, stop_proxy};
use crate::headroom::proxy::{proxy_port, proxy_running};
use crate::util::colors::*;
use crate::util::headroom_proxy::set_proxy_routing_enabled;
use crate::util::logger::L;
use crate::util::progress::*;

pub fn proxy_agent_ids() -> Vec<String> {
    vec![
        "claude".into(),
        "codex".into(),
        "opencode".into(),
        "omp".into(),
        "kilo".into(),
        "pi".into(),
        "droid".into(),
        "antigravity".into(),
        "grok".into(),
        "copilot".into(),
        "cline".into(),
        "cursor".into(),
    ]
}

pub fn run_proxy_up(opts: InitOptions) -> i32 {
    cmd_header("proxy up", "route agents through headroom");

    if opts.dry_run {
        L.raw(&format!("  {} Dry run — would start headroom and configure agents.", C.cyan(Sym::INFO)));
        L.raw("");
        return 0;
    }

    if let Err(e) = start_proxy() {
        L.err(&format!("failed to start headroom proxy: {}", e));
        return 1;
    }
    let _ = enable_proxy_autostart();
    let _ = set_proxy_routing_enabled(true);

    let targets = opts.agents.unwrap_or_else(proxy_agent_ids);
    tree_top("Agents");
    for id in &targets {
        let ok = configure_proxy_agent(id);
        let mark = if ok { C.green(Sym::CHECK) } else { C.yellow(Sym::WARN) };
        let status = if ok { C.green("routed") } else { C.yellow("could not route") };
        tree_leaf(&format!("{} {} {}", mark, paint_name(&pad_end(id, 14)), status));
    }
    tree_close();

    tree_status(&[status_ok(&format!("Headroom running on port {}.", proxy_port()))]);
    print_repo_footer(true);
    L.raw("");
    0
}

pub fn run_proxy_down(opts: InitOptions) -> i32 {
    cmd_header("proxy down", "restore direct agent connections");

    if opts.dry_run {
        L.raw(&format!("  {} Dry run — would restore direct routing and stop headroom.", C.cyan(Sym::INFO)));
        L.raw("");
        return 0;
    }

    let targets = opts.agents.unwrap_or_else(proxy_agent_ids);
    tree_top("Agents");
    for id in &targets {
        let ok = remove_proxy_agent(id);
        let mark = if ok { C.green(Sym::CHECK) } else { C.gray(Sym::BULLET) };
        let status = if ok { C.green("restored") } else { C.gray("untouched") };
        tree_leaf(&format!("{} {} {}", mark, paint_name(&pad_end(id, 14)), status));
    }
    tree_close();

    let _ = disable_proxy_autostart();
    let _ = stop_proxy();
    let _ = set_proxy_routing_enabled(false);

    tree_status(&[status_ok("Headroom stopped and agent routing restored.")]);
    print_repo_footer(true);
    L.raw("");
    0
}

pub fn run_proxy_status(opts: InitOptions) -> i32 {
    cmd_header("proxy status", "headroom proxy routing");

    let running = proxy_running();
    let autostart = proxy_autostart_enabled();

    tree_top("Daemon");
    let status_str = if running {
        format!("{} on port {}", C.green("running"), proxy_port())
    } else {
        C.dim("stopped")
    };
    tree_leaf(&format!("{}{}", paint_key(&pad_end("status", 12)), status_str));
    let autostart_str = if autostart { C.green("enabled") } else { C.dim("disabled") };
    tree_leaf(&format!("{}{}", paint_key(&pad_end("autostart", 12)), autostart_str));
    tree_close();

    let targets = opts.agents.unwrap_or_else(proxy_agent_ids);
    tree_corner("Agents");
    for id in &targets {
        let det = detect_proxy(id);
        let label = proxy_state_label(det.state);
        let mark = match det.state {
            ProxyState::ProxyStateManagedBYOK | ProxyState::ProxyStateManagedOAuth => C.green(Sym::CHECK),
            ProxyState::ProxyStateNativeDirect | ProxyState::ProxyStateNativeUnconfigured => C.gray(Sym::BULLET),
            _ => C.yellow(Sym::WARN),
        };
        tree_leaf(&format!("{} {} {}", mark, paint_name(&pad_end(id, 14)), label));
    }
    tree_close();

    print_repo_footer(true);
    L.raw("");
    0
}

