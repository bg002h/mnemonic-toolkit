# BCH residue conformance hardening — plan R0 round 1 (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; full read+bash, recomputed NUMS derivations + polymod length-invariance in python against all 4 on-disk repos + published crate versions).
> Plan: `design/PLAN_bch_residue_conformance_hardening_2026-06-18.md`. **Verdict: GREEN (0C/0I).** Two non-blocking Minors folded into the plan before implementation.

---

All confirmed. The mk-codec comment at lines 181-185 is indeed misleading: it claims `0x23181b3` is the "(BIP 93)" initial residue and that `ms32_polymod`/`ms32_long_polymod` "start with this residue" — but BIP-93's actual `ms32_polymod` starts from residue `1`, not `0x23181b3`. The plan correctly identifies this (D2). The mk comment at line 166 ("Source: BIP 93 reference implementation, `ms32_polymod`") for `GEN_REGULAR` is correct (those generator values ARE from BIP-93). md's `POLYMOD_INIT` is bare-private, so D1's doc comment is purely additive (no visibility change).

## R0 Review — `PLAN_bch_residue_conformance_hardening_2026-06-18.md`

I read the plan in full and independently verified every load-bearing claim against the on-disk source, the published crates.io versions the toolkit actually builds against, and the historical bug record. Empirical recomputations (SHA-256 NUMS derivations, the polymod length-invariance argument, and the cross-init mismatch) were run in Python mirroring the exact Rust arithmetic.

### Verification results (all independently confirmed)

**Open Q1 — md NUMS bit-extraction is correct.** Recomputed: `top65(SHA256("shibbolethnums")) = 0x815c07747a3392e7` == `MD_REGULAR_CONST` (md-codec `bch.rs:17`). It is the exact mirror of mk's already-passing technique (`mnemonic-key/crates/mk-codec/src/consts.rs:71-83`): same `u128::from_be_bytes(digest[0..16])` then `>> 63`. mk's regular (`>>63`→`0x1062435f91072fa5c`) and long (`>>53`→`0x41890d7e441cbe97273`) both reproduce too. The prescribed T1 (plan:38-39) will pass; the comment's derivation is true, not aspirational. md-codec genuinely has **no** NUMS re-derivation test today — `tests/bch_visibility_pin.rs:43` only touches `MD_REGULAR_CONST` for compilation, never re-derives it from the domain string. The "MISSING" claim (plan:25) is accurate.

**Open Q2 — length-invariance math is correct (with one wording caveat, below).** Empirically, for inits `{0x1, 0x23181b3, 0xdeadbeef, 0x0, 0xffff...}`, a self-contained create+verify round-trips at every length and lands on exactly one residue == TARGET. So the plan's central claim — "any fixed init is self-consistent / length-invariant for a self-contained code, the init term cancels" (plan:17, plan:42) — is mathematically sound.

**Open Q3 — T2 is sound and reachable.** Cross-reject (b) is robust: a codeword valid under init=1 verifies False under init=0x23181b3 at every length (3/16/32 all False), and the four targets are pairwise distinct. The toolkit builds against published ms-codec 0.4.4 / md-codec 0.36.0 / mk-codec 0.4.0 (Cargo.lock:677/715/767; the `[patch.crates-io]` at toolkit `Cargo.toml:28-29` patches only miniscript). All four import paths resolve in those published versions: `ms_codec::bch::MS_REGULAR_CONST` (`pub mod bch`, `pub const`), `md_codec::bch::MD_REGULAR_CONST` (`pub`), `mk_codec::{MK_REGULAR_CONST, MK_LONG_CONST}` (root re-export, mk `lib.rs:44-48`), `mk_codec::string_layer::bch::{hrp_expand, polymod_run}` (already consumed by `repair.rs:28-31`).

**Open Q4 — scope is correct.** md's round-trip suites (`proptest_roundtrip.rs`, `chunking.rs` from single-chunk to near-cap multi-chunk) drive `encode_md1_string`/`decode_md1_string`, which internally call `bch_create_checksum_regular`/`bch_verify_regular` across varying payload lengths — so the "multi-length is a non-gap for md/mk" claim (plan:55) holds; no specific missing length. Nothing in-scope duplicates ms-codec's already-complete coverage.

**Open Q5 — NO-BUMP is correct.** No wire-format/API/CLI touch. md `POLYMOD_INIT` is bare-private (`bch.rs:19`) so D1 is purely additive doc — no visibility change. No `mlock.rs` exists in md/mk (g6-sync claim vacuously safe). All four working-copy HEADs match the plan's cited SHAs (md `4ec2110`, mk `c79aa42`, ms `6b28918`, toolkit `9e64710`).

### Findings

**Critical: none.**

**Important: none.**

**Minor 1 — D1's one-line retelling of the ms1 bug is imprecise; tighten the prescribed comment so the committed text doesn't misattribute the root cause.** Plan:17 and the D1 prescription (plan:42) compress the history to "`0x23181b3` there produced a length-variant checksum." Per the authoritative bug record (`mnemonic-secret/design/BUG_decode_with_correction_length_divergence.md:34-41`), the length-variance did **not** come from the init value alone. ms *encodes* via the external rust-codex32 engine (init=1, target SECRETSHARE32) but the old hand-rolled *verify* used init=`0x23181b3` **and** a const (`0x962958058f2c192a`) empirically lifted from a single 12-word vector; the two engines disagreed, so verify residues were length-variant relative to that single-calibration const. My test confirms `0x23181b3` in a *self-contained* create+verify is itself perfectly length-invariant — it lands on one residue at all lengths. So the precise framing for the D1/D2 comment is: *"ms1's verify must match the **external** codex32 engine (init=1); the reverted v0.2.1 path used a non-codex32 init **paired with** an empirically-miscalibrated target, so its verify residues diverged from codex32's across lengths."* This is a Minor only because R0 catching it now keeps a subtly-wrong "why" out of a committed doc comment — exactly the over-claim the plan's own Q2 asks to soften. Recommended fix: in the D1/D2 prescriptions, replace "`0x23181b3` produced a length-variant checksum" with the init-plus-miscalibrated-target-vs-external-engine framing above, and cross-ref `BUG_...md:34-41` rather than just `:48-52`.

**Minor 2 — mk D2's cite line is slightly off.** Plan:27/45 cite the misleading comment at `string_layer/bch.rs:~182`/`~178-185`. The actual misleading text is the doc block at **lines 181-185** ("Initial residue value ... (BIP 93). Both `ms32_polymod` and `ms32_long_polymod` start with this residue ..."). The `~178-185` range is close enough to land the implementer, but note that line 166 in the same file ("Source: BIP 93 ... `ms32_polymod`") is for `GEN_REGULAR` and is **correct** (those generators are BIP-93's) — D2 must not touch it. Worth a one-word note so the fix targets only the `POLYMOD_INIT` block.

### Verdict: GREEN (0 Critical, 0 Important)

The plan is correct, in-scope, and sound. The NUMS derivation, length-invariance argument, T2 reachability, out-of-scope justifications, and NO-BUMP all verify. The two Minors are non-blocking polish for the committed doc-comment wording; folding them is advisable but not gating. Per the hard gate, this plan may proceed to implementation.

### Critical Files for Implementation
- /scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/bch.rs (T1 drift-guard + D1 init doc; on-disk line 19 bare)
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/bch.rs (D2 comment fix at lines 181-185, not 166)
- /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs (T2 home / import precedent at lines 28-46)
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/consts.rs (the mirror technique T1 must match, lines 71-83)
- /scratch/code/shibboleth/mnemonic-secret/design/BUG_decode_with_correction_length_divergence.md (the authoritative bug record the D1/D2 comments must paraphrase accurately)
