// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/ansi.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::borrow::Cow;

/// Check if a string contains any ANSI escape codes.
#[inline]
pub fn has_ansi(s: &str) -> bool {
    s.as_bytes().contains(&0x1b)
}

/// Zero-allocation ANSI escape sequence stripper.
///
/// When the input contains no ANSI escape sequences (the common case for plain text),
/// returns `Cow::Borrowed` with zero heap allocation.
/// When ANSI sequences are present, allocates a single pre-sized `String` and strips
/// CSI (`\x1b[...]`), OSC (`\x1b]...\x07` or `\x1b]...\x1b\`), and other escape codes.
pub fn strip_ansi_cow(input: &str) -> Cow<'_, str> {
    if !has_ansi(input) {
        return Cow::Borrowed(input);
    }

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                // CSI: \x1b[ ... [0x40..=0x7E]
                b'[' => {
                    i += 1;
                    while i < bytes.len() {
                        let b = bytes[i];
                        if (0x40..=0x7E).contains(&b) {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                }
                // OSC: \x1b] ... (\x07 or \x1b\)
                b']' => {
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == 0x07 {
                            i += 1;
                            break;
                        }
                        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                // Character set designators (\x1b(B, \x1b)0, etc.)
                b'(' | b')' | b'*' | b'+' => {
                    i += 2;
                }
                // Two-character escape sequences
                _ => {
                    i += 1;
                }
            }
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != 0x1b {
                i += 1;
            }
            if let Ok(valid_chunk) = std::str::from_utf8(&bytes[start..i]) {
                out.push_str(valid_chunk);
            }
        }
    }

    Cow::Owned(out)
}

/// Strip ANSI escape codes, returning a new `String`.
pub fn strip_ansi(input: &str) -> String {
    strip_ansi_cow(input).into_owned()
}

/// Compute the visible character length of a string ignoring ANSI escape codes.
pub fn visible_len(input: &str) -> usize {
    strip_ansi_cow(input).chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_allocation_when_no_ansi() {
        let text = "Hello world! This is a clean string without escapes.";
        let res = strip_ansi_cow(text);
        assert!(matches!(res, Cow::Borrowed(_)));
        assert_eq!(res.as_ref(), text);
    }

    #[test]
    fn test_strip_csi_colors() {
        let input = "\x1b[31mError:\x1b[0m \x1b[1;32mEverything failed\x1b[m";
        let res = strip_ansi_cow(input);
        assert!(matches!(res, Cow::Owned(_)));
        assert_eq!(res.as_ref(), "Error: Everything failed");
    }

    #[test]
    fn test_strip_osc_titles() {
        let input = "\x1b]0;Terminal Window\x07Cargo build succeeded";
        assert_eq!(strip_ansi(input), "Cargo build succeeded");

        let input_st = "\x1b]0;Terminal Window\x1b\\Cargo build succeeded";
        assert_eq!(strip_ansi(input_st), "Cargo build succeeded");
    }

    #[test]
    fn test_strip_24bit_rgb_and_cursor() {
        let input = "\x1b[38;2;255;128;64mColored\x1b[2K\x1b[1GLine";
        assert_eq!(strip_ansi(input), "ColoredLine");
    }

    #[test]
    fn test_visible_len() {
        let input = "\x1b[32mOK\x1b[0m";
        assert_eq!(visible_len(input), 2);
    }
}

