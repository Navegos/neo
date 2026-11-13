// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/kernel/langs.rs
// created: 2026-10-28
// lastModified: 2026-11-13

use tree_sitter::Language;

pub const LANGUAGES: &[&str] = &[
    "rust", "typescript", "tsx", "javascript", "jsx", "python", "go", "c", "cpp",
    "zig", "swift", "java", "csharp", "kotlin", "scala", "php", "ruby", "lua",
    "julia", "elixir", "erlang", "ocaml", "haskell", "html", "css", "scss",
    "vue", "svelte", "graphql", "bash", "dockerfile", "hcl", "nix", "make", "cmake",
    "json", "yaml", "toml", "protobuf", "sql", "markdown", "latex", "dart", "r",
];

pub fn grammar_for(language: &str) -> Option<Language> {
    match language {
        "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "javascript" | "jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "c" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "lua" => Some(toksi_grammars::language_lua().into()),
        "dart" => Some(toksi_grammars::language_dart().into()),
        "kotlin" => Some(toksi_grammars::language_kotlin().into()),
        "scala" => Some(toksi_grammars::language_scala().into()),
        "swift" => Some(toksi_grammars::language_swift().into()),
        _ => None,
    }
}
