# Whole-branch final review (CLAUDE.md pre-tag gate) — verbatim

> Persisted verbatim per CLAUDE.md. The final whole-branch review before the release ritual + tag.
> Dispatched via Agent (feature-dev:code-reviewer, Opus 4.8). Diff: `master..HEAD` (crates/ + docs/).
> **Verdict: SHIP-READY — 0 Critical / 0 Important / 3 Minor (all doc-only).** The load-bearing
> funds-safety question answered NO (no reachable silent-wrong-emit path). 3 Minor doc fixes folded
> before the release ritual.

---

## Load-bearing Question: Is there any reachable silent-wrong-emit path for non-NUMS taproot?

**No.** The three protection layers are correctly composed and non-overlapping:

1. **Non-NUMS general single-leaf/depth-1 (route-around arm):** `faithful_multisig_descriptor` → `md_codec::to_miniscript_descriptor` `is_nums:false → lookup_key` reads the actual trunk key off the wire — no Template "leaf = all-others" shortcut reachable. `ReconstructTranslator.pk()` promotes the `XPub` cosigner key to multipath with network correction. Display-fidelity guard active (`is_taproot || template_opt.is_none()`, line ~1348). For `--format`: `None`-branch explicit bip388 guard covers `P2tr|P2trMulti`; green's `P2tr` guard; bsms's own `P2tr|P2trMulti` refusal; template-requiring formats refuse via absent `template`. No silent-wrong-emit.

2. **Non-NUMS distinct-trunk multisig (Template arm):** `refuse_at_in_both` fires BEFORE `Template(t, Cosigner(idx))`. For a passing (distinct-trunk) card, `key_segs \ {idx}` is correct (trunk provably absent from leaf indices). `build_tr_multi_a_descriptor` + `format_bip388_wallet_policy` Cosigner arm (`bip388.rs:115-127`) emit `tr(@idx/**,multi_a(k,…))` faithfully — the Template arm never reaches the `None`-branch bip388 guard. No silent-wrong-emit.

3. **@-in-both guard:** `indices.iter().any(|&idx| idx == *i)` — all `u8`, no cast. Fires for every `MultiA`/`SortedMultiA` arm uniformly. n≥3 RED-proof (N4) is the genuine dangerous case; n=2 (N4b) + SortedMultiA (N4c) secondary pins. Display-fidelity guard provably can't catch the Template wrong-leaf; the structural classify-time precondition is the only net. NUMS trunks + general-arm leaves correctly excluded.

**The funds-safety property is fully closed for all non-NUMS taproot shapes reachable by `restore --md1`.**

## Strengths
- **Architecture correctness:** split routing exhaustive/non-mis-routing; `TaprootInternalKey` threads continuously classify → call site (1279-1281) → `build_multisig_import_payload` (1598) → `EmitInputs.taproot_internal_key` (930). All `u8`, no casts, no inference.
- **Guard completeness:** `refuse_at_in_both` in BOTH `MultiA`/`SortedMultiA` arms, the only Template entry for those tags; `Body::MultiKeys` decoder-guaranteed. No @-in-both slips through.
- **NUMS regression safety:** `is_nums:true → Nums` unchanged; pre-existing NUMS goldens pinned + unperturbed.
- **Test quality:** goldens captured-once from binary (trunk = depth-0 `xpub661My…` w/ K2's `[28645006/87'/0'/0']`, not NUMS hex); N4 genuinely n≥3 (exit-0 RED confirmed); N4b/N4c independent pins; N5-N9 full format matrix; `build_at_in_both_descriptor` parameterized w/ tag, populates `tlv.pubkeys`, encodes cleanly.
- **Manual prose accurate** (3 sites), manual lint passed (6 stages). **FOLLOWUPS hygiene** correct.

## Issues

### Critical — None.
### Important — None.

### Minor (doc/comment only — no functionality impact)
**Minor 1 (confidence 92): Stale clap `--md1` arg doc-string** `restore.rs:67` — "a non-NUMS (cosigner-internal) taproot md1 is refused" is now WRONG and appears in `mnemonic restore --help`. Fix: replace with accurate text (NUMS or non-NUMS distinct-trunk multisig + general single-leaf/depth-1 supported; @-in-both + depth-≥2 refused).

**Minor 2 (confidence 87): Stale inline comment block** `restore.rs:1262-1265` — line 1263 says `tr(NUMS,…)` (omits real-key); line 1265 "non-NUMS → loud structural refusals" now only true for @-in-both + depth-≥2. Fix: update both.

**Minor 3 (confidence 81): Test module doc artifact** `cli_restore_taproot.rs:15-16` — "the guard lands in the next commit" is a dev-process artifact (all commits landed). Fix: drop the phrase.

## Pre-release Checklist
- Version bump / READMEs / install.sh / CHANGELOG still at 0.55.2 — EXPECTED (Task 5 release ritual not started).
- No GUI `schema_mirror` impact (zero clap flag add/remove/rename). PATCH SemVer correct.
- Manual lint passed; FOLLOWUPS filed.

## Assessment: SHIP-READY (pending the 3 minor doc fixes + the release ritual)
Three Minor issues all doc-only, none blocking. Funds-safety fully closed; @-in-both guard necessary/sufficient/RED-proven; NUMS regression byte-identical; format matrix correct + tested. Fix the 3 doc minors in one cleanup commit (the `--md1` clap string most visible), then proceed to the release ritual.
