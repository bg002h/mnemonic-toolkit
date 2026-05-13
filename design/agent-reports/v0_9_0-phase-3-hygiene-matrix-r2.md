# v0.9.0 Phase 3 — Hygiene-matrix R2 (cross-repo fold-verification)

**Reviewer:** Sonnet 4.6 via `feature-dev:code-reviewer` agent, 2026-05-13.
**Branches:**
- mnemonic-toolkit: `v0_9_0-phase-3-hygiene-matrix`
- mnemonic-secret: `v0_9_0-phase-3-hygiene-matrix`

## Verdict

**0C / 0I — Phase 3 READY TO CLOSE.**

R2 trivial fold-verification per project convention. R1 was 1C/1I/2N
(Opus); all findings folded in a single pass across both repos. R2
confirms each fold is present and the matrix set satisfies SPEC §6
gate 4.

## Per-fold confirmation

### C-1 — toolkit FOLLOWUPS entries

All 9 expected headers confirmed present in
`mnemonic-toolkit/design/FOLLOWUPS.md` (open-items section, between
`convert-minikey-stdout-redaction` and `secret-memory-hygiene-v0_9-cycle-a`):

- ✓ `argv-overwrite-after-parse` (line 90)
- ✓ `clap-argv-pre-parse-residue` (line 99)
- ✓ `allocator-pool-residue` (line 108)
- ✓ `pub-struct-drop-semver-risk-monitor` (line 117)
- ✓ `dedicated-secret-arena` (line 126)
- ✓ `sha3-shake256-zeroize-upstream` (line 135)
- ✓ `bip38-crate-internal-zeroize-upstream` (line 143)
- ✓ `secret-memory-hygiene-cycle-b` (line 151)
- ✓ `md-mk-private-key-surface-watch` (line 165)

All 5 expected headers confirmed present in
`mnemonic-secret/design/FOLLOWUPS.md` (open-items section, after
`secret-memory-hygiene-v0_9-cycle-a` and before
`bip-vector-adoption-v0_8`):

- ✓ `ms-codec-payload-zeroize-public-api` (line 42)
- ✓ `ms-codec-doc-example-zeroize-consistency` (line 51)
- ✓ `ms-cli-decode-emit-zeroize-intermediate` (line 60)
- ✓ `rust-codex32-zeroize-upstream` (line 69)
- ✓ `md-mk-private-key-surface-watch` (line 77)

### I-1 — slug renames in both matrix files

- ✓ Old slug `libc-osstring-pre-clap-residue`: 0 hits in toolkit
  matrix, 0 hits in ms-secret matrix (fully removed).
- ✓ New slug `clap-argv-pre-parse-residue`: 2 hits in toolkit matrix
  (lines 66, 225). ms-secret matrix has 0 hits — expected and
  correct: R1 scoped this rename to the toolkit matrix only; ms-secret
  matrix never carried `libc-osstring-pre-clap-residue` and delegates
  §3 to the toolkit canonical. The R2 checklist's "≥1 hit each"
  over-specifies for ms-secret; the fold itself is complete and
  the absence in ms-secret is architecturally correct.
- ✓ Old slug `ms-codec-payload-entr-zeroize-public-api`: 0 hits in
  toolkit matrix, 0 hits in ms-secret matrix (fully removed).
- ✓ New slug `ms-codec-payload-zeroize-public-api`: 1 hit in toolkit
  matrix (line 223), 2 hits in ms-secret matrix (lines 41, 126).

### N-1 — evidence-cite line-range cleanup (toolkit matrix §1)

- ✓ `bundle.rs:346` (line 107; was `:344-352`)
- ✓ `bundle.rs:447` (line 108; was `:444-455`)
- ✓ `derive.rs:20-58` (line 148; was `:14-66`)
- ✓ `derive_slot.rs:32-34` (line 152; was `:18-37`)
- ✓ `derive_child.rs:108-122` with L135 consumer mention (line 127;
  was `:108-119`, no L135 mention)

### N-2 — §0 row-1 Delta-cells reconciliation

✓ Toolkit matrix §0 row-1 right-most cell now reads "9 new argv-flag
closures (Phase 1) + ~30 toolkit OWNED-row wraps per SPEC §2
(enumerated in §1: 38 row-cells) + 32 SAFETY anchors" (line 22;
was "~27 OWNED-row wraps"). Reconciles with SPEC §2 "~30" + §1
enumeration "38 row-cells".

### Gate-4 reconciliation

✓ Toolkit matrix §5 gate-4 (line 281) now reads "All 14 SPEC §3 OOS
entries have FOLLOWUPS opened" (was "11").

### §3 expansion

Toolkit matrix §3 now has two tables:
- ✓ "SPEC §3 OOS entries (14):" — 14-row table (lines 218-233)
- ✓ "Cycle-surfaced entries (4) — not in SPEC §3 but opened during
  Phase 1-2:" — 4-row table (lines 237-242)

Row counts: 14 + 4 = 18 total entries. Both table headers and row
counts verified.

### ms-secret §3 update

✓ ms-secret matrix §3 (line 123) cites "the full 14-SPEC-OOS +
4-cycle-surfaced list" in the toolkit matrix and enumerates 5
ms-secret-side FOLLOWUP slugs (entries confirmed present in
ms-secret FOLLOWUPS.md): `ms-codec-payload-zeroize-public-api`,
`ms-codec-doc-example-zeroize-consistency`,
`ms-cli-decode-emit-zeroize-intermediate`,
`rust-codex32-zeroize-upstream`, `md-mk-private-key-surface-watch`.
Also cites `secret-memory-hygiene-v0_9-cycle-a` (cycle meta;
ms-secret FOLLOWUPS.md line 33).

## Disposition

**MERGE.** All folds confirmed present and correctly applied. The
matrix set satisfies SPEC §6 gate 4 (14 OOS entries + 4
cycle-surfaced entries, all FOLLOWUPS opened in the correct repos).
Phase 3 of v0.9.0 Cycle A (cross-repo secret-memory-hygiene audit
matrix) is COMPLETE. Phase E (release rollup) is the next step
per plan.
