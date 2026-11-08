# fold-vf-s1: architect vf-s1 folded into the vendor-freshness spec

Fold commit 77777ffc on spec/vendor-freshness-shared (pushed). No STOP: the ruling decided every point folded.

## What changed
- `design/SPEC_vendor_freshness_shared.md`: Status line no longer says "one open point"; section 10 rewritten as the
  decision (option A text dropped); section 11 uses `ci/repro/shared-vendor.sha256` (toolkit, key, secret, dm) and
  states mr's two appended lines and no new mr step; section 12 PR-T2 carries the section 10 diff, PR-E and PR-X
  dropped; section 13 lists C11-C15 as covered. `grep -n "S-1\|OPEN\|option A"` now hits only the Status line's
  historical "(STOP S-1, ruled by architect vf-s1)".
- The architect's drift diff is embedded verbatim in section 10 as block `vf:drift-diff`; the gate applies that
  block, not a copy. Blocks added: pin-file, pin-cmd, pin-check-green, pin-plant-edit, pin-plant-junk, mr-pin-check,
  mr-pin-plant, drift-diff, drift-sha256, drift-test-green, drift-red, drift-mutant-M1, drift-mutant-M2.
- `design/spec-vendor-freshness/gate.sh`: restore() also removes the pin file; new C11 (pin check, 4 repos + mr),
  C12 (plants: edited script, junk line; mr edited script), C13 (diff applies at the spec SHA, hashes 6dc3f8b7 and
  6e88cb5e, test prints the 9-row block), C14 (unmodified test REDs: D1 D2, 5 passed 2 failed), C15 (planted
  faults M1: D1 D8, 7 passed 2 failed; M2: D1 D2 D8 D9, 5 passed 4 failed). All match the architect's figures.

## Gate
`bash design/spec-vendor-freshness/gate.sh /scratch/cache/spec-vf-wt /scratch/cache/spec-vf-logs/run1` ended
`gate: PASS` (exit 0), 72 PASS lines, 0 FAIL; log `/scratch/cache/spec-vf-logs/run1.out`. All five repo
worktrees clean afterwards. `bash ci/code-docs-check.sh` in the spec worktree: ok, rc 0.

## Judgement calls (not product points)
- Status wording: "Status: DRAFT. Every point is decided here, including "Shared-file pinning" below (STOP S-1,
  ruled by architect vf-s1)."
- Embedded the diff in the spec (rather than a side file) so the exact text is in the spec and gated.
- Section 13's pointer to the gate run now names this report.
- The mr plant blocks include mr's two existing OK lines because the check runs on the whole file.

Not covered (stated in section 13): the `.github/` edits other than the pin step's run command, catalog and
FOLLOWUPS edits, a real network outage, mr runner rsync, CI parity.

Counts: 0 Critical, 0 Important, 0 Minor, 0 Nit; 0 STOP.
