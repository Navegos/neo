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
    unsafe { LanguageFn::from_raw(tree_sitter_lua) }
}

pub fn language_dart() -> LanguageFn {
    unsafe { LanguageFn::from_raw(tree_sitter_dart) }
}

pub fn language_kotlin() -> LanguageFn {
    unsafe { LanguageFn::from_raw(tree_sitter_kotlin) }
}

pub fn language_scala() -> LanguageFn {
    unsafe { LanguageFn::from_raw(tree_sitter_scala) }
}

pub fn language_swift() -> LanguageFn {
    unsafe { LanguageFn::from_raw(tree_sitter_swift) }
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
    }
}
