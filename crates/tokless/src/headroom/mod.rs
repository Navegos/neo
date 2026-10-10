pub mod autostart;
pub mod byok_gateway;
pub mod proxy;
pub mod router;

pub use autostart::{disable_proxy_autostart, enable_proxy_autostart, proxy_autostart_configured, proxy_autostart_enabled};
pub use byok_gateway::run_byok_gateway_serve;
pub use proxy::{
    ensure_copilot_proxy_up, ensure_proxy_up, proxy_live_z, proxy_port, proxy_running,
    proxy_url, run_proxy_foreground, run_proxy_watchdog, start_proxy, stop_proxy,
};
pub use router::Router;

pub fn grok_oauth_proxy_owned() -> bool {
    false
}

