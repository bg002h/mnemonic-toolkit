# R0 review — `SPEC_md1_bip_alignment_and_code_honesty.md` (round 4, DEFER-fold convergence)

**Reviewer:** Fable, adversarial, read-only. Persisted verbatim per CLAUDE.md. Ground truth: `descriptor-mnemonic` @ `origin/main ef1f3e71` (re-confirmed live), toolkit @ `master 600b7215` (crates/ unchanged since v0.84.0).
**Dispatched:** 2026-07-10 (MD SPEC, R0 round 4).

## 1. Round-3 fold verification

**I-1(a) — false "No runtime flip (verified)" claim: RESOLVED.** The blanket claim is gone. Line 106 scopes "NO runtime flip" to tier (A) comment/routing only; line 113 states tier (B) is "a REAL runtime flip, one WALLET-CHANGING, ZERO test coverage." Correctly split.

**I-1(b) — the four flipped behaviors: CORRECT, source-verified.** All cites check out against toolkit master:
- `bundle.rs:1416-1418` — `canonicity_probe` + `is_non_canonical = canonical_origin(&canonicity_probe.tree).is_none()`: confirmed.
- Behavior (2): §4.12.g guard `if !is_non_canonical && args.account != 0 → ModeViolation` at `bundle.rs:1421-1427`: confirmed.
- Behavior (3): §6.6 row-4 canonical-mode `[Phrase,Path]` rejection at `bundle.rs:1432ff`: confirmed.
- Behavior (1): `bind_descriptor_mode_paths` early-return `if !is_non_canonical { return Ok(defaulted_indices) }` at `bundle.rs:2262-2266`, H12 BIP-48 sh→1' default inference only on the non-canonical branch — so the 48'/0'/0'/1'→49' silent switch is accurate: confirmed.
- Behavior (4): `verify_bundle.rs:1408-1414` mirror probe: confirmed; fail-loud characterization matches round-3's empirical run.

**I-1(c) — DEFER internal consistency: INCOMPLETE — see Important.** Lines 105/113/114 are consistent; acceptance #6 (line 125) satisfied by DEFER. But a full-spec grep for "re-pin" finds a fifth mention: **Phase 3 (line 133) still ends "…release ritual (md-codec + md-cli lockstep) + toolkit re-pin"** — a surviving this-cycle re-pin instruction.

**I-1(d) — bip49 refusal stays pinned: RESOLVED.** Lines 106-107 KEEP pinned / "do NOT add sh(wpkh)→Bip49"; source confirms the pin mechanism. `tests/cli_gui_schema_classify_descriptor.rs` exists for the line-112 cell addition.

**M-A / M-B / M-C: FOLDED.** M-A: line 111 adds `error.rs:343-349` accurately. M-B: B-C2 (line 74) mandates the BIP carry ONLY "under separate design; current decoder rejects `MissingExplicitOrigin`", forbids naming the mechanism in the BIP; mechanism survives only in §F-A1b + the acceptance-#5 FOLLOWUP. M-C: line 5 lists rounds 1-3.

## 2. DEFER coherence sweep

Acceptance #1-#3 repo-local. GUI mirror bullet correctly untouched. Line 116's `.examples-build` note is follow-up context, not a this-cycle regen. Tier-(A) comment updates assigned to the follow-up. DG-5 correctly absent from the FOLLOWUP list (F-A8 implements it). One manual-ripple under-specification → M-D.

### IMPORTANT

- **I-1 (round-4). The DEFER fold is incomplete in two gate-bearing sections.** (a) **Phasing Phase 3 (line 133) still instructs "+ toolkit re-pin" as a this-cycle step**, contradicting the line-114 DECISION and line-105 "This cycle does NOT re-pin the toolkit." An implementer executing the phase list by the letter performs the untested wallet-changing flip this cycle. (b) **Acceptance #5 (line 124) omits `toolkit-repin-sh-wpkh-canonical-flip`** from the FOLLOWUPs-filed gate list, while line 114 mandates filing it in BOTH repos — the deferred flip loses its acceptance-checked tracking anchor. **Fold:** line 133 → "…release ritual (md-codec + md-cli lockstep); toolkit re-pin NOT in this cycle (deferred — see ripple DECISION; file the FOLLOWUPs)"; add `toolkit-repin-sh-wpkh-canonical-flip` to acceptance #5.

### MINOR

- **M-D (new).** The manual ripple (line 103) says "update `40-cli-reference`; `verify-examples` reruns live cmds" — but CI's `MD_BIN` is tag-pinned at `descriptor-mnemonic-md-cli-v0.11.2` (`.github/workflows/manual.yml:86`; local Makefile default builds the sibling checkout, `docs/manual/Makefile:45`). Any transcript demonstrating NEW behavior (A3 hard error, A2 chunked decode) reds the manual gate until manual.yml's tag bumps to the new md-cli release — a bump independent of, and compatible with, the deferred toolkit **lib** re-pin (MD_BIN is doc-verification-only). Adjacent trap: `scripts/install.sh:35`'s md-cli sibling pin is the FROZEN v0.11.2 baseline policed by sibling-pin-check (the v0.75.0 post-tag-revert precedent) — bump **manual.yml's** tag, NEVER install.sh's. (Help-text-table-only updates are un-gated by lint.sh and can land without the bump; only new-behavior transcripts are blocked.)

## 3. Recovery-safety

Unchanged from rounds 1-3: F-A1 additive, F-A2 dispatch-bit additive, F-A8 rejects only malformed non-zero pads, F-A3 exit≠0, F-A4 stderr-only, F-A5/A9 cosmetic. The DEFER is the maximally-safe posture for the tier-(B) flip; the only defect is that the spec's own phasing text hasn't fully caught up.

**VERDICT: OPEN (0C / 1I / 1M)** — fold I-1 (both edits) + M-D, then re-dispatch for convergence.
