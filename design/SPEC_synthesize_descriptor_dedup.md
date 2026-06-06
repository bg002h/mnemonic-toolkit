# SPEC — dedup `synthesize_unified` → delegate to `synthesize_descriptor`

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUP `synthesize-descriptor-deduplicate-with-unified`.
**Source SHA:** branch `synthesize-descriptor-dedup` off master `6506948`.
**SemVer:** PATCH — pure refactor, no user-visible behavior change. v0.47.0 → **v0.47.1**.

---

## 1. Summary

`synthesize_unified` (`synthesize.rs:745`) and `synthesize_descriptor` (`synthesize.rs:229`) share a **byte-identical back-half**: from a `Descriptor` + a cosigner list, compute `policy_id`→stub, emit per-slot `ms1` (Entr/Mnem by `language.unwrap_or(run_language)`), per-cosigner `mk1` (`Single` n==1 / `Multi`), and `md1 = chunk::split(descriptor)` → `Bundle`. The recon corrected the FOLLOWUP's framing:

- **Not a whole-function merge** — the two have different *front-halves*: `synthesize_descriptor(descriptor: &Descriptor, cosigners: &[CosignerKeyInfo], privacy, run_language)` is HANDED a pre-built descriptor + cosigners; `synthesize_unified(slots: &[ResolvedSlot], template, threshold, network, privacy, run_language)` *builds* the descriptor from `template`+`slots`+`threshold` (validation + path-decl + `Descriptor{…}` construction, `:753-817`).
- **The clincher (verified): `pub type CosignerKeyInfo = ResolvedSlot;` (`synthesize.rs:219`).** So `synthesize_descriptor`'s `cosigners: &[CosignerKeyInfo]` IS `&[ResolvedSlot]` — exactly what `synthesize_unified` already holds in `slots`. **No mapping, no bridge struct.**

**The dedup:** `synthesize_unified` keeps its front-half (build `descriptor`), then **replaces its entire back-half (`:819-896`) with one delegation** — `synthesize_descriptor(&descriptor, slots, privacy_preserving, run_language)`. Deletes ~78 lines.

## 2. The change — `synthesize.rs`

In `synthesize_unified`, after the `descriptor` is built (`:802-817`), replace everything from `:819` (`let policy_id = …`) through the final `Ok(Bundle { ms1, mk1, md1 })` (`:896`) with:

```rust
    // The card-emission back-half (policy_id → ms1 → mk1 → md1) is identical to
    // synthesize_descriptor; `slots: &[ResolvedSlot]` IS `&[CosignerKeyInfo]`
    // (`type CosignerKeyInfo = ResolvedSlot`), so delegate (FOLLOWUP
    // `synthesize-descriptor-deduplicate-with-unified`).
    synthesize_descriptor(&descriptor, slots, privacy_preserving, run_language)
```

- The now-dead local `let stubs: Vec<[u8;4]> = vec![stub; n];` (`:822`) + `let mut stub`/`policy_id` (`:819-821`) are deleted (they were only consumed by the deleted back-half; `synthesize_descriptor` recomputes them).
- `synthesize_descriptor`'s leading count-check (`cosigners.len() != descriptor.n`, `:235`) passes trivially — `synthesize_unified` built `descriptor.n = n = slots.len()`. Its `debug_assert!(descriptor.is_wallet_policy())` holds (template-built wallet policy).
- `synthesize_descriptor` is **unchanged** (it already IS the shared implementation).

## 3. Why this is behavior-preserving (R0 to confirm byte-identity)
The two back-halves are byte-identical given the same `(descriptor, &[ResolvedSlot])`:
- **policy_id/stub:** both `compute_wallet_policy_id(descriptor)` → first 4 bytes.
- **ms1:** both iterate the cosigner/slot list; `Some(entropy)` → `Entr` (English) / `Mnem{wire_lang, entropy}` (else) via `language.unwrap_or(run_language)`; `None` → `String::new()`. (`synthesize_descriptor:296-318` ≡ `synthesize_unified:827-851`.)
- **mk1:** `n==1` → `Single` with `vec![stub]`, fingerprint-or-`None` (privacy), `mk1_origin_path(&xpub,&path)`, xpub, csi `derive_mk1_chunk_set_id(&stub)`; `n>1` → `Multi` per-cosigner with `stubs.clone()`, csi `derive_mk1_chunk_set_id(&xpub.fingerprint().to_bytes())`. (`synthesize_descriptor:249-289` ≡ `synthesize_unified:854-889`.)
- **md1:** both `md_codec::chunk::split(&descriptor)`.
- The cosigner field reads (`.fingerprint`/`.xpub`/`.path`/`.entropy`/`.language`) are identical because `CosignerKeyInfo == ResolvedSlot`. `master_xpub`/`_entropy_pin` are NOT read by either back-half.
- **(R0 M1) Statement-order difference is immaterial.** `synthesize_unified` computes ms1→mk1→md1; `synthesize_descriptor` md1→mk1→ms1. The three `Bundle` fields are independent + the iteration order over the slice is identical → byte-identical `Bundle`. The only observable difference would be which internal encoder error surfaces first if two failed simultaneously — unreachable on a freshly-built `is_wallet_policy`-asserted descriptor. Not a behavior change.

## 4. Tests
- **Green-stays-green:** the `synthesize_unified_*` cells (`:1601-1700`) + the `synthesize_descriptor_*` cells (`:1378-1460`) + every `bundle`/`verify-bundle` end-to-end test that exercises `synthesize_unified` (multisig/template-mode bundles) cover the back-half byte-shape. A behavior-preserving refactor → all stay green. Run the full workspace suite + `make -C docs/manual verify-examples` (the foreign-format/cross-recipe transcripts that round-trip template-mode bundles).
- **(R0 I1) A multisig (n>1) byte-shape characterization cell IS required — captured PRE-edit.** The n==1 `Single` branch is well-guarded by the frozen 16-cell golden (`cli_bundle_full.rs:14-37`, `tests/vectors/v0_1/*.txt`). But the **n>1 `Multi` branch has NO byte-exact golden** — the `synthesize_unified_*` multisig cells (`:1637-1693`) assert only `ms1.len()`/`starts_with`/`any_secret_bearing`, the `bundle|verify-bundle` round-trips co-move (both call `synthesize_unified`), and the multisig self-check golden was deleted in v0.4.2 (orphaning the `tests/vectors/v0_2/` multisig fixtures). So a Multi-branch drift would ship silently. **Phase 1 adds ONE characterization cell** capturing the CURRENT `synthesize_unified` n>1 Bundle byte-shape (a 2-of-2 `wsh-sortedmulti` with two distinct phrases — `TREZOR_12_ZERO` + a second distinct test phrase, satisfying BIP-388 distinctness), asserting byte-exact on **ms1[0] + ms1[1] + mk1[0] + mk1[1] + md1**. GREEN against the current binary (captures current output); stays GREEN after §2 (proving no Multi-branch drift) — any csi/ordering/stub change goes RED. This is the load-bearing guard the SPEC's original "existing cells guard it" claim falsely assumed.
- Full `cargo test --no-fail-fast` + clippy `--all-targets` GREEN (clippy will flag the now-unused `stub`/`stubs` if any survive → remove).

## 5. Lockstep / scope
- **NONE.** No clap flag/value/subcommand change → no GUI `schema_mirror`, no manual mirror, no sibling-codec change. No new error variant. Pure internal refactor of one function body.
- The ~9 call sites (`synthesize_descriptor` ×5: `bundle.rs:1563`/`:1641`/`:1882`, `import_wallet.rs:1398`, `verify_bundle.rs:1002`; `synthesize_unified` ×4: `bundle.rs:399`, `verify_bundle.rs:374`/`:464`/`:568`) are UNCHANGED — they call the two public fns, whose signatures are unchanged; only `synthesize_unified`'s body shrinks.

## 6. Phased plan
- **Phase 1 (characterization, R0 I1):** add ONE multisig (n>1) byte-shape characterization cell (§4) capturing the CURRENT `synthesize_unified` n>1 Bundle bytes (2-of-2 `wsh-sortedmulti`, two distinct phrases; assert ms1[0]+ms1[1]+mk1[0]+mk1[1]+md1 byte-exact). Captured + committed BEFORE the §2 edit. GREEN now; the guard that proves the Multi branch is behavior-preserved after §2. (The n==1 path is already covered by the frozen 16-cell golden.)
- **Phase 2 (GREEN):** §2 edit. Full workspace `cargo test --no-fail-fast` + clippy + `make verify-examples` GREEN. Per-phase opus review → persist.
- **Phase 3 (release):** CHANGELOG `[0.47.1]`; version v0.47.0 → **v0.47.1** (Cargo.toml/lock + 2 READMEs + install.sh self-pin); FOLLOWUP `synthesize-descriptor-deduplicate-with-unified` → resolved. Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.47.1` → push → watch CI (rust, install/sibling-pin-check; manual fires only if a manual file changed — none here).

## 7. Risk
Very low. A 1-function-body shrink that delegates to an already-tested public fn; the back-halves are byte-identical (the `CosignerKeyInfo == ResolvedSlot` alias makes the delegation typecheck with zero conversion). The full multisig/template bundle + verify-bundle suites + verify-examples transcripts are the behavior guard. R0 MUST confirm: (i) the two back-halves are genuinely byte-identical (esp. the mk1 csi derivation + the ms1 Entr/Mnem branch); (ii) `synthesize_unified`'s front-half builds a `descriptor` whose `.n` == `slots.len()` so the delegated count-check passes; (iii) a `synthesize_unified` multisig test actually pins the Bundle byte-shape (else add a characterization cell before the edit).
