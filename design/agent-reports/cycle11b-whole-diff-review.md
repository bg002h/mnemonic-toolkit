# WHOLE-DIFF ADVERSARIAL REVIEW — cycle-11b toolkit hygiene (L21 + L24 + L25)

Post-implementation mandatory review (worktree `wt-cycle11b`, off toolkit `origin/master = d35952ba` = v0.65.0; ships v0.65.1).

## VERDICT: GREEN (0 Critical / 0 Important)

All four axes verified against worktree source, plan/spec, and empirical test/clippy runs. Ready to ship.

### Axis 1 — L21 (SECRET/funds-footgun) — CLEAN
- **(a) Fires for all three sources.** Refusal at the composite `Bip38 =>` sub-arm head (`convert.rs:1380-1382`) INSIDE the `Seedqr | Phrase | Entropy =>` outer arm (`:1242`). Position-based — no `from`-set list that could drop seedqr. Seedqr RED (`composite_seedqr_to_bip38_unset_bip38_passphrase_refuses`) present + GREEN; `(Seedqr,Bip38)` is a permitted edge (`:648`) that decodes to a phrase and reaches the same arm. Phrase + entropy REDs GREEN.
- **(b) Fires before the empty-encrypt.** Refusal `:1380-1382`; `unwrap_or("")` at `:1402` — strictly after. No silent empty-passphrase ciphertext possible.
- **(c) `is_none()` not `is_empty()`.** `effective_bip38_passphrase = args.bip38_passphrase.clone()` (`:864`) → `bip38_passphrase = …as_deref()` (`:991`). `--bip38-passphrase ""` → `Some("")` → still encrypts. GREEN-1/1b/2 confirm explicit-empty (phrase + seedqr) + real-passphrase still encrypt + round-trip.
- **(d) Reuses `ConvertRefusal`, exit 2.** New helper `refusal_composite_bip38_no_bip38_passphrase()` → `ConvertRefusal`; no new variant/flag.
- **(e) Direct `(wif,bip38)`/`(bip38,wif)` arms unaffected** (`:1544`/`:1563`, `unwrap_or(pbkdf2_passphrase)`, no `is_none()` gate). `direct_wif_to_bip38_unset_bip38_passphrase_still_falls_back` GREEN. The `run()` guard at `:943` (`&&`-joined) does not pre-empt (with `--passphrase X` it doesn't fire → control reaches the new refusal).
- **(f) Message** names `--bip38-passphrase` + the `""` escape hatch (RED-1 dual-substring assert).
- **Pinned-test update legitimate.** `composite_phrase_to_bip38_separate_passphrase_semantics_pinned` previously relied on the silent empty-encrypt (the bug); now passes `--bip38-passphrase ""` explicitly and preserves the v0.8 §12.b invariant (`--passphrase` drives PBKDF2 only → `recovered == wif_b != wif_a`). No masked regression.

### Axis 2 — L24 (CLI-reachable panic → typed error) — CLEAN
Gate (`verify_bundle.rs:1353-1378`) mirrors `bundle.rs:1373-1387` byte-for-byte (only `args.slot` vs `slots`). Placed after `validate_slot_set` (`:1351`), before the canonicity probe (`:1387`)/`is_non_canonical` block (`:1399`). OOB write at `:1463`. RED fixture `wsh(andor(pkh(@0),after(12000000),pk(@1)))` (n=2) genuinely non-canonical; `@2={Phrase,Path}` required (`is_legal_set` `slot_input.rs:347-371` has `[Phrase,Path]` but no bare `[Path]` → path-only `@2` rejected at `validate_slot_set` first) — the co-located `@2.phrase` is what reaches `:1463`. Exact-coverage (`!= n`) regression GREEN (in-range `@0`/`@1` don't over-fire). Dummy `--mk1 "" --md1 ""` sentinels benign (empty-HRP exemption, fall through to descriptor-mode).

### Axis 3 — L25 (cosmetic err-msg) — CLEAN
Additive anchor `(?:tr|pk|pk_k|pk_h)\([0-9a-fA-F]{64}` appended to the alternation. 15 adversarial regex probes pass: `sha256/hash256/ripemd160/hash160` 64-hex stay keyless; `tr(`/`pk(`/`pk_k(`/`pk_h(` + 64-hex x-only becomes keyed; 66-hex `02/03` compressed (bare + in `pk(`) stay keyed; bare 64-hex token NOT matched; timelock-only keyless. Both `(false,false)` arms still `Err` — message-only, no `--json` change.

### Axis 4 — Scope / version / gates — CLEAN
16 files: 3 src + 2 test + 2 manual prose + FOLLOWUPS + bughunt report + 6 version/lock. No `lib.rs`, no fuzz source, no fmt churn. Version sweep consistent at 0.65.1 (Cargo.toml, both READMEs, install.sh, fuzz/Cargo.lock, Cargo.lock); CHANGELOG gate-format `^## mnemonic-toolkit [0.65.1]` present. No new `ToolkitError` variant; no clap/`--json`/dropdown → no schema_mirror; manual prose-only (sole flag-table touch is re-text of the existing `--bip38-passphrase` row — no NAME add/remove → flag-coverage lint GREEN). 3 FOLLOWUPS filed (2 RESOLVED + 1 OPEN `verify-bundle-bundle-rs-descriptor-mode-dedup` carrying L24's gate). `cargo test -p mnemonic-toolkit` GREEN (191 ok, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings` exit 0.

### Notes (non-blocking)
- The known pre-existing unrelated fuzz-build break (`lib.rs:170` cfg(fuzzing), E0433) — cycle-11b touched only `fuzz/Cargo.lock` (version bump), never lib.rs/fuzz source. Out-of-scope, pre-existing.
- Branch off d35952ba; current master ac81f111 added cycle-10/11a review files — disjoint, came along on the rebase.

## Disposition
GREEN — clear to push/tag `mnemonic-toolkit-v0.65.1`. Then cycle-10's pin-bump (md-codec 0.38→0.39) rides in as 0.65.2.
