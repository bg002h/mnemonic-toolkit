# SPEC R0 round 3 — bundle-md1-template-multisig (#28 phase 2) (opus architect, verbatim)

> Reviewer: opus architect (read+bash; toolkit `e97e8470`; `Cargo.toml:35`=`mk-codec "0.4.0"`). Focused confirmation of the round-2 one-line fold (I-NEW §9:154) + pin-label minor. **Verdict: GREEN — 0 Critical, 0 Important.** SPEC R0 converged (RED r1/r2 → GREEN r3).

## I-NEW (§9:154) — CLOSED
The "Origin handling DECIDED" bullet now reads "carry source per-`@N` origins for DECODE-VALIDITY ONLY; at completion `path_decl` BUILT FRESH from supplied-key origins; the rebuild is the write-SITE only, NOT the origin source (NOT `compute_default_origin_path`); plan-time = Guard-C relaxation + carried-origin-never-loaded test." The repudiated "supplied-key origins win via the rebuild"/"confirm the rebuild reuse" framing is gone; matches §4.2a:73/78/80 + §7-floor-6:130 + §3.2:50-54; complements (no longer contradicts) §9:155. Invariant verb upgraded to the stronger "never-loaded."

## Pin minor — CLOSED
Header `:5` distinguishes linked lib `mk-codec 0.4.0` (`Cargo.toml:35`, holds the `KeyCard.{origin_fingerprint,origin_path,xpub}` the I-B intake reads) from the `mk-cli v0.10.0` CLI binary. I-B feasibility correctly attributed.

## Drift — none
§9:154 reword consistent with §4.2a/§3.2/§7-floor-6 (one framing: carry-for-decode-validity + build-fresh + rebuild-is-write-SITE). C1 invariant intact + reinforced (never-loaded). Edit purely subtractive of dead framing + verb precision; pin edit orthogonal.

## Verdict
**GREEN — 0 Critical, 0 Important.** SPEC clears the R0 gate → proceed to the plan-doc (which re-enters the mandatory plan-time R0 loop).
