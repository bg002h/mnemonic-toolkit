# POST-IMPL WHOLE-DIFF REVIEW — Cycle F (ms1-repair-demote-to-candidate) — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — release-ready.
**Reviewer:** FRESH independent Fable, cold whole-diff. Toolkit `ecce14a7..29a4e424` + ms-cli `c2fd4eb..0212b2e`. Both suites re-run; funds attacks driven against real binaries. Per user directive "fable for review".
**Dispatched:** 2026-07-09 (Cycle F, mandatory post-impl whole-diff). Persisted verbatim per CLAUDE.md.

## Independent counts
toolkit `cargo test -p mnemonic-toolkit` → **3664 passed / 0 failed / 18 ignored** (205 suites), clippy clean. ms-cli `cargo test -p ms-cli` → **225 passed / 0 failed / 5 ignored**, clippy clean. Manual gates (cycleF binaries): `verify-examples` OK 62/62; `lint` OK (markdownlint/cspell/lychee 261 OK 0 err/flag-coverage/glossary/index).

## Funds attacks (all binary-run) — every SAFE
1. **§5.5 wrong-bundle single-sig** (seed E + ms1→corrects-to-A + clean mk1/md1 A) → exit 4, `ms1_entropy_match:fail` ("not a card for this seed"), full 9-row table, `result:mismatch`, never "confirmed"/5/`?`-abort. Multisig analogue suite-pinned.
2. MATCH control (seed A + corroded ms1 A) → exit 0, "confirmed against expected seed"; corrected value used only for the byte-compare then dropped.
3. `mnemonic repair --ms1 <subst>` → exit 4 + corrected stdout + UNVERIFIED/BIP-93 advisory; clean→0; 6-error→2.
4. `--json` → exit 4 `"verdict":"candidate"` after `kind`; clean→`"blessed"`.
5. subst + `--max-indel 2` (valid-length) → exit 4 (indel flag can't launder a substitution to 5).
6. unique indel `--max-indel 1` (toolkit only) → exit 5 (carve-out intact, SPEC §3).
7. inspect/convert/xpub-search corroded ms1 (`MNEMONIC_FORCE_TTY=1`) → exit 1 (original decode err), empty stdout, I2 advisory, corrected nowhere.
8. `ms repair <subst>` → exit 4 + advisory; no `--max-indel` → exit 5 unreachable from ms-cli.
9. `--slot @0.ms1=<corrupt>` (expected-side) → exit 1 typed error (fail-closed, pre-existing).
10. `--no-auto-repair` verify-bundle → legacy fail rows, no compare, exit 4.
**No path presents an ms1 substitution-correction as recovered.** The 4 remaining `try_repair_and_short_circuit` sites (verify_bundle.rs:2213/2430/2451/3118) are all Mk1/Md1; the 2 ms1 sites fully replaced by `ms1_ground_truth_compare` which fails closed in every degenerate branch (empty corrected→`""`≠expected→loud fail; repair Err→legacy rows; empty ground truth debug_assert-guarded + still mismatch-fails in release).

## Cross-repo parity — HOLDS
`ms repair` vs `mnemonic repair --ms1`: stdout BYTE-IDENTICAL (text + `--json`), exit 0/4/2 identical, advisory body byte-identical (`repair: ` prefix). D27 superset is the actual wire behavior: both emit `schema_version<kind<verdict<corrected_chunks<repairs`; md/mk NO-BUMP lack verdict, docs correctly narrowed. One cosmetic asymmetry (stderr line ORDER differs toolkit vs ms-cli — D9/D27 pin stdout+presence only; note-level, no action).

## Secret-hygiene — CLEAN
Mismatch row: fixed detail (only a cosigner index interpolated in multisig), `expected`/`actual`/`diff_byte_offset` absent — confirmed on the JSON wire live; grep-scans found no clean/corrected/corrupted ms1 in any attack output. Corrected string `Zeroizing` at the toolkit compare site + throughout ms-cli. Advisory/verdict fixed text. Engine-wide un-zeroized `corrected_chunks` drop is pre-existing + FOLLOWUP'd. ms-codec self-redacts `{:?}` ("input withheld").

## Docs (P2) — accurate; Regressions — NONE
All 4 chapters match live behavior (per-surface exit table, "verified now/verifiable-later" model, mk-cli asymmetry, D20 ms1-unreachable, xpub-search exit-1, indel carve-out, glossary, 4 transcripts). Residual stale sentence only in the GUI-manual book (`docs/manual-gui/src/50-md/5A-repair.md:67`) — out of §6 scope, FOLLOWUP'd. All flipped cells correct intended-behavior (each strengthens, none masks). mk1/md1 unchanged (partial-set falls through WITHOUT the ms1 advisory, unit-pinned; 4 short-circuit sites intact; md1 exit-5 via cell_25/26). Indel keep-5/ambiguous-4 intact. Codecs NO-BUMP (ms-codec 0-line diff). No clap surface → no schema_mirror/GUI. mlock.rs untouched both repos (g6).

## Release-readiness — CLEARED
Pre-release state exact: ms-cli 0.13.2/ms-codec 0.7.0(stays), toolkit 0.80.0, 4 pin sites @ ms-cli-v0.13.2, self-pin @ v0.80.0, no [0.81.0] yet, FOLLOWUPS ms1-leg OPEN. Sequence: **ms-cli 0.14.0 (bump+tag+cargo publish) → toolkit v0.81.0** (4 pin refs + self-pin + Cargo.toml/locks + READMEs + CHANGELOG + FOLLOWUPS flip). REMINDERS (gotchas, not findings): regen `.examples-build` in lockstep (pins mnemonic 0.80.0, FATALs on mismatch; only version strings move — no repair content change); md/mk sibling pins are FROZEN — do NOT touch.

**GREEN. The sequenced release can proceed.**
