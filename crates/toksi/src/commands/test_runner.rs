// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/test_runner.rs
// created: 2026-11-20
// lastModified: 2026-11-20

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::util::paths::normalize_path;

// -----------------------------------------------------------------------------
// Pure-Rust BLAKE3 Implementation (Zero External Dependencies, Zero Unsafe)
// -----------------------------------------------------------------------------

const IV: [u32; 8] = [
    0x6A09_E667, 0xBB67_AE85, 0x3C6E_F372, 0xA54F_F53A,
    0x510E_527F, 0x9B05_688C, 0x1F83_D9AB, 0x5BE0_CD19,
];

const MSG_PERM: [usize; 16] = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];

const CHUNK_START: u32 = 1 << 0;
const CHUNK_END: u32 = 1 << 1;
const PARENT: u32 = 1 << 2;
const ROOT: u32 = 1 << 3;
const KEYED_HASH: u32 = 1 << 4;
const CHUNK_LEN: usize = 1024;
const BLOCK_LEN: usize = 64;

fn g(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, mx: u32, my: u32) {
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(mx);
    state[d] = (state[d] ^ state[a]).rotate_right(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(12);
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(my);
    state[d] = (state[d] ^ state[a]).rotate_right(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(7);
}

fn round(state: &mut [u32; 16], m: &[u32; 16]) {
    g(state, 0, 4, 8, 12, m[0], m[1]);
    g(state, 1, 5, 9, 13, m[2], m[3]);
    g(state, 2, 6, 10, 14, m[4], m[5]);
    g(state, 3, 7, 11, 15, m[6], m[7]);
    g(state, 0, 5, 10, 15, m[8], m[9]);
    g(state, 1, 6, 11, 12, m[10], m[11]);
    g(state, 2, 7, 8, 13, m[12], m[13]);
    g(state, 3, 4, 9, 14, m[14], m[15]);
}

fn permute(m: &mut [u32; 16]) {
    let mut permuted = [0u32; 16];
    for i in 0..16 {
        permuted[i] = m[MSG_PERM[i]];
    }
    *m = permuted;
}

fn compress(
    cv: &[u32; 8],
    block_words: &[u32; 16],
    counter: u64,
    block_len: u32,
    flags: u32,
) -> [u32; 16] {
    let mut state = [
        cv[0], cv[1], cv[2], cv[3], cv[4], cv[5], cv[6], cv[7],
        IV[0], IV[1], IV[2], IV[3], counter as u32, (counter >> 32) as u32, block_len, flags,
    ];
    let mut m = *block_words;
    for r in 0..7 {
        round(&mut state, &m);
        if r < 6 {
            permute(&mut m);
        }
    }
    for i in 0..8 {
        state[i] ^= state[i + 8];
        state[i + 8] ^= cv[i];
    }
    state
}

fn words_from_le_bytes_64(bytes: &[u8]) -> [u32; 16] {
    let mut padded = [0u8; 64];
    let take = bytes.len().min(64);
    padded[..take].copy_from_slice(&bytes[..take]);
    let mut words = [0u32; 16];
    for (i, chunk) in padded.as_chunks::<4>().0.iter().enumerate() {
        words[i] = u32::from_le_bytes(*chunk);
    }
    words
}

fn first_8_words(out: [u32; 16]) -> [u32; 8] {
    [out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]]
}

fn chunk_cv_with_iv(
    initial_cv: &[u32; 8],
    base_flags: u32,
    chunk: &[u8],
    chunk_index: u64,
    is_root: bool,
) -> [u32; 8] {
    let mut cv = *initial_cv;
    if chunk.is_empty() {
        let block = [0u32; 16];
        let flags = base_flags | CHUNK_START | CHUNK_END | if is_root { ROOT } else { 0 };
        return first_8_words(compress(&cv, &block, chunk_index, 0, flags));
    }
    let num_blocks = chunk.len().div_ceil(BLOCK_LEN);
    for (idx, block_bytes) in chunk.chunks(BLOCK_LEN).enumerate() {
        let mut flags = base_flags;
        if idx == 0 {
            flags |= CHUNK_START;
        }
        if idx + 1 == num_blocks {
            flags |= CHUNK_END;
            if is_root {
                flags |= ROOT;
            }
        }
        let words = words_from_le_bytes_64(block_bytes);
        cv = first_8_words(compress(&cv, &words, chunk_index, block_bytes.len() as u32, flags));
    }
    cv
}

fn parent_cv_with_iv(
    initial_cv: &[u32; 8],
    base_flags: u32,
    left: &[u32; 8],
    right: &[u32; 8],
    is_root: bool,
) -> [u32; 8] {
    let mut block = [0u32; 16];
    block[..8].copy_from_slice(left);
    block[8..].copy_from_slice(right);
    let flags = base_flags | PARENT | if is_root { ROOT } else { 0 };
    first_8_words(compress(initial_cv, &block, 0, BLOCK_LEN as u32, flags))
}

fn blake3_hash_with_iv(initial_cv: &[u32; 8], base_flags: u32, input: &[u8]) -> [u8; 32] {
    if input.len() <= CHUNK_LEN {
        let cv = chunk_cv_with_iv(initial_cv, base_flags, input, 0, true);
        let mut out = [0u8; 32];
        for (i, w) in cv.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&w.to_le_bytes());
        }
        return out;
    }

    let mut stack: Vec<[u32; 8]> = Vec::new();
    let chunks: Vec<&[u8]> = input.chunks(CHUNK_LEN).collect();
    for (idx, chunk) in chunks.iter().enumerate() {
        let mut cv = chunk_cv_with_iv(initial_cv, base_flags, chunk, idx as u64, false);
        let mut total_chunks = idx + 1;
        let is_last = idx + 1 == chunks.len();
        if !is_last {
            while total_chunks & 1 == 0 {
                if let Some(left) = stack.pop() {
                    cv = parent_cv_with_iv(initial_cv, base_flags, &left, &cv, false);
                }
                total_chunks >>= 1;
            }
            stack.push(cv);
        } else {
            while let Some(left) = stack.pop() {
                let root_now = stack.is_empty();
                cv = parent_cv_with_iv(initial_cv, base_flags, &left, &cv, root_now);
            }
            let mut out = [0u8; 32];
            for (i, w) in cv.iter().enumerate() {
                out[i * 4..(i + 1) * 4].copy_from_slice(&w.to_le_bytes());
            }
            return out;
        }
    }

    [0u8; 32]
}

pub fn blake3_hash(input: &[u8]) -> [u8; 32] {
    blake3_hash_with_iv(&IV, 0, input)
}

pub fn blake3_keyed_hash(key: &[u8; 32], input: &[u8]) -> [u8; 32] {
    let mut key_words = [0u32; 8];
    for (i, chunk) in key.chunks_exact(4).enumerate() {
        key_words[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    blake3_hash_with_iv(&key_words, KEYED_HASH, input)
}

pub fn blake3_hex(input: &[u8]) -> String {
    let digest = blake3_hash(input);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

pub fn blake3_keyed_hex(key: &[u8; 32], input: &[u8]) -> String {
    let digest = blake3_keyed_hash(key, input);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// -----------------------------------------------------------------------------
// Incremental Test Baseline Manifest (.toksi/test-baseline.json)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteBaseline {
    pub watched_paths: Vec<String>,
    pub tree_blake3: String,
    pub last_passed_unix: i64,
    pub cargo_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBaselineManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub hash_algorithm_note: String,
    pub cargo_lock_blake3: String,
    pub suites: BTreeMap<String, SuiteBaseline>,
}

fn collect_files_sorted(root: &Path, rel_paths: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for raw in rel_paths {
        let p = root.join(raw);
        if p.is_file() {
            files.push(p);
        } else if p.is_dir() {
            collect_dir_recursive(&p, &mut files);
        }
    }
    files.sort_by(|a, b| {
        normalize_path(&a.to_string_lossy()).cmp(&normalize_path(&b.to_string_lossy()))
    });
    files.dedup();
    files
}

fn collect_dir_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut sorted: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    sorted.sort();
    for p in sorted {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == "target" || name == ".git" || name == ".toksi" {
            continue;
        }
        if p.is_dir() {
            collect_dir_recursive(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

pub fn compute_tree_blake3(root: &Path, watched_paths: &[String]) -> String {
    let files = collect_files_sorted(root, watched_paths);
    let mut buf = Vec::new();
    for f in files {
        let rel = f
            .strip_prefix(root)
            .map(|r| normalize_path(&r.to_string_lossy()))
            .unwrap_or_else(|_| normalize_path(&f.to_string_lossy()));
        buf.extend_from_slice(rel.as_bytes());
        buf.push(0);
        if let Ok(bytes) = fs::read(&f) {
            let h = blake3_hash(&bytes);
            buf.extend_from_slice(&h);
        }
        buf.push(b'\n');
    }
    blake3_hex(&buf)
}

pub fn compute_cargo_lock_blake3(root: &Path) -> String {
    compute_tree_blake3(
        root,
        &[
            "Cargo.lock".to_string(),
            "crates/toksi/Cargo.toml".to_string(),
        ],
    )
}

pub fn run_test_cli(args: &[String]) -> i32 {
    let force_all = args.iter().any(|a| a == "--all" || a == "--force");
    let root = PathBuf::from(".");

    // Always run workspace check and strict clippy
    let check_ok = Command::new("cargo")
        .args(["check", "--workspace"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !check_ok {
        eprintln!("[ERROR] cargo check --workspace failed");
        return 1;
    }

    let clippy_ok = Command::new("cargo")
        .args(["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !clippy_ok {
        eprintln!("[ERROR] cargo clippy --workspace --all-targets -- -D warnings failed");
        return 1;
    }

    let baseline_path = root.join(".toksi/test-baseline.json");
    let raw = match fs::read_to_string(&baseline_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Cannot read {}: {}", baseline_path.display(), e);
            return 1;
        }
    };
    let mut manifest: TestBaselineManifest = match serde_json::from_str(&raw) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[ERROR] Invalid {}: {}", baseline_path.display(), e);
            return 1;
        }
    };

    let current_lock_hash = compute_cargo_lock_blake3(&root);
    let lock_unchanged = current_lock_hash == manifest.cargo_lock_blake3;
    let now_unix = chrono::Utc::now().timestamp();

    let suite_names: Vec<String> = manifest.suites.keys().cloned().collect();
    let mut any_ran = false;

    for name in suite_names {
        let Some(suite) = manifest.suites.get(&name).cloned() else {
            continue;
        };
        let current_tree = compute_tree_blake3(&root, &suite.watched_paths);
        let unchanged = !force_all
            && lock_unchanged
            && suite.last_passed_unix > 0
            && current_tree == suite.tree_blake3;

        if unchanged {
            println!("[SKIP-UNCHANGED] {}", name);
            continue;
        }

        any_ran = true;
        println!("[RUN] {} -> cargo {}", name, suite.cargo_args.join(" "));
        let ok = Command::new("cargo")
            .args(&suite.cargo_args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            eprintln!("[FAIL] {}", name);
            return 1;
        }

        if let Some(entry) = manifest.suites.get_mut(&name) {
            entry.tree_blake3 = current_tree;
            entry.last_passed_unix = now_unix;
        }
    }

    manifest.cargo_lock_blake3 = current_lock_hash;
    manifest.hash_algorithm_note = "pure_rust_blake3".to_string();

    if any_ran
        && let Ok(serialized) = serde_json::to_string_pretty(&manifest)
    {
        let _ = fs::write(&baseline_path, serialized);
    }

    println!("[PASS] Incremental test baseline verified.");
    0
}
