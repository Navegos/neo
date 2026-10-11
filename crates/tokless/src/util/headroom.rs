use std::path::{Path, PathBuf};
use std::time::Duration;
use super::exec::{run, RunOptions};
use super::mcpspawn::{pick_mcp_spawn, McpSpawn};
use super::paths::{exists, tokless_data_dir, IS_WIN};
use super::versions::semver_compare;

pub struct HeadroomPaths {
    pub root: String,
    pub uv: String,
    pub tools: String,
    pub bin: String,
    pub python: String,
}

pub fn headroom_paths_resolved() -> HeadroomPaths {
    let root = Path::new(&tokless_data_dir()).join("headroom");
    let bin = root.join("bin");
    let uv_name = if IS_WIN { "uv.exe" } else { "uv" };
    let uv = root.join("uv").join(uv_name);
    HeadroomPaths {
        tools: root.join("tools").to_string_lossy().to_string(),
        bin: bin.to_string_lossy().to_string(),
        python: root.join("python").to_string_lossy().to_string(),
        uv: uv.to_string_lossy().to_string(),
        root: root.to_string_lossy().to_string(),
    }
}

pub fn headroom_bin() -> String {
    let p = Path::new(&headroom_paths_resolved().bin).join("headroom");
    if IS_WIN {
        format!("{}.exe", p.to_string_lossy())
    } else {
        p.to_string_lossy().to_string()
    }
}

pub fn headroom_env() -> Vec<(String, String)> {
    let p = headroom_paths_resolved();
    vec![
        ("UV_TOOL_DIR".to_string(), p.tools),
        ("UV_TOOL_BIN_DIR".to_string(), p.bin),
        ("UV_PYTHON_INSTALL_DIR".to_string(), p.python),
        ("UV_MANAGED_PYTHON".to_string(), "1".to_string()),
    ]
}

pub fn headroom_uv_bootstrap_env() -> Vec<(String, String)> {
    let p = headroom_paths_resolved();
    let mut env = headroom_env();
    if let Some(uv_dir) = Path::new(&p.uv).parent() {
        env.push(("UV_INSTALL_DIR".to_string(), uv_dir.to_string_lossy().to_string()));
    }
    env.push(("UV_NO_MODIFY_PATH".to_string(), "1".to_string()));
    env
}

pub fn mcp_spawn_for(tool_id: &str) -> McpSpawn {
    pick_mcp_spawn(tool_id, &[])
}

pub fn headroom_installed() -> bool {
    let b = headroom_bin();
    exists(&b) && Path::new(&b).is_file()
}

pub fn headroom_installed_version() -> Option<String> {
    if let Some(v) = headroom_dist_info_version() {
        return Some(v);
    }
    let res = run(
        &headroom_bin(),
        &["--version"],
        RunOptions {
            capture: true,
            timeout: Some(Duration::from_secs(10)),
            ..Default::default()
        },
    );
    if res.code != 0 {
        return None;
    }
    if let Some((_, ver)) = res.stdout.trim().split_once("version ") {
        let trimmed = ver.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn headroom_dist_info_version() -> Option<String> {
    let p = headroom_paths_resolved();
    let mut site_dirs: Vec<PathBuf> = vec![
        Path::new(&p.tools)
            .join("headroom-ai")
            .join("Lib")
            .join("site-packages"),
    ];
    let lib_dir = Path::new(&p.tools).join("headroom-ai").join("lib");
    if let Ok(entries) = std::fs::read_dir(&lib_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("python") {
                site_dirs.push(e.path().join("site-packages"));
            }
        }
    }
    let mut best: Option<String> = None;
    for site in &site_dirs {
        if let Ok(entries) = std::fs::read_dir(site) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("headroom_ai-") && name.ends_with(".dist-info") {
                    let ver = name
                        .trim_start_matches("headroom_ai-")
                        .trim_end_matches(".dist-info")
                        .to_string();
                    if !ver.is_empty() {
                        match &best {
                            None => best = Some(ver),
                            Some(cur) if semver_compare(&ver, cur) > 0 => best = Some(ver),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    best
}

pub fn ensure_headroom(upgrade: bool) -> Result<bool, String> {
    crate::headroom::ensure_installed(crate::core::RunOpts {
        upgrade,
        ..Default::default()
    })
    .map_err(|e| e.to_string())
}
