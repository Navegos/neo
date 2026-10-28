// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/compression/mod.rs

pub mod code;
pub mod json;
pub mod log;

pub use code::{compress_code, detect_code_language, CodeCompressorConfig, CodeLanguage};
pub use json::{prune_embedded_json, prune_json, JsonPrunerConfig};
pub use log::{classify_line, compress_logs, deduplicate_logs, detect_format, LogCompressorConfig, LogFormat, LogLevel};

use crate::rtk::strip_ansi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Code(CodeLanguage),
    Json,
    Log(LogFormat),
    Text,
}

pub fn detect_content_kind(content: &str, hint: Option<&str>) -> ContentKind {
    let clean = strip_ansi(content);
    let trimmed = clean.trim();

    if let Some(h) = hint {
        let lang = CodeLanguage::from_hint(h);
        if lang != CodeLanguage::Unknown {
            return ContentKind::Code(lang);
        }
        if h.eq_ignore_ascii_case("json") {
            return ContentKind::Json;
        }
        if h.eq_ignore_ascii_case("log") {
            let lines: Vec<&str> = trimmed.lines().collect();
            return ContentKind::Log(detect_format(&lines));
        }
    }

    if (trimmed.starts_with('{') && trimmed.ends_with('}')) || (trimmed.starts_with('[') && trimmed.ends_with(']')) {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return ContentKind::Json;
        }
    }

    let code_lang = detect_code_language(trimmed);
    if code_lang != CodeLanguage::Unknown {
        return ContentKind::Code(code_lang);
    }

    let lines: Vec<&str> = trimmed.lines().collect();
    let log_fmt = detect_format(&lines);
    if log_fmt != LogFormat::Generic || lines.len() > 10 {
        return ContentKind::Log(log_fmt);
    }

    ContentKind::Text
}

pub fn compress_content(content: &str, hint: Option<&str>) -> String {
    let clean = strip_ansi(content);
    let kind = detect_content_kind(&clean, hint);

    match kind {
        ContentKind::Code(lang) => compress_code(&clean, lang, &CodeCompressorConfig::default()),
        ContentKind::Json => prune_json(&clean, &JsonPrunerConfig::default()),
        ContentKind::Log(_) => deduplicate_logs(&clean),
        ContentKind::Text => prune_embedded_json(&clean, &JsonPrunerConfig::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_content_auto_detection() {
        let json_data = "{\n  \"hello\": \"world\",\n  \"error\": null\n}";
        assert_eq!(compress_content(json_data, None), r#"{"hello":"world"}"#);

        let log_data = "Compiling lib v0.1.0\nCompiling lib v0.1.0\nerror: aborting due to 1 previous error";
        let res = compress_content(log_data, None);
        assert!(res.contains("[... repeated 1 times"));
        assert!(res.contains("error: aborting"));
    }
}

