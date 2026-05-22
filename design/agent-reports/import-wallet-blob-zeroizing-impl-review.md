# `import-wallet-blob-zeroizing` implementation — opus end-of-cycle review (verbatim)

Review of the uncommitted migration (`blob: Vec<u8>` → `Zeroizing<Vec<u8>>` in `cmd/import_wallet.rs`), feature-dev:code-reviewer (opus). Persisted per CLAUDE.md. **VERDICT: GREEN (0 Critical / 0 Important / 0 Minor). Cleared to ship v0.33.3.**

All six confirmation points verified against source:
1. Three `blob =` sites only (`:390` via `read_blob`, `:434` BIE1, `:1043` BSMS); `:2452` is a test literal. Both secret-bearing reassigns land in `Zeroizing` (`:434` `blob = plaintext;` — plaintext already `Zeroizing<Vec<u8>>`; `:1043` `Zeroizing::new(plaintext.into_bytes())`). No third reassign missed.
2. `read_blob` (`:2082`) returns `Zeroizing<Vec<u8>>` both arms; stdin arm reads into `Zeroizing::new(Vec::new())` via `read_to_end(&mut buf)` (works through `DerefMut<Target=Vec<u8>>`).
3. BIE1 pin reorder sound: `trimmed = from_utf8(&blob)` (`:420`) last used at `:423` before the `:434` move (no live borrow); `_pin_pt = pin_pages_for(&blob)` post-move; owned guard, Vec move preserves heap buffer → pinned pages correct. No use-after-move.
4. No read site broke: 8 parse arms (`:1047-1054`), `detect_storage_magic` (`:400`), `sniff_format` (`:460`), `from_utf8` (`:420`/`:1028`), roundtrip `&blob` (`:1158`/`:1167`) all target `&[u8]` via multi-step deref coercion. No by-value move; no inherent-`Vec`-method needing `&Vec<u8>`.
5. No new un-scrubbed copy (lossy `.to_vec()` dropped; BSMS re-wraps in place). M1 (`decrypt_bsms_record` → plain `String`) + M2 (non-BIE1 blob unpinned) are pre-existing out-of-scope follow-ons, not regressions.
6. SemVer PATCH correct; `use zeroize::Zeroizing;` folds M3; no CLI/wire/GUI/manual/schema-mirror surface.

2253/0 + clippy clean (compiler-as-backstop, per the type-level plan).
