# `import-wallet-blob-zeroizing` plan-doc — opus R0 (verbatim)

Review of `design/PLAN_import_wallet_blob_zeroizing.md` (feature-dev:code-reviewer, opus). Persisted per CLAUDE.md. **VERDICT: GREEN (0 Critical / 0 Important / 3 Minor).** No R1 needed (no Critical/Important; Minors don't alter the plan).

## Verified (all confirmed vs `db9657f`)
1. **Deref coercion** correct for ALL read sites — every `&blob` consumer takes `&[u8]` (`detect_storage_magic`, `sniff_format` sniff.rs:84, the 8 `WalletImportParser::parse(blob: &[u8])` arms mod.rs:52, `emit_json_envelope` :1230, `emit_roundtrip_stderr_warning` :1884, `from_utf8`). Multi-step `&Zeroizing<Vec<u8>> → &[u8]` applies. `grep blob\.` returns only comments → NO inherent-`Vec` method call, NO by-value move.
2. **Two reassigns** both correct — `:430` `plaintext` is `Zeroizing<Vec<u8>>` (drop the lossy `.to_vec()` → `blob = plaintext;`); `:1036` `plaintext` is `String` (`decrypt_bsms_record` :2146) → `Zeroizing::new(plaintext.into_bytes())`.
3. **mlock reorder** sound — `pin_pages_for(&[u8]) -> PinnedPageRange` (mlock.rs:90) returns an OWNED guard (raw ptr + page_count, no borrow); Vec move preserves the heap buffer → pinning `&blob` post-move valid.
4. **`mut`** fine — no `&mut blob`/DerefMut site.
5. **SemVer PATCH** + no lockstep correct (internal type change only).
6. **Test posture** acceptable (type guarantee + 2253-cell regression; matches `resolved-slot-derived-account-zeroizing-field` precedent — no runtime zeroize cell).

## Minor
- **M1** (out of scope; file follow-on): `decrypt_bsms_record` (:2146) returns plain `String` — the intermediate descriptor plaintext is un-zeroized before the `:1036` wrap. Low sensitivity (watch-only descriptor). Note as follow-on, don't expand scope.
- **M2** (out of scope; file sibling FOLLOWUP): the non-BIE1 plaintext-Electrum path (seed-bearing) + other formats never `pin_pages_for(&blob)` — only the BIE1 branch pins. Real pre-existing gap, orthogonal to zeroize-on-drop. File sibling FOLLOWUP (`import-wallet-plaintext-blob-mlock-pin`).
- **M3** (fold into impl): prefer `use zeroize::Zeroizing;` over scattered fully-qualified paths (file currently uses inline `zeroize::Zeroizing` at :2031). Either is clippy-clean; be consistent.
