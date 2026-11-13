// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let mut lua = cc::Build::new();
    lua.include("grammars/lua");
    lua.file("grammars/lua/parser.c");
    lua.file("grammars/lua/scanner.c");
    lua.flag_if_supported("-Wno-unused-parameter");
    lua.flag_if_supported("-Wno-unused-but-set-variable");
    lua.flag_if_supported("-Wno-trigraphs");
    lua.flag_if_supported("-utf-8");
    lua.compile("tree-sitter-lua");
    println!("cargo:rerun-if-changed=grammars/lua");

    let mut dart = cc::Build::new();
    dart.include("grammars/dart");
    dart.file("grammars/dart/parser.c");
    dart.file("grammars/dart/scanner.c");
    dart.flag_if_supported("-Wno-unused-parameter");
    dart.flag_if_supported("-Wno-unused-but-set-variable");
    dart.flag_if_supported("-Wno-trigraphs");
    dart.flag_if_supported("-Wno-unused-function");
    dart.flag_if_supported("-utf-8");
    dart.compile("tree-sitter-dart");
    println!("cargo:rerun-if-changed=grammars/dart");

    let mut kotlin = cc::Build::new();
    kotlin.include("grammars/kotlin");
    kotlin.file("grammars/kotlin/parser.c");
    kotlin.file("grammars/kotlin/scanner.c");
    kotlin.flag_if_supported("-Wno-unused-parameter");
    kotlin.flag_if_supported("-Wno-unused-but-set-variable");
    kotlin.flag_if_supported("-Wno-trigraphs");
    kotlin.flag_if_supported("-utf-8");
    kotlin.compile("tree-sitter-kotlin");
    println!("cargo:rerun-if-changed=grammars/kotlin");

    let mut scala = cc::Build::new();
    scala.include("grammars/scala");
    scala.file("grammars/scala/parser.c");
    scala.file("grammars/scala/scanner.c");
    scala.flag_if_supported("-Wno-unused-parameter");
    scala.flag_if_supported("-Wno-unused-but-set-variable");
    scala.flag_if_supported("-Wno-trigraphs");
    scala.flag_if_supported("-utf-8");
    scala.compile("tree-sitter-scala");
    println!("cargo:rerun-if-changed=grammars/scala");

    let mut swift = cc::Build::new();
    swift.std("c11");
    swift.include("grammars/swift");
    swift.file("grammars/swift/parser.c");
    swift.file("grammars/swift/scanner.c");
    swift.flag_if_supported("-Wno-unused-parameter");
    swift.flag_if_supported("-Wno-unused-but-set-variable");
    swift.flag_if_supported("-Wno-trigraphs");
    swift.flag_if_supported("-utf-8");
    swift.compile("tree-sitter-swift");
    println!("cargo:rerun-if-changed=grammars/swift");
}
