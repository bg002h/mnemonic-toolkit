//! First-party `unsafe` SAFETY-comment discipline for `src/mlock.rs`.
//!
//! Peer of `lint_safety_third_party_blocked.rs` (which covers third-party-
//! blocked patterns at `Xpriv::derive_priv`, `SecretKey::from_slice`,
//! `bip39::Mnemonic` constructor sites). This lint scans the toolkit's own
//! `unsafe {` opener tokens in `src/mlock.rs` and requires a `// SAFETY:`
//! doc-comment within ±5 lines above each.
//!
//! Per Cycle B Phase 2 R0 I-R0-3 fold (commit `8193e22`). Under Fix B the
//! mlock module has exactly 2 `unsafe` blocks: the `libc::mlock` syscall in
//! `pin_pages_for` and the `libc::munlock` syscall in `PinnedPageRange::drop`.

use std::fs;

const MLOCK_SRC: &str = "src/mlock.rs";
const WINDOW_LINES_ABOVE: usize = 5;

#[test]
fn every_first_party_unsafe_block_has_safety_comment_within_window() {
    let source =
        fs::read_to_string(MLOCK_SRC).unwrap_or_else(|e| panic!("read {}: {}", MLOCK_SRC, e));
    let lines: Vec<&str> = source.lines().collect();
    let mut violations: Vec<String> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !is_unsafe_opener(trimmed) {
            continue;
        }
        let lo = idx.saturating_sub(WINDOW_LINES_ABOVE);
        let has_safety = lines[lo..idx].iter().any(|l| {
            let t = l.trim_start();
            t.starts_with("// SAFETY:") || t.starts_with("//! SAFETY:")
        });
        if !has_safety {
            violations.push(format!(
                "{}:{}: `unsafe {{` opener missing `// SAFETY:` comment within prior {} lines",
                MLOCK_SRC,
                idx + 1,
                WINDOW_LINES_ABOVE,
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "first-party unsafe SAFETY-comment discipline violations:\n  {}",
        violations.join("\n  "),
    );
}

/// True when the (trimmed) line opens an unsafe block as a bare statement.
/// Matches `unsafe {` (with optional whitespace before `{`). Excludes false
/// positives like `let foo = unsafe_fn(...)`, `// unsafe ...` comments,
/// `pub unsafe fn ...` function-signature uses (which carry their own SAFETY
/// discipline at the caller site).
fn is_unsafe_opener(trimmed: &str) -> bool {
    if trimmed.starts_with("//") || trimmed.starts_with("/*") {
        return false;
    }
    // Allow leading-statement prefixes like `let x = ` before `unsafe {`.
    let needle = "unsafe {";
    let Some(idx) = trimmed.find(needle) else {
        return false;
    };
    // Reject `pub unsafe fn ...` / `unsafe fn ...` signatures.
    if trimmed.contains(" fn ") || trimmed.starts_with("fn ") {
        return false;
    }
    // Reject `unsafe trait` / `unsafe impl` declarations.
    if trimmed.contains("trait ") || trimmed.contains("impl ") {
        return false;
    }
    // Reject identifiers like `is_unsafe_opener` — needle must be at a word
    // boundary at the start (preceded by start-of-line or whitespace/punct).
    if idx > 0 {
        let prev = trimmed.as_bytes()[idx - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return false;
        }
    }
    true
}
