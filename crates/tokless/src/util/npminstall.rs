use super::exec::{resolve_node_binary, resolve_npm_binary, run, RunOptions};

pub fn node_major() -> i32 {
    let node_bin = resolve_node_binary();
    if node_bin.is_empty() {
        return 0;
    }
    let r = run(&node_bin, &["--version"], RunOptions { capture: true, ..Default::default() });
    let v = r.stdout.trim().trim_start_matches('v');
    if let Some((maj, _)) = v.split_once('.') {
        return maj.parse::<i32>().unwrap_or(0);
    }
    0
}

pub fn node_too_old_hint(min: i32) -> String {
    let maj = node_major();
    if maj > 0 && maj < min {
        format!(
            "Node.js v{} is too old (need v{}+). Upgrade via nvm or nodejs.org",
            maj, min
        )
    } else {
        String::new()
    }
}

pub fn npm_install_global(pkg: &str) -> bool {
    let npm_bin = resolve_npm_binary();
    if npm_bin.is_empty() {
        return false;
    }
    let res = run(&npm_bin, &["install", "-g", pkg], RunOptions::default());
    res.code == 0
}
