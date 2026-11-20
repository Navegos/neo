// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_lua() -> *const ();
    fn tree_sitter_dart() -> *const ();
    fn tree_sitter_kotlin() -> *const ();
    fn tree_sitter_scala() -> *const ();
    fn tree_sitter_swift() -> *const ();
}

pub fn language_lua() -> LanguageFn {
    // SAFETY: `tree_sitter_lua` is a statically linked C grammar entrypoint compiled in `build.rs`
    // conforming to the Tree-sitter C ABI (ABI 14/15), returning a valid static `TSLanguage` pointer.
    unsafe { LanguageFn::from_raw(tree_sitter_lua) }
}

pub fn language_dart() -> LanguageFn {
    // SAFETY: `tree_sitter_dart` is a statically linked C grammar entrypoint compiled in `build.rs`
    // conforming to the Tree-sitter C ABI (ABI 14/15), returning a valid static `TSLanguage` pointer.
    unsafe { LanguageFn::from_raw(tree_sitter_dart) }
}

pub fn language_kotlin() -> LanguageFn {
    // SAFETY: `tree_sitter_kotlin` is a statically linked C grammar entrypoint compiled in `build.rs`
    // conforming to the Tree-sitter C ABI (ABI 14/15), returning a valid static `TSLanguage` pointer.
    unsafe { LanguageFn::from_raw(tree_sitter_kotlin) }
}

pub fn language_scala() -> LanguageFn {
    // SAFETY: `tree_sitter_scala` is a statically linked C grammar entrypoint compiled in `build.rs`
    // conforming to the Tree-sitter C ABI (ABI 14/15), returning a valid static `TSLanguage` pointer.
    unsafe { LanguageFn::from_raw(tree_sitter_scala) }
}

pub fn language_swift() -> LanguageFn {
    // SAFETY: `tree_sitter_swift` is a statically linked C grammar entrypoint compiled in `build.rs`
    // conforming to the Tree-sitter C ABI (ABI 14/15), returning a valid static `TSLanguage` pointer.
    unsafe { LanguageFn::from_raw(tree_sitter_swift) }
}

pub fn language_dockerfile() -> LanguageFn {
    arborium_dockerfile::language()
}

pub fn language_scss() -> LanguageFn {
    arborium_scss::language()
}

pub fn language_vue() -> LanguageFn {
    arborium_vue::language()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_fn_pointers() {
        let _ = language_lua();
        let _ = language_dart();
        let _ = language_kotlin();
        let _ = language_scala();
        let _ = language_swift();
        let _ = language_dockerfile();
        let _ = language_scss();
        let _ = language_vue();
    }
}
