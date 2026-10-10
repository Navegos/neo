use std::path::Path;
use super::paths::{exists, tokless_data_dir, IS_WIN};
use super::exec::{run, RunOptions};

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

pub fn headroom_installed() -> bool {
    let b = headroom_bin();
    exists(&b) && Path::new(&b).is_file()
}

pub fn headroom_installed_version() -> Option<String> {
    if let Some(v) = headroom_dist_info_version() {
        return Some(v);
    }
    let res = run(&headroom_bin(), &["--version"], RunOptions { capture: true, ..Default::default() });
    if res.code != 0 {
        return None;
    }
    if let Some((_, ver)) = res.stdout.trim().split_once("version ") {
        if !ver.is_empty() {
            return Some(ver.trim().to_string());
        }
    }
    None
}

fn headroom_dist_info_version() -> Option<String> {
    let p = headroom_paths_resolved();
    let site_dirs = [
        Path::new(&p.tools).join("headroom-ai").join("Lib").join("site-packages"),
        Path::new(&p.tools).join("headroom-ai").join("lib"),
    ];
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
                        return Some(ver);
                    }
                }
            }
        }
    }
    None
}
