// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/cloudcode_patch.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::path::{Path, PathBuf};

use crate::util::headroom::headroom_paths_resolved;
use crate::util::logger::L;
use crate::util::paths::write_file_atomic;

pub const CLOUD_CODE_PATCH_MARKER: &str = "gemini:tool_schema_compaction";

pub fn cloud_code_patch_roots() -> Vec<String> {
    let tools = headroom_paths_resolved().tools;
    vec![
        Path::new(&tools)
            .join("headroom-ai")
            .join("lib")
            .join("python*")
            .join("site-packages")
            .join("headroom")
            .join("proxy")
            .join("handlers")
            .join("gemini.py")
            .to_string_lossy()
            .to_string(),
        Path::new(&tools)
            .join("headroom-ai")
            .join("Lib")
            .join("site-packages")
            .join("headroom")
            .join("proxy")
            .join("handlers")
            .join("gemini.py")
            .to_string_lossy()
            .to_string(),
    ]
}

fn expand_patch_candidate_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let tools = PathBuf::from(headroom_paths_resolved().tools);
    let unix_lib = tools.join("headroom-ai").join("lib");
    if let Ok(entries) = std::fs::read_dir(&unix_lib) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("python") {
                let candidate = entry
                    .path()
                    .join("site-packages")
                    .join("headroom")
                    .join("proxy")
                    .join("handlers")
                    .join("gemini.py");
                if candidate.is_file() {
                    out.push(candidate);
                }
            }
        }
    }
    let win_candidate = tools
        .join("headroom-ai")
        .join("Lib")
        .join("site-packages")
        .join("headroom")
        .join("proxy")
        .join("handlers")
        .join("gemini.py");
    if win_candidate.is_file() {
        out.push(win_candidate);
    }
    out
}

pub fn patch_cloud_code_gemini(
    path: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let data = std::fs::read_to_string(path)?;
    if data.contains(CLOUD_CODE_PATCH_MARKER) {
        return Ok(false);
    }

    let replacements: &[(&str, &str)] = &[
        (
            "        contents = request_payload.get(\"contents\", [])\n        headers = dict(request.headers.items())\n",
            "        contents = request_payload.get(\"contents\", [])\n        tools_before = request_payload.get(\"tools\")\n        tools_after = tools_before\n        tools_modified = False\n        headers = dict(request.headers.items())\n",
        ),
        (
            "        if not messages:\n            original_tokens = 0\n        optimized_messages = messages\n",
            "        if not messages:\n            original_tokens = 0\n        _tool_tokens_before = 0\n        _tool_tokens_after = 0\n        optimized_messages = messages\n",
        ),
        (
            "            if not is_antigravity:\n                if optimized_system:\n                    request_payload[\"systemInstruction\"] = optimized_system\n                elif \"systemInstruction\" in request_payload:\n                    del request_payload[\"systemInstruction\"]\n\n        tokens_saved = original_tokens - optimized_tokens\n        optimization_latency = (time.time() - start_time) * 1000\n        base_url = self._resolve_cloudcode_base_url(is_antigravity)\n",
            "            if optimized_system:\n                request_payload[\"systemInstruction\"] = optimized_system\n\n        if not _decision.bypass_header_set and isinstance(tools_before, list) and tools_before:\n            try:\n                from headroom.proxy.tool_schema_compaction import compact_tools\n\n                compacted_payload, tools_modified, _, _ = compact_tools(\n                    {\"tools\": tools_before}\n                )\n                tools_after = compacted_payload.get(\"tools\", tools_before)\n                if tools_modified:\n                    request_payload[\"tools\"] = tools_after\n            except Exception:\n                tools_modified = False\n                tools_after = tools_before\n                logger.debug(\"Cloud Code tool schema compaction skipped\", exc_info=True)\n            if tools_modified:\n                try:\n                    _tool_tokens_before = tokenizer.count_text(\n                        json.dumps(tools_before, default=str)\n                    )\n                    _tool_tokens_after = tokenizer.count_text(\n                        json.dumps(tools_after, default=str)\n                    )\n                except Exception:\n                    logger.debug(\"Cloud Code tool token recount skipped\", exc_info=True)\n\n        if 0 < _tool_tokens_after < _tool_tokens_before:\n            original_tokens += _tool_tokens_before\n            optimized_tokens += _tool_tokens_after\n            transforms_applied.append(\"gemini:tool_schema_compaction\")\n        tokens_saved = max(0, original_tokens - optimized_tokens)\n        optimization_latency = (time.time() - start_time) * 1000\n        base_url = self._resolve_cloudcode_base_url(is_antigravity)\n",
        ),
    ];

    let mut patched = data;
    for &(old, new) in replacements {
        if patched.matches(old).count() != 1 {
            return Ok(false);
        }
        patched = patched.replacen(old, new, 1);
    }

    let _meta = std::fs::metadata(path)?;
    write_file_atomic(path, &patched)?;
    Ok(true)
}

pub fn apply_cloud_code_patch() -> bool {
    let mut changed = false;
    for path in expand_patch_candidate_paths() {
        let path_str = path.to_string_lossy().to_string();
        match patch_cloud_code_gemini(&path_str) {
            Ok(patched) => {
                changed = changed || patched;
            }
            Err(err) => {
                L.sub(&format!("headroom Cloud Code patch skipped: {}", err));
            }
        }
    }
    changed
}
