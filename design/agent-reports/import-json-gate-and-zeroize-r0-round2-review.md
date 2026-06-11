# R0 Review — import-json schema-gate (A) + sp/nostr secret-zeroize (B) — ROUND 2 (GREEN)

**Source SHA:** `91a2b20`. Re-review after folding all round-1 findings.

**Verdict: 🟢 GREEN (0 Critical / 0 Important).** Implementation may proceed.

## Fold confirmations
- **I1 (README/install.sh self-pin):** all three sites confirmed at `0.53.5` — `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, `scripts/install.sh:32`. Ritual now lists all three. Accurate + complete.
- **I2 (T-A3 fixture):** single-entry `"1"/"4"` raw + `index=None` → the `None` arm returns `Ok` only when `len==1`; `validate_schema_versions` runs on the valid entry → `Ok`. A genuine accept/no-regression path, not a reject path. Accurate.
- **m1:** text-path citation corrected to `silent_payment.rs:294-295` (the scan_priv/spend_priv writeln). Confirmed.
- **m2:** T-B3 removal correct; compile-time type enforcement via T-B4 anchors is the structural guarantee.
- **m3:** `wif_for` returns `String` → `SecretString::new(wif_for(...))` wraps correctly; `wif.clone()` is a `SecretString`; `format!("{p}{wif}")` via `Display`. Consistent with the `:231`/`:221` `Option<SecretString>` targets.
- **m4:** range widen `(18..=35)` → `(18..=42)` is correct and sufficient regardless of whether current count is 31 or 32 (resulting 35/36 ≤ 42).

## Note-quality observations (NOT findings; compile-time self-correcting — no fold required)
- The Part B PROBLEM section describes the CURRENT code (`electrum = format!("{p}{wif}")`) — correct as a description of the existing leak; the Design/`:221 build` prescribes `SecretString::new(format!("{p}{wif}"))` as the fix. The type system enforces the `Option<SecretString>` field at build, so any literal mis-transcription fails to compile.
- The exact `ZEROIZE_ROWS.len()` (31 vs 32) is confirmed at impl time via `cargo test`; the `(18..=42)` bound has headroom either way.

Design holds; A+B coherent as one PATCH cycle; SemVer PATCH; no schema_mirror/manual/GUI/sibling lockstep.
