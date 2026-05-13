# v0.9.0 Phase 3 — Hygiene-matrix R1 (cross-repo)

**Reviewer:** Opus 4.7 via `feature-dev:code-reviewer` agent, 2026-05-13.
**Branches (Phase 3 docs-only, both untracked working-tree files at review time):**
- mnemonic-toolkit: `v0_9_0-phase-3-hygiene-matrix` at HEAD `863f18a` (Phase 2 close).
- mnemonic-secret: `v0_9_0-phase-3-hygiene-matrix` at HEAD `123dea3` (Phase 2 close).

**Matrix files under review:**
- `mnemonic-toolkit/design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` (canonical hub).
- `mnemonic-secret/design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` (sibling, cites canonical).

## Verdict

**1 Critical / 1 Important / 2 Notable — NEEDS-WORK (foldable in one pass).**

The matrices' row/flag-status coverage is complete. The Critical
is a SPEC §6 gate-4 reconciliation: matrix §3 cites FOLLOWUPS IDs
that don't (yet) exist in either repo's `FOLLOWUPS.md`. Once the
missing entries are opened and the slug names are rationalized
against SPEC §3, the matrix set satisfies SPEC §6 gate 6 cleanly.

## Critical findings

### C-1 — Matrix §3 cites FOLLOWUPS IDs absent from `FOLLOWUPS.md` files (conf 95)

**Where:**
- toolkit matrix `§3` (lines 211-223)
- ms-secret matrix `§3` (lines 122-131)
- toolkit `design/FOLLOWUPS.md`
- ms-secret `design/FOLLOWUPS.md`
- SPEC `design/SPEC_secret_memory_hygiene_v0_9_0.md` §3

**Problem.** SPEC §6 gate 4 reads: "All 11 SPEC §3 OOS entries have
FOLLOWUPS opened." Toolkit matrix §5 self-asserts this gate ✓ but
the underlying FOLLOWUPS entries are absent in many cases.

Toolkit `FOLLOWUPS.md` currently has only these Cycle-A entries
under "Open items":

- `resolved-slot-entropy-zeroizing-field`
- `rust-secp256k1-secretkey-zeroize-upstream`
- `rust-bip39-mnemonic-zeroize-upstream`
- `rust-bitcoin-xpriv-zeroize-upstream`
- `convert-minikey-stdout-redaction`
- `secret-memory-hygiene-v0_9-cycle-a` (cycle meta)

ms-secret `FOLLOWUPS.md` carries only the cross-repo meta entry
`secret-memory-hygiene-v0_9-cycle-a`.

**Missing entries that the matrix or SPEC §3 references:**

Toolkit-side (open in `mnemonic-toolkit/design/FOLLOWUPS.md`):

- `argv-overwrite-after-parse` (SPEC §3 — `/proc/self/cmdline`)
- `clap-argv-pre-parse-residue` (SPEC §3 — `OOS-libc-osstring`)
- `allocator-pool-residue` (SPEC §3 — `OOS-allocator-residue`)
- `pub-struct-drop-semver-risk-monitor` (SPEC §3 — `OOS-pub-struct-drop`)
- `dedicated-secret-arena` (SPEC §3 — `OOS-secret-arena`)
- `sha3-shake256-zeroize-upstream` (SPEC §3 — SHAKE256 XOF state)
- `bip38-crate-internal-zeroize-upstream` (SPEC §3 — bip38 internals)
- `secret-memory-hygiene-cycle-b` (SPEC §3 — `OOS-mlock-cycle-b`)
- `md-mk-private-key-surface-watch` (SPEC §3 — `OOS-md-mk`; cross-repo companion)

ms-secret-side (open in `mnemonic-secret/design/FOLLOWUPS.md`):

- `ms-codec-payload-zeroize-public-api` (SPEC §3 — OOS-public-payload)
- `ms-codec-doc-example-zeroize-consistency` (SPEC §3 — OOS-7)
- `ms-cli-decode-emit-zeroize-intermediate` (SPEC §3 — OOS-decode-stdout)
- `rust-codex32-zeroize-upstream` (ms-secret matrix §0.5 cites this; surfaced during ms-codec envelope.rs work)
- `md-mk-private-key-surface-watch` (cross-repo companion)

**Fix (path A — preferred):** Open the listed FOLLOWUPS entries in
lockstep with Phase 3. Each entry uses the standard tracker schema
(Surfaced / Where / What / Why deferred / Status / Tier — and
Companion: lines for cross-repo).

**Fix (path B — fallback):** Weaken matrix §5 gate-4 self-assertion
to "FOLLOWUPS to be opened in Phase E rollup" and add a Phase E
pre-tag checklist line. Less clean and pushes the gate to the
last possible moment.

Path A also fixes the natural reader-experience hazard: matrix
§3 currently functions as a forward-visibility index, and an index
pointing at non-existent entries breaks the contract.

## Important findings

### I-1 — Slug-name divergence between SPEC §3 and matrix §3 (conf 90)

**Where:**
- toolkit matrix §3 row 5 + §0.5 class 4 use slug `libc-osstring-pre-clap-residue`
- SPEC §3 OOS-libc-osstring uses slug `clap-argv-pre-parse-residue`
- toolkit matrix §3 row 10 + ms-secret matrix §3 use slug `ms-codec-payload-entr-zeroize-public-api`
- SPEC §3 OOS-public-payload uses slug `ms-codec-payload-zeroize-public-api`

Cross-repo cite consistency requires one canonical slug per concept.
SPEC is authoritative; the matrices should adopt SPEC's names.
ms-secret matrix §0.5 also carries the longer slug
(`ms-codec-payload-entr-zeroize-public-api`) and needs the same
edit.

**Fix:** Rename in both matrix files. Either pick is fine technically
but adopting SPEC's slugs keeps SPEC as the canonical reference.

## Notable findings

### N-1 — Evidence-cite line-range drift (±2-6 lines) (conf 75)

Spot-checks of toolkit matrix §1 evidence cites show small editorial
mismatches that don't affect correctness — the cited code lives in
the cited file, just a few lines off:

- `synthesize.rs:404-405` — accurate.
- `derive_child.rs:108-119` — actual span 108-122.
- `derive.rs:14-66` — actual struct + impls span 12-58.
- `derive_slot.rs:18-37` — actual `derive_master_seed` is 17-34.
- `bundle.rs:344-352` — actual `into_parts()` consumption at L346.

Drift is editorial, not structural. Worth a single sweep before
Phase E tagging.

### N-2 — Matrix §0 "~27 OWNED-row wraps" vs SPEC §2 "~30 toolkit rows" (conf 70)

Matrix §0 row-1 right-most cell says "9 argv-flag closures + ~27
OWNED-row wraps + 32 SAFETY anchors". SPEC §2 prose mentions "~30
toolkit rows". Matrix §1 (when enumerated) lists 38 toolkit rows.

Reconciling the §0 summary count with §1 enumeration (or adopting
SPEC's "~30") would tighten the cross-repo coverage cell.

## Cross-repo coverage assessment

- Per-row counts agree between matrices for the ms-secret side
  (4 ms-codec + 10 ms-cli OWNED rows; 5 ms-cli flag-rows).
- ms-secret matrix §0 correctly cites the toolkit matrix as the
  cross-repo canonical authority (lines 4-9 of the ms-secret
  matrix).
- ms-secret §0.5 inherits the toolkit §0.5 classes and adds 3
  ms-secret-specific residuals (`Payload::Entr` public-API,
  `bip39::Mnemonic` interior, `codex32::Codex32String`). Coherent.
- ms-secret §3 cites toolkit §3 and adds 3 ms-secret entries —
  but inherits C-1: 3 of those 3 (`rust-codex32-zeroize-upstream`,
  `ms-codec-payload-entr-zeroize-public-api`, `secret-memory-hygiene-v0_9-cycle-a`)
  need FOLLOWUPS entries in ms-secret too (one already exists,
  two are missing).
- ms-secret §4 correctly declares non-participation per SPEC §3
  OOS-md-mk split-cycle scope; toolkit §4 enumerates the 5 mlock
  candidates per survey §4 + SPEC §3 OOS-mlock-cycle-b.

## §0.5 prose check

Both matrices have the §0.5 prose block. Toolkit §0.5 covers the
six classes (Xpriv-Copy, Mnemonic-interior, SecretKey-stack-bound,
libc-OsString pre-clap, /proc/self/cmdline + allocator-pool,
mlock-Cycle-B). ms-secret §0.5 cites toolkit and adds 3
ms-secret-specific residuals. Adequate for SPEC §6 gate 6.

## Row-status completeness

§1 + §2 coverage is complete. Every survey-§1 toolkit OWNED row
and every survey-§5 flag-row has a status cell. ms-secret §1
enumerates the 4+10 rows and §2 the 5 flag-rows. Status legend
(CLEAR / PARTIAL-3RD-PARTY / OUT-OF-SCOPE) used consistently.

## Disposition

**NEEDS-WORK — fold C-1 + I-1, then merge.**

C-1 is a real gate failure under SPEC §6 gate 4 literal text.
Foldable in one cycle:
1. Open the ~9 missing toolkit FOLLOWUPS + 4 missing ms-secret
   FOLLOWUPS entries (path A).
2. Slug-sync matrix §3 + §0.5 against SPEC §3 (I-1).
3. Optionally sweep N-1 evidence-cite line drift + N-2 count
   reconciliation in the same fold pass.

After fold + R2 verification, the matrix set satisfies SPEC §6
gate 6.

Phase E (release rollup) is the next step after R2 closes.
