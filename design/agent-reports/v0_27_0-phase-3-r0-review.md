# v0.27.0 Phase 3 R0 architect review — BSMS Round-2 emitter + first-address verify

**Reviewer:** opus (feature-dev:code-reviewer). **Date:** 2026-05-18.
**Verdict:** YELLOW (2 Critical + 3 Important + 3 Minor + 7 Confirmations).
**Folded by:** Phase 3 implementer in-cycle (this commit).

---

## Critical issues

### C1 — FOLLOWUPS.md Status flips MISSING for both closure entries

**File:** `design/FOLLOWUPS.md` lines 2093, 2162.

Per-phase commit-shape brief is explicit: Phase 3 commit closes BOTH
`wallet-export-bsms-emitter` and `bsms-first-address-verify` FOLLOWUPS via
`design/FOLLOWUPS.md` Status flips IN THE SAME COMMIT. Matches the recurring
failure mode `[[feedback-per-phase-agents-forget-followup-status-flip]]`.

**FOLDED:** Status flips staged for the Phase 3 commit. Closure narratives
written for both entries.

### C2 — Import-side WARNING template deviates from FOLLOWUP body + SPEC §2.4 row 3 wording

**File:** `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:224-230`.

FOLLOWUP body specifies: `warning: import-wallet: bsms: first-address mismatch at path <P>: computed <C>, blob declares <D>`.

Implementation omitted `at path <P>` segment.

**FOLDED:** Template updated to include the path segment sourced from
`audit.derivation_path`. SPEC §2.4 row 3 un-struck.

---

## Important issues

### I1 — Path-restrictions divergent-multipath false-positive in descriptor-passthrough mode

**File:** `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:132`.

`is_multipath()` returns true for ANY multipath cosigner; literal
`contains("<0;1>/*")` matches as long as one key has the canonical shape.
Mixed-shape descriptors (via user-supplied `--descriptor`) emit incorrect
path-restrictions.

**FOLDED:** Replaced string-contains heuristic with structural per-key
inspection via `parsed.for_each_key` walking each `DescriptorPublicKey`'s
multipath alternatives. All-`<0;1>/*` → `/0/*,/1/*`; any divergence →
`No path restrictions`.

### I2 — `bsms_6_line_happy_path` now emits unasserted first-address-mismatch WARNING

**File:** `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:132-159`.

Synthetic `bc1qexample...` placeholder is unequal to the toolkit's computed
first-address, so v0.27.0 emits the new WARNING for this test fixture.

**FOLDED:** Computed first-address from the actual descriptor at test-build
time via `mnemonic_toolkit::derive_address::derive_first_address`. The test
now uses the real first-address so the WARNING does NOT fire for happy-path
ingest, and an explicit negative assertion (`!contains("first-address
mismatch")`) gates against regression.

### I3 — Helper name mismatch: recon doc says `derive_address_at_path`, impl is `derive_first_address`

**Files:** `design/agent-reports/v0_27_0-phase-3-r0-recon.md:24` vs
`crates/mnemonic-toolkit/src/derive_address.rs:26`.

**FOLDED:** Recon doc updated to match impl name (`derive_first_address`).
Impl name is more accurate (hardcodes `/0/0`, not generic path).

---

## Minor issues

### M1 — Test cell 8 misnamed (`bsms_4line_then_...` but exercises 2-line)

**File:** `crates/mnemonic-toolkit/tests/cli_export_wallet_bsms.rs:366`.

**FOLDED:** Renamed to `bsms_2line_then_import_byte_exact_idempotent` to
match what the test exercises (2-line emit through v0.26.0's 2-line lenient
parser).

### M2 — `path_restrictions_line` accepted `parsed: &MsDescriptor` but only used `is_multipath()`

**File:** `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:126-147`.

**FOLDED:** Closed via I1's structural-inspection fix; the `parsed` param
now drives the per-key walk.

### M3 — Closure narrative for `bsms-first-address-verify` should note taproot-skip

**File:** `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:220-221`.

**FOLDED:** Closure narrative in `design/FOLLOWUPS.md:2093` mentions the
taproot-skip explicitly so future-cycle readers don't assume all 6-line
blobs get verified.

---

## Confirmations (verified correct; no fold needed)

1. **Taproot rejection wins before parse/derive in export-wallet.** Cells 6
   + script_type_from_descriptor mapping handle the descriptor-passthrough
   path correctly.
2. **`EmitInputs.bsms_form` field placement correct.** Last field; default
   `FourLine`; other emitters silently ignore.
3. **Cell 5 cross-check is not tautological.** Independent miniscript call;
   structural regression guard, not a logic check.
4. **2-line round-trip cell (cell 8) closes the correct gap.** v0.27.0
   ingest does NOT add 4-line parser; `bsms-bip129-full-cutover` v0.28+.
5. **gui-schema format-count bump (8 → 9) is correct.** Additive enum
   value; SPEC §7 schema-version stays at 5.
6. **Rename + flip of `bsms_first_address_mismatch_warning` is clean.**
   `stdout.contains("bsms_audit=some")` preservation invariant pinned.
7. **No verify-bundle round-trip exposure.** Phase 3 helper consumed only
   by export-wallet line-4 emit + import-wallet WARNING; bundle synth path
   untouched.
