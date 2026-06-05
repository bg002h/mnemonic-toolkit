# End-of-Cycle Architect Review — v0.44.0 release prep (multisig restore)

**Reviewer:** opus `feature-dev:code-reviewer` (final gate before tag; scoped to Phase 3/4 prose + coherence)
**Date:** 2026-06-05
**Branch:** `restore-multisig-cosigner`
**Verdict:** 0 Critical / 2 Important / 0 Minor — **GATE: RED** (both folded → GREEN)

> Persisted verbatim per CLAUDE.md. Both Important items were folded (prose-only) after this review; manual lint re-run clean (0 errors).

---

## Critical
None.

## Important

**1. `docs/manual/src/30-workflows/35-recovery-paths.md:84-89` — recovery-paths chapter still said multisig restore "is a planned follow-on addition" (contradicts the shipped feature).** The "Multisig note" read "restore is single-sig this release … planned follow-on addition." This contradicts the v0.44.0 CHANGELOG + the new `41-mnemonic.md#multisig-cosigner-restore` section, and is a missed SPEC §8 Phase 3 deliverable (the recovery recipe was never landed). Stale prose tells a recovering user the capability does not exist. **Fix:** replace with a short worked multisig-restore pointer cross-linking the CLI section.

**2. `docs/manual/src/40-cli-reference/41-mnemonic.md:920-922` — worked-example cosigner table printed `[m/87'/0'/0']`; the binary emits `[87'/0'/0']` (no `m/`).** The table renders `c.origin` (a `bitcoin::bip32::DerivationPath`) via `restore.rs:1066-1072`; in `bitcoin 0.32.8` `DerivationPath`'s `Display` writes components joined by `/` with NO leading `m`. So the real output is `[87'/0'/0']`. (Distinct from the descriptor key-origin `[73c5da0a/87'/0'/0']` in the same example, which is correct BIP-380.) No test pins the bracket format. **Fix:** change the three table lines to `[87'/0'/0']`.

## Minor
None worth blocking. (SPEC §1 line 17's single-path `…/0/*` descriptor illustration is an early-framing abbreviation; SPEC §8 / CHANGELOG / manual all correctly say multipath `<0;1>/*`, test-backed.)

## What verified clean (no action)

- **CHANGELOG `[0.44.0]`** — every factual claim checked against source: wallet-policy reframe, the `to_miniscript_descriptor`→`template_from_descriptor`→`build_descriptor_string` pipeline + multipath `<0;1>/*`, PARTIAL semantics ("only positions you supply are verified" — matches `restore.rs:989-1016`), 65-byte cross-check, watch-only-out, taproot refusal reason, `--from required_unless_present="md1"`. Accurate.
- **`41-mnemonic.md` flag-table rows + synopsis** — `--md1`/`--cosigner` present, match clap; flag-coverage lint satisfied. Text worked-example wording matches `restore.rs:1056-1073` byte-for-byte except the `[m/…]` bracket (Important #2).
- **FOLLOWUPS** — `restore-multisig-cosigner-scope` flipped resolved with accurate option-(c)/inversion/C1 prose; the 3 spawned entries accurate + cite the right blockers; all 4 cited audit-trail files exist.
- **Version bumps** — consistently `0.44.0` at all five sites (`Cargo.toml:3`, `Cargo.lock:706`, `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, `scripts/install.sh:32`); only lingering `0.43.1` is the correctly-retained Cycle-A entropy-arm comment.
- **SPEC §7 lockstep coherence** — matches reality: `lint_argv_secret_flags` NO route change (`--md1`/`--cosigner` non-secret; `restore --from`/`--passphrase` routes pre-exist); GUI deferred to FOLLOWUP (correct — `schema_mirror` cannot lead the pin); `cli_gui_schema.rs` auto-derives flags from clap (count stays 29). `--md1`/`--cosigner` correctly `secret:false` everywhere.

VERDICT: 0 Critical / 2 Important
GATE: RED → both Important items are localized manual-prose edits; fold both, re-run flag-coverage, then GREEN for tag.

---

## Fold note (applied after persisting)

- **Important #1 — FOLDED.** `35-recovery-paths.md` "Multisig note" replaced with a real `mnemonic restore --md1` multisig-recovery pointer (md1-alone reconstruction; optional `--from`/`--cosigner` per-position cross-check; PARTIAL until all verified; wsh/sh-wsh, taproot refused) cross-linking `#multisig-cosigner-restore`.
- **Important #2 — FOLDED (runtime-verified).** Ran a real `restore --md1` and confirmed the cosigner table emits `[87'/0'/0']` (no `m/`); fixed the three table lines `[m/87'/0'/0']` → `[87'/0'/0']` and the illustrative checksum `#y65a0dtg` (single-path probe) → `#yjp7hj7w` (the real multipath descriptor checksum).
- Manual flag-coverage + link lint re-run: 0 errors, `#multisig-cosigner-restore` anchor resolves. GATE GREEN — cleared for tag.
