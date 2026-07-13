# CONVERGENCE R0 (round 1) — SPEC_verify_bundle_nonchunked_canonicalization.md

**Reviewer:** Fable architect (`model:"fable"`), dispatched 2026-07-12.
**Source SHA:** `de140a08` (HEAD == origin/master, reviewer confirmed via `git rev-parse`).
**Usage:** 38 tool-uses, ~722s, 114467 subagent tokens.
**Main-loop note:** I-1's mechanism (expected template derived from the supplied card's own tree via `cli_template_from_tree`) independently confirmed against `verify_bundle.rs:602`/`:687-695` + `synthesize.rs:369-378` before folding.

## Verdict: RED — 0 Critical / 1 Important (3 Minor)

The 6 design-R0 Minor folds all landed faithfully and are accurate to source. Facet 1 and Facet 2's code contracts are sound (type-check, correct `d`, correct verdict routing, no new `--json` state). The one Important is a NEW defect the written artifact introduces: test §6.3 #7's probative claim is false against the actual compare-target derivation, leaving INV-4 anchored by test #8 alone.

## Important

### I-1. Test §6.3 #7 cannot prove what it claims; truth-table row 3's note mischaracterizes the same mechanism
`verify_bundle.rs:602` — `verify_singlesig_template` derives the template FROM the supplied card's own tree (`cli_template_from_tree(&d.tree)`); `:687-695` — the EXPECTED bundle is synthesized from that same template. `synthesize.rs:369-377` — `cli_template_from_tree` matches only `(tag, body-variant)`. `:634-636` — the template md1 is account/origin-agnostic, byte-identical, and equally seed-/network-agnostic (keyless, origin-elided).

A *genuine* template card with a "wrong seed" or "wrong script-type" always id-matches its own re-derived expected — `md1_template_match` PASSES. The invocation's exit-4 comes solely from `mk1_template_stub_bind` (`:697-700`). So test #7 as written: (a) if it asserts only `mismatch`/exit 4, it passes even under a broken Facet 2 that hard-codes `md1_match=true` — zero INV-4 coverage; (b) if it asserts `md1_template_match.passed==false` (natural TDD reading), it is unrealizable with the stated fixture — stays RED forever. Truth-table row 3 propagates the same mischaracterization.

**Fix:** re-specify #7's fixture as a descriptor-content variant that still classifies as a single-sig template and assert `md1_template_match.passed==false`. Two realizable constructions verified: (a) keyless wpkh card with a non-standard use-site path — `cli_template_from_tree` matches `(Tag::Wpkh, Body::KeyArg)` regardless of use-site (synthesize.rs:374), `use_site_path` enters `encode_payload` (encode.rs:120), so the id differs from canonical synthesis; or (b) keyless card with a retained `Fingerprints` TLV (decodes fine — `validate_xpub_bytes` no-op with no pubkeys, validate.rs:255-258; TLV bits enter the id). And reword §4 row 3 to name the actual carrier of wrong-wallet rejection (mk1-stub-bind + recompose + `--expect-wallet-id`).

## Minor

### M-1. §3.1 "single-sig templates are EXACTLY the canonical-elidable types" — overstated; a strict subset
`canonical_origin.rs:68-70`: `sh(wpkh(@N))` is canonical-elidable (`Some(m/49'/0'/0')`), but `cli_template_from_tree` has no `Sh` arm (synthesize.rs:372-377) and `synthesize.rs:1120-1125` documents the intentional divergence. Safety survives (subset is the safe direction — every emittable single-sig template still has `canonical_origin=Some`), but "exactly" → "a subset of".

### M-2. §6.1 test #1 sketch omits `--mk1`, required for the asserted exit 0
`verify_bundle.rs:697-700`: `mk1_match = chunks == &args.mk1` — `--mk1` absent → expected non-empty vs empty → false → exit 4. The existing harness `tests/cli_verify_bundle_md1_template.rs:88-91` (`verify_args`) always supplies the mk1 cards; the sketch should too.

### M-3. §1/§3.2 line-cite micro-drift (informational)
Multisig partial-decode site is `:2508-2510` (spec says `:2509`); mk1 push block `:712-721` ✓; `error.rs` ModeViolation→2 at `:637` ✓. Nothing load-bearing; re-grep at plan phase.

## Fold-fidelity verification (the 6 design-R0 Minors) — ALL LANDED
- #1 (§4 account rationale): faithful + correct. `verify_bundle.rs:634-636` verbatim; `encode_payload` writes `path_decl` verbatim (encode.rs:119→origin_path.rs:54-66), WDT-id excludes origin+Fingerprints (identity.rs:48-53). Test #8 fixture IS constructible (validate_explicit_origin_required no-op for canonical trees, validate.rs:221-224; fields public; encode_md1_string re-exported lib.rs:55).
- #2 (§5 INV-KEYED / OUT-1): faithful + AIRTIGHT. codex32.rs:25 (80 symbols/400 bits, enforced :89 encode/:174 decode); Pubkeys entry (u8,[u8;65])=520 bits (tlv.rs:32, validate.rs:248-251); is_wallet_policy requires non-empty pubkeys (encode.rs:50-52); zero-length Pubkeys TLV undecode-able (tlv.rs:253-255). No sub-400-bit keyed shape exists → OUT-1 is a proof.
- #3 (residual filing): filed in §0 OUT-2/§7/§8; asymmetry real (chunk-only partial :2508-2510/:3113; md1_partial.rs:55-63).
- #4 (§7 migration note): carries v0.89.0-style wording.
- #5 (§3.2 Vec<String>→&[&str]): precedent at verify_bundle.rs:2902 (.expect :2904; second site :3131-3132).
- #6 (§6.4 test #10): meaningful — mk1_template_stub_bind stays raw compare (:697-700), intake no case-fold (:279-302).

## Checklist answers (VERIFY items)
- Facet 1 snippet type-checks: YES — inspect.rs:205 `decode_card(chunks:&[&str])` + :256-261 ship the exact `match chunks {[single]=>decode_md1_string_with_opts(single,…),_=>reassemble_with_opts(chunks,…)}` (`[single]` binds `&&str`, deref-coerced; compiled + shipped v0.89.0). Discriminator/BCH-before-flag/misread mechanism all confirmed (decode.rs:178-196, codex32.rs:182, chunk.rs:68-72/311-313, header.rs:27).
- Facet 2 `d` is the SUPPLIED card, compare not vacuous: confirmed (:591-592 sig, :578 doc, `&d` passed :394; expected independently synthesized :687-695). Bare `?` compiles (From<md_codec::Error> error.rs:1048; MdCodec→nonzero exit).
- Truth table / verdict routing: accurate (:702-711, :796-831; result∈{ok,mismatch} only; exit `if any_fail {4} else {0}` :831) — EXCEPT row 3 note (I-1).
- Test realizability: #1/#2/#4/#5/#6 realizable (assert_cmd + verify_args). #3 realizable (frozen corpus proves a keyless n=3 general template fits a single string, cli_inspect.rs:217-220). #8/#9 realizable. #7 — see I-1.
- Scope completeness: `:696` is the ONLY supplied-md1 raw equality (grep `\.md1 ==` → 1 hit); `:386-408` sole classify gate; all other args.md1 sites are strip/HRP/plumbing/chunk-only-partial (OUT-2).

## Proof of work (files + line ranges reviewer read)
SPEC (all 176) + design-R0 report (all 82); verify_bundle.rs 279-459/576-965/1100-1120/1200-1215/2005-2020/2500-2515/2895-2915/3104-3135 (+greps); decode.rs 100-196; codex32.rs 15-94/130-199; chunk.rs 60-89/300-329; header.rs 1-50; identity.rs 1-60; encode.rs 40-52/85-126; origin_path.rs 28-82; validate.rs 17-41/205-264; tlv.rs 212-337; lib.rs 52-60; canonical_origin.rs 36-76; synthesize.rs 20-44/369-378/1120-1244; inspect.rs 202-263; md1_partial.rs 50-63; error.rs 260-268/505-520/630-640/1048-1052; cli_inspect.rs 210-235; cli_verify_bundle_md1_template.rs 60-119; Cargo.toml 30-38; FOLLOWUPS.md 28-42.
