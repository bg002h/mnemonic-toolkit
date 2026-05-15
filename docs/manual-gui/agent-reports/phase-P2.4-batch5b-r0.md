# Phase P2.4 sub-batch 5b (Track M — 40-mnemonic bundle) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** 5b — `docs/manual-gui/src/40-mnemonic/42-bundle.md` (NEW, 430 lines, 15 flags); `.cspell.json` (+3 words).

**Verdict:** **ITERATE 4C / 1I / 0N / 2n.**

The chapter has correct outline structure (15-bullet subcommand outline; per-Dropdown 4/10/10/2 variant outlines), correct conditional-visibility prose, correct slot-editor render description (modulo enum-count finding), correct cross-references, correct worked-example output strings. **But four byte-exactness drifts** in source-pinned content (`mode_text` constants, `BundleJson` schema, `SlotSubkey` enum) plus a worked-example invocation that would be refused by the CLI. Per the cycle's `[[feedback-architect-must-run-prose-commands]]` discipline and SPEC §6.6/§6.9 byte-pinning, these are all Critical.

## Critical

### C-1 — JSON envelope `schema_version` and field shape wrong

`42-bundle.md:320-339` showed `schema_version: "2"` with 8 fields. Source (`crates/mnemonic-toolkit/src/format.rs:120-145` `BundleJson`) is `schema_version: "4"` with 14 fields in canonical order: `schema_version`, `mode`, `network`, `template`, `descriptor`, `account`, `origin_path`, `origin_paths`, `master_fingerprint`, `ms1`, `mk1`, `md1`, `multisig`, `privacy_preserving`. Five fields absent (`mode`, `template`, `descriptor`, `account`, `privacy_preserving`); field order wrong.

### C-2 — SlotSubkey enum is 8 variants, not 7 (missing `master_xpub`)

`42-bundle.md:412-423` enumerated 7 subkeys. Source (`crates/mnemonic-toolkit/src/slot_input.rs:17-32` `SlotSubkey` enum) has 8: `Phrase, Entropy, Xpub, MasterXpub, Fingerprint, Path, Wif, Xprv`. The toolkit's parser refusal message lists only 7 (a CLI manual drift); the GUI manual must document all 8.

### C-3 — Refusal table mode-violation strings paraphrased, not byte-exact

`42-bundle.md:494-499` invented strings like `mode: descriptor — flag --template is incompatible ...`. Source (`crates/mnemonic-toolkit/src/cmd/bundle.rs:91-101` `mode_text::*`) constants are SPEC §6.6/§6.9 byte-pinned with integration-test harnesses. All 6 mode-violation rows must mirror the constant exactly.

### C-4 — `--account` body refusal has broken inline newline + invented string

`42-bundle.md:307-311` had a literal newline inside backticks producing broken HTML/PDF render. Replaced with byte-exact `mode_text::DESCRIPTOR_WITH_NONZERO_ACCOUNT`.

## Important

### I-1 — Worked-example invocation includes `--multisig-path-family bip87` for single-sig BIP-84, which is refused

`42-bundle.md:460` Preview showed `mnemonic bundle ... --template bip84 --account 0 --multisig-path-family bip87 --slot ...`. Source (`bundle.rs:186-192`) refuses this via `mode_text::PATH_FAMILY_WITHOUT_MULTISIG`. The CLI exits 2 with a mode-violation error. The worked example as written would not produce the documented output.

Root cause: GUI seeds `--multisig-path-family bip87` as a default (`main.rs:188-211`); the conditional-visibility engine does not yet auto-disable this flag for single-sig templates. Documented workaround (clear the field manually) folded into the worked example; FOLLOWUP `gui-bundle-multisig-flags-conditional` filed in the GUI repo.

## Nitpicks

### n-3 — `1..16` Rust-half-open vs `1..=16` inclusive ambiguity

`--threshold` body said "Allowed range 1..16" — Rust readers would interpret as exclusive of 16. Replaced with "Allowed range 1 to 16 inclusive".

### n-4 — `:::danger` admonition lacked link to §14 Defense 2

The cross-reference was prose-only. Made it a markdown link to match the consistent pattern from earlier chapters.

## Lint state

- Phase 4 schema-coverage RED at **403 missing** (was 445 → -42 = 1 sub + 15 flags + 26 variants for bundle).
- Phase 5 outline-coverage RED at **52 missing** (was 57 → -5 = 1 subcommand-outline + 4 flag-outlines).
- Phases 1-3 GREEN; phases 6-7 WARN-skip.
- HTML 17 H1 chapters (was 16 → +1).
- PDF 64 pages post-fold (was 50 at sub-batch 5a → +14).

After folds, R1 should LOCK.
