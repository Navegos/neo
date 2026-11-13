// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/kernel/langs.rs
// created: 2026-10-28
// lastModified: 2026-11-13

//! Language registry: maps codegraph language names to native Tree-sitter grammars.

use tree_sitter::Language;

/// All supported polyglot languages (41 specified + Dart, R, Luau, TSX/JSX).
pub const LANGUAGES: &[&str] = &[
    "rust", "typescript", "tsx", "javascript", "jsx", "python", "go", "c", "cpp",
    "zig", "swift", "java", "csharp", "kotlin", "scala", "php", "ruby", "lua", "luau",
    "julia", "elixir", "erlang", "ocaml", "haskell", "html", "css", "scss",
    "vue", "svelte", "graphql", "bash", "dockerfile", "hcl", "nix", "make", "cmake",
    "json", "yaml", "toml", "protobuf", "sql", "markdown", "latex", "dart", "r",
];

/// Normalize common language aliases to canonical registry identifiers.
pub fn canonical_language(language: &str) -> &str {
    match language.trim().to_ascii_lowercase().as_str() {
        "rs" | "rust" => "rust",
        "ts" | "typescript" => "typescript",
        "tsx" => "tsx",
        "js" | "javascript" => "javascript",
        "jsx" => "jsx",
        "py" | "python" => "python",
        "go" | "golang" => "go",
        "c" => "c",
        "c++" | "cpp" | "cc" | "cxx" => "cpp",
        "zig" => "zig",
        "swift" => "swift",
        "java" => "java",
        "cs" | "c#" | "c_sharp" | "csharp" => "csharp",
        "kt" | "kts" | "kotlin" => "kotlin",
        "sc" | "scala" => "scala",
        "php" => "php",
        "rb" | "ruby" => "ruby",
        "lua" => "lua",
        "luau" => "luau",
        "jl" | "julia" => "julia",
        "ex" | "exs" | "elixir" => "elixir",
        "erl" | "hrl" | "erlang" => "erlang",
        "ml" | "mli" | "ocaml" => "ocaml",
        "hs" | "haskell" => "haskell",
        "htm" | "html" => "html",
        "css" => "css",
        "scss" => "scss",
        "vue" => "vue",
        "svelte" => "svelte",
        "gql" | "graphql" => "graphql",
        "sh" | "zsh" | "shell" | "bash" => "bash",
        "docker" | "dockerfile" => "dockerfile",
        "tf" | "terraform" | "hcl" => "hcl",
        "nix" => "nix",
        "mk" | "makefile" | "make" => "make",
        "cmake" => "cmake",
        "json" => "json",
        "yml" | "yaml" => "yaml",
        "toml" => "toml",
        "proto" | "protobuf" => "protobuf",
        "sql" => "sql",
        "md" | "markdown" => "markdown",
        "tex" | "latex" => "latex",
        "dart" => "dart",
        "r" | "rlang" => "r",
        _ => language,
    }
}

/// Resolve the native Tree-sitter grammar for a language identifier.
pub fn grammar_for(language: &str) -> Option<Language> {
    match canonical_language(language) {
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "javascript" | "jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "c" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "zig" => Some(tree_sitter_zig::LANGUAGE.into()),
        "swift" => Some(toksi_grammars::language_swift().into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "csharp" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        "kotlin" => Some(toksi_grammars::language_kotlin().into()),
        "scala" => Some(toksi_grammars::language_scala().into()),
        "php" => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        "ruby" => Some(tree_sitter_ruby::LANGUAGE.into()),
        "lua" => Some(toksi_grammars::language_lua().into()),
        "luau" => Some(tree_sitter_luau::LANGUAGE.into()),
        "julia" => Some(tree_sitter_julia::LANGUAGE.into()),
        "elixir" => Some(tree_sitter_elixir::LANGUAGE.into()),
        "erlang" => Some(tree_sitter_erlang::LANGUAGE.into()),
        "ocaml" => Some(tree_sitter_ocaml::LANGUAGE_OCAML.into()),
        "haskell" => Some(tree_sitter_haskell::LANGUAGE.into()),
        "html" | "vue" => Some(tree_sitter_html::LANGUAGE.into()),
        "css" | "scss" => Some(tree_sitter_css::LANGUAGE.into()),
        "svelte" => Some(tree_sitter_svelte_ng::LANGUAGE.into()),
        "graphql" => Some(tree_sitter_graphql::LANGUAGE.into()),
        "bash" => Some(tree_sitter_bash::LANGUAGE.into()),
        "hcl" => Some(tree_sitter_hcl::LANGUAGE.into()),
        "nix" => Some(tree_sitter_nix::LANGUAGE.into()),
        "make" => Some(tree_sitter_make::LANGUAGE.into()),
        "cmake" => Some(tree_sitter_cmake::LANGUAGE.into()),
        "json" => Some(tree_sitter_json::LANGUAGE.into()),
        "yaml" => Some(tree_sitter_yaml::LANGUAGE.into()),
        "toml" => Some(tree_sitter_toml_ng::LANGUAGE.into()),
        "protobuf" => Some(tree_sitter_proto::LANGUAGE.into()),
        "sql" => Some(tree_sitter_sequel::LANGUAGE.into()),
        "markdown" => Some(tree_sitter_md::LANGUAGE.into()),
        "latex" => Some(codebook_tree_sitter_latex::LANGUAGE.into()),
        "dart" => Some(toksi_grammars::language_dart().into()),
        "r" => Some(tree_sitter_r::LANGUAGE.into()),
        _ => None,
    }
}
