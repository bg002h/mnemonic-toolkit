# AUDIT FINDINGS — manual-v0.3.0 cycle (Cycle 4 / Wave 2 second)

**Phase:** P0 recon (this doc) → P1a captures (4 transcript recaptures) → P1b classification → P2 prose fixes → P3 SKIP_STEMS removal → P4 cycle close.
**Working SHA at P0 recon:** `eaab552` (post-Cycle 3 ship; GUI side) / `826efbc` (toolkit side).
**Binary under test:** `target/debug/mnemonic` (mnemonic 0.28.4); `md 0.6.0`; `ms 0.4.0`; `mk 0.4.1`.

---

## P0 recon — scope narrowing (vs original FOLLOWUP body)

The original FOLLOWUP `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh` framed scope as "9 chapters with stale card strings; 3-5 days effort" (per architect I3 in the brainstorm `design/BRAINSTORM_followups_abc_release_plan.md`).

**P0 recon refines this dramatically:** the bulk `ms10entrsq|mk1qprsqhp|md1zsxdsp` grep counts 60 hits across 9 chapters, but separating by prefix shows:

- **`ms10entrsq` (32 hits)** — CURRENT. ms1 encoding is wire-format-stable post-v0.15.0 because it encodes entropy bytes directly (entropy doesn't change across encoder versions). The abandon-test-vector's `ms10entrsqqqq...cj9sxraq34v7f` is the canonical all-zeros-entropy encoding; valid against the v0.28.4 binary.
- **`mk1qprsqhp` (13 hits)** — CURRENT. Spot-checked against v0.28.4 binary: the abandon-vector BIP-84 mk1 emits `mk1qprsqhpqqsq3cqtsleeutks2qvz...` and `mk1qprsqhpp0f30mtxzd65mvwcur9u...` — BYTE-IDENTICAL to the strings in chapters 22/23/24/31/44 manuals. The v0.15.0 wire-format break changed mk1 internals but the encoding output for this specific test vector is stable.
- **`md1zsxdsp` (15 hits)** — STALE. v0.28.4 binary emits `md1fgdxlpqpqpm6jzzqq...` (new prefix). The wallet-policy encoding changed at v0.15.0 in a way that does change output for the abandon-vector. THIS is the actual refresh scope.

**Revised effort estimate: ~half-day to 1 day** (not 3-5 days). The fix is mechanical: each of the 15 stale md1 hits is one of 3 stable md1 lines (the abandon-vector wallet-policy splits across 3 chunks at the BCH limit). One bundle run gives the canonical replacement.

## Per-chapter status

| Chapter | LOC | md1-stale hits | Status |
|---|---|---|---|
| 20-quickstart/22-first-bundle.md | 118 | 4 | Refresh: L63-65 md1 lines + L93 prose example reference |
| 20-quickstart/23-verify.md | 87 | 3 | Refresh: L24-26 `--md1` argv values |
| 20-quickstart/24-recover.md | 125 | 3 | Refresh: L86-88 `--md1` argv values |
| 30-workflows/31-singlesig-steel.md | 173 | 3 | Refresh: L87-89 `--md1` argv values |
| 30-workflows/35-recovery-paths.md | 159 | 0 | **CLEAN — no refresh needed** (only contains ms1 references which are current) |
| 40-cli-reference/41-mnemonic.md | 2709 | 0 | **CLEAN — no refresh needed** (inheritance composite already audited in v0.2.0 cycle; remaining `ms10entrsq` hits are current) |
| 40-cli-reference/42-md.md | 341 | 1 | Refresh: 1 md1 reference |
| 40-cli-reference/43-ms.md | 204 | 0 | **CLEAN — no refresh needed** (ms-cli reference; only ms1 strings, which are current) |
| 40-cli-reference/44-mk-cli.md | 405 | 1 | Refresh: 1 md1 reference (L54 `--from-md1` argv) |

**Total refresh scope: 6 chapters / 15 md1-stale hits.**

## Transcript recapture scope (unchanged from original plan)

4 transcript files in `docs/manual/transcripts/` need recapture against v0.28.4:

- `22-first-bundle.{cmd,out}` — bundle command; produces canonical ms1 + mk1 + md1 cards for the abandon-vector BIP-84 wallet.
- `23-verify.{cmd,out}` — verify-bundle with the ms1/mk1/md1 strings from 22-first-bundle.out. Currently fails with `result: mismatch` because the embedded md1 strings are pre-v0.15.0 stale.
- `24-recover.{cmd,out}` — convert flow demonstrating phrase recovery from ms1 + secret reconstruction.
- `24-recover-md1.{cmd,out}` — md decode of the bundle's md1 cards back to the descriptor.

The recapture is mechanical: re-run each `.cmd` against `target/debug/mnemonic` (or the sibling-CLI binaries for `24-recover-md1`) and capture stdout to the corresponding `.out`. 23-verify's `.cmd` needs the md1 string updated in lockstep with 22-first-bundle's new output (the verify cmd embeds the md1 cards from the bundle).

## SKIP_STEMS removal (final P3 step)

`docs/manual/tests/verify-examples.sh` carries a 4-entry SKIP_STEMS array added in manual-v0.2.0 commit `52f33f7` to exclude the 4 transcripts from the audit gate. After recapture + verify the new transcripts pass `make audit`, remove the SKIP_STEMS array + the `is_skipped` helper + the call-site filter (~10 LOC).

## Risk flags (carried from brainstorm)

- **23-verify cascade dependency:** the .cmd embeds the md1 strings from 22-first-bundle.out. Must recapture 22-first-bundle first, then update 23-verify.cmd with the new md1 lines, then recapture 23-verify. Per-cmd `mktemp -d` cwd is fine since the cmd has the md1 inline.
- **Local PATH `md` shell alias collision:** `which md` may return `mkdir -p`; use `/home/bcg/.cargo/bin/md` for 24-recover-md1 spot-check.
- **`docs/manual/tests/verify-examples.sh` SKIP_STEMS removal triggers full 14-transcript gate.** Local `make audit` must pass before tag push.

## Next-phase trigger

This recon dossier is the P0 deliverable. The next phase (P1a transcript recapture + P2 prose refresh + P3 SKIP_STEMS removal + P4 cycle close) can ship in a single session given the narrowed scope. User direction needed on whether to continue immediately or pause for review.
