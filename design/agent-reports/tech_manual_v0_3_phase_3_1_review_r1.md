# tech-manual v0.3 — Phase 3.1 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.3.0` (in progress) |
| Phase | 3.1 (Part IV §IV.1 — Bundle Anatomy) |
| Commit under review | `f2c54e2` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `docs/technical-manual/src/40-bundle-formation/41-bundle-anatomy.md` · `docs/technical-manual/src/60-back-matter/62-index-table.md` (+19 rows) · `docs/technical-manual/transcripts/mnemonic-bundle-bip84-abandon.{cmd,out}` · `docs/technical-manual/transcripts/mnemonic-verify-bundle-bip84-abandon.{cmd,out}` · `docs/technical-manual/.cspell.json` (+1 word "subkeys") · figure cache PDFs |

## Findings: 0 Critical / 2 Important / 2 Low / 2 Nit

---

## Important

**I-1. `expected.ms1[i].is_empty()` misstates the actual access pattern at `verify_bundle.rs:621` (confidence: 90)**

`41-bundle-anatomy.md:112`:

> verify-bundle's `emit_verify_checks` discriminates on `expected.ms1[i].is_empty()` (`verify_bundle.rs:621`)

Actual code at `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:621`:

```rust
let watch_only = expected.ms1.first().map(|s| s.is_empty()).unwrap_or(true);
```

The code uses `.first()`, not index-based `[i]`. The `[i]` notation implies a per-slot loop, whereas line 621 is in the single-sig arm of `emit_verify_checks` (entered only when `is_multisig == false`) and unconditionally takes the zeroth element via `.first()`. A reader tracing the multi-slot slot-index correspondence `ms1[i] ↔ mk1[i]` from this citation will not find indexed access at line 621 — the multi-slot variant lives in `emit_multisig_checks`, not here.

Fix: Change `expected.ms1[i].is_empty()` to `expected.ms1.first().map(|s| s.is_empty())` and add a parenthetical noting this is the single-sig arm; the per-cosigner multi-index equivalent is in `emit_multisig_checks`.

---

**I-2. Engraving-card section intro calls all identifiers "4-hex `chunk_set_id`" — wrong for mk1/ms1 (confidence: 85)**

`41-bundle-anatomy.md:118`:

> It is the human's index card to the physical bundle, identifying each plate by its 4-hex `chunk_set_id`.

mk1 and ms1 identifiers are 5 hex chars (20 bits), computed via `derive_mk1_chunk_set_id` and formatted `{:05x}` at `bundle.rs:724`. Only the md1 identifier is 4 hex chars (16 bits), formatted `{:02x}{:02x}` at `bundle.rs:707`. Confirmed by `mnemonic-bundle-bip84-abandon.out`:

```
# ms1: 1c017      ← 5 hex
# mk1: 1c017      ← 5 hex
# md1: 1c01       ← 4 hex
```

The paragraph at line 138 and the worked example at line 193 both correct this accurately, but the section's lead sentence states "4-hex" as if universal. A reader who reads only the intro before the sub-paragraphs will have the wrong model.

Fix: Change "identifying each plate by its 4-hex `chunk_set_id`" to "identifying each plate by its `chunk_set_id` (4 hex chars for md1, 5 hex chars for mk1/ms1)".

---

## Low

**L-1. Worked-example description calls the `.out` transcripts "stdout" — they capture stdout+stderr combined (confidence: 85)**

`41-bundle-anatomy.md:189`:

> The full invocation and stdout are captured at `transcripts/mnemonic-bundle-bip84-abandon.cmd`/`.out`

`docs/technical-manual/tests/verify-examples.sh:53` runs each cmd as:

```bash
actual=$(bash -c "$cmd_line" 2>&1 || true)
```

The `2>&1` merges stderr into stdout before capture. For the bundle transcript, the engraving card (which the chapter's own text describes as "stderr-only emission") and the `warning: secret material on stdout` line both appear in `mnemonic-bundle-bip84-abandon.out`. Calling these files "stdout" captures misrepresents what they contain and contradicts the chapter's own claim that the engraving card goes to stderr.

Fix: Change "The full invocation and stdout are captured" to "The full invocation and combined stdout+stderr output are captured".

---

**L-2. `ResolvedSlot` field list omits `path_raw` (confidence: 80)**

`41-bundle-anatomy.md:87`:

> `synthesize_unified` … takes a `Vec<ResolvedSlot>` (one entry per slot, carrying `xpub`, `fingerprint`, `path`, and optional `entropy`; `synthesize.rs:568-582`)

Actual struct at `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:568-576` has five fields: `xpub`, `fingerprint`, `path`, `path_raw`, `entropy`. The `path_raw: String` field (preserving the user-supplied raw path string for SPEC §4.11.b raw-equality) is absent from the description. A reader following the source pointer would see a field not accounted for.

Fix: Change "carrying `xpub`, `fingerprint`, `path`, and optional `entropy`" to "carrying `xpub`, `fingerprint`, `path`, `path_raw`, and optional `entropy`".

---

## Nit

**N-1. Single-sig card described as always emitting "four direct lines" — fingerprint and origin path are conditional (confidence: 80)**

`41-bundle-anatomy.md:136`:

> The single-sig card collapses the cosigners block to four direct lines (`ms1`, `mk1`, `fingerprint`, `origin path`; `format.rs:310-324`).

`format.rs:318-323`:

```rust
if let Some(fp) = &blk.fingerprint {
    s.push_str(&format!("# fingerprint: {}\n", fp));
}
if let Some(p) = &blk.origin_path {
    s.push_str(&format!("# origin path: {}\n", p));
}
```

Both `fingerprint` and `origin path` are guarded by `if let Some(...)`. Under `--privacy-preserving`, `blk.fingerprint` is None and the fingerprint line is absent; for certain slot shapes `origin_path` can also be None. "Four direct lines" overstates the guarantee.

Fix: Change "four direct lines" to "up to four lines (fingerprint and origin path omitted when absent)".

---

**N-2. `chunk_set_id_extract` lower bound off by one — doc-comment opens at line 379, not 380 (confidence: 80)**

`41-bundle-anatomy.md:138`:

> A separate helper `chunk_set_id_extract` (`format.rs:380-395`)

`/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/format.rs:379`:

```
/// Extract a chunk_set_id from an mk1 chunked-header string per SPEC §2.2.1
```

The doc-comment block opens at line 379; line 380 is its second continuation line. The function signature is at line 385. The cited range `380-395` clips the opening doc-comment line.

Fix: Change `format.rs:380-395` to `format.rs:379-395`.

---

## Resolution (Phase 3.1 close)

All six findings folded inline at the closing commit. None deferred — fix sizes are uniformly small one-line edits, and inline folding keeps `FOLLOWUPS.md` lean.

---

## Verified-correct items (no action needed)

- `schema_version: "4"` at `bundle.rs:572` and `verify_bundle.rs:182` — both confirmed exact.
- `BundleMode` enum `bundle_unified.rs:14-26`, `detect_bundle_mode` `34-63`, `pre_check_threshold` `67-90`, `pre_check_template_n` `94-112` — all confirmed exact.
- `SlotSubkey::is_secret_bearing` at `slot_input.rs:47-49` — confirmed.
- `synthesize_unified` at `synthesize.rs:593`, `Bundle` struct `20-28`, `any_secret_bearing` `33-35`, `ResolvedSlot` `568-582`, `derive_mk1_chunk_set_id` `42-44` — all confirmed.
- `MsField` `format.rs:42-54`, `MkField` `64-70`, `MultisigInfo` `103-111`, `BundleJson` `119-145`, `VerifyBundleJson` `148-153`, `VerifyCheck` `165-183`, `engraving_card_unified` `259-376`, `BundleInputForCard` `222-233`, `DESCRIPTOR_MAX_INLINE = 80` at line `260` — all confirmed exact.
- `verify_bundle.rs:98-201` dispatch function boundaries — confirmed. Watch-only discriminator at line 621 (correct line number). Skip-check block `623-637` (actual range through closing brace at 638, half-line imprecision below Nit threshold; the cited range still puts a reader at the right code).
- md1 4-hex at `bundle.rs:707`, mk1/ms1 5-hex at `bundle.rs:724` — confirmed.
- `chunk_set_id_extract` is imported and used at `verify_bundle.rs:11` and `869` respectively. The `#[allow(dead_code)]` on the function definition at `format.rs:384` is a pre-existing stale attribute, not introduced by this chapter.
- Cross-card binding claim: `1c017` / `1c017` / `1c01` from transcript confirm shared leading-16-bits invariant. "Ten-line `ok` log" claim: transcript has exactly 9 named check lines + `result: ok` = 10 lines. Confirmed.
- All 19 new `\index{}` terms have matching rows in `62-index-table.md` pointing to `Bundle Anatomy`.
- `BundleJson` field order in prose table matches struct definition order in `format.rs:120-144`.
- `CosignerEntry` fields (`index`, `master_fingerprint`, `origin_path`, `xpub`) confirmed at `format.rs:93-100`.
- Hardware-wallet caveat `format.rs:367-373` for `tr-multi-a`/`tr-sortedmulti-a` — confirmed exact range and template names.
- `MultisigHybrid` watch-only skip sentinel (`passed: true, decode_error: Some("skipped: watch-only slot")`) confirmed at `verify_bundle.rs:625-638`.
