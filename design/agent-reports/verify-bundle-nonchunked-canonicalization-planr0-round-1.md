# R0 Review (round 1) — IMPLEMENTATION_PLAN_verify_bundle_nonchunked_canonicalization.md

**Reviewer:** Fable architect (`model:"fable"`), 2026-07-12. **Source SHA:** `de140a08`. **Usage:** 35 tool-uses, ~833s, 151442 tokens.
**Main-loop verification:** I-1 (no `--force-chunked`/`--chunk-size` in bundle.rs — confirmed), I-2 (`pre_check_template_n:107` rejects n==1 multisig — confirmed), I-4 (fuzz at repo-root `fuzz/`, `fuzz/Cargo.lock:578-579` pins 0.89.0 — confirmed) all verified against source before folding.

## Verdict: RED — 0 Critical / 4 Important / 3 Minor
Both source snippets compile and land on the correct sites; 8/10 test cells constructible + correctly sequenced. Defects concentrated in two fixtures, one coverage gap, one release-path.

## Important
### I-1 — Task 1 Step 3 `verify_bundle_chunked_multichunk_template_unchanged`: unconstructible + premise impossible
`bundle` has NO `--force-chunked`/`--chunk-size` flag (full `BundleArgs` enum, bundle.rs:22-180; the only `--force-chunked` string is advisory md-CLI text at friendly.rs:390). `chunk::split` takes no size param (chunk.rs:240); a keyless single-sig template is far under the 80-symbol cap (codex32.rs:25) → the toolkit can NEVER emit a multi-chunk single-sig template md1; any hand-forced multi-chunk form fails the raw `expected.md1 == args.md1` (:696) on master → the "GREEN before AND after" requirement is violated (Step 4 RED on master). **Fix:** re-fixture as a KEYED policy-form bundle verify (keyed cards are naturally multi-chunk — cf. 3-chunk SS_MD1_ORIGIN); it enters the same classify `match`, exercises `_ => reassemble` (len>1), skips both template branches (is_wallet_policy, encode.rs:50-52), verifies via general path — GREEN before and after both facets.

### I-2 — Task 1 Step 1 primary multisig fixture is broken; promote the NOTE fallback to primary
`template_cards("wsh-sortedmulti", PHRASE_A, "0")` = `bundle --template wsh-sortedmulti --md1-form template --slot @0…` at n=1, no `--threshold` → `pre_check_template_n` (bundle_unified.rs:107-113) rejects `is_multisig_template && n==1` ("requires N>1") → the helper's `.assert().success()` (:36-37) PANICS in fixture construction → Step-2 "verify RED" is a fixture panic, not the asserted stderr failure (corrupts the TDD RED signal). The NOTE fallback is verified constructible: `canonical_multisig_template_args("wsh-sortedmulti", "2", cosigners)` (cli_bundle_md1_template_multisig.rs:78-107, used :366) emits a keyless 2-of-2 elided-origin template (canonical origin, canonical_origin.rs:59), <400 bits → `to_nonchunked` works. **Fix:** make the fallback primary; delete the dead primary.

### I-3 — Spec §6.1 #3 coverage gap: positive non-chunked multisig verify (+`--from`) never tested
The plan's only multisig cell asserts the no-`--from` refusal (routing proof, floor :876-892, msg has "--from"+"seed" via main.rs:243-247 — sound). But nothing proves OUT-3's "free ride": a non-chunked multisig template actually verifies GREEN end-to-end through the WDT-id compare (:937-941). **Fix:** add one positive cell mirroring the chunk-form positive in cli_verify_bundle_md1_template_multisig.rs (`--from` :306 + `--cosigner` mk1s), md1 passed through `to_nonchunked`.

### I-4 — Task 3 fuzz paths wrong: `crates/mnemonic-toolkit/fuzz/` does not exist
Fuzz crate is at repo root `fuzz/`; `fuzz/Cargo.lock:578-579` pins mnemonic-toolkit 0.89.0 (needs refresh). As written, `( cd crates/mnemonic-toolkit/fuzz && cargo check )` fails, the "if present" hedge silently skips a §7 site, and `git add crates/mnemonic-toolkit/fuzz/Cargo.lock` aborts the release commit with a pathspec error. **Fix:** `( cd fuzz && cargo check )` + `git add fuzz/Cargo.lock`.

## Minor
- **M-1:** `verify_bundle_form_equivalence_same_verdict` compares stdout only; per-check ✓/✗ lines go to STDERR (:811-820) and spec #5 wants "identical verdict + `--json` shape". Add stderr and/or `--json` equivalence.
- **M-2:** Task 3 Step 5 commits `design/agent-reports/…-*.md`, but the post-impl whole-diff report is written in Step 6 AFTER that commit → untracked at tag. Add an explicit commit of the post-impl report + plan folds before `git tag`.
- **M-3:** Task 3 omits spec §7's "`.examples-build/` — confirm unaffected". Add the one-line confirm.

## Verified-sound (against source)
- **Facet 1**: site :387-388 exact; `[single]` binds `&&str` deref-coerces to `decode_md1_string(&str)` (decode.rs:178-180); both arms `Result<Descriptor,Error>`; mirrors shipped inspect.rs:256-261; crate-root re-exports lib.rs:46-66; chunked-of-1 byte-identical.
- **Facet 2**: :696 exact; `d: &Descriptor` is the SUPPLIED card (sig :591-598, `&d` at :394); `compute_md1_encoding_id(&Descriptor)->Result<Md1EncodingId,Error>` PartialEq (identity.rs:15,39-45); `?` compiles (From<md_codec::Error> error.rs:1048); :2902-2904 precedent verbatim.
- **Test #7** constructible (TlvSection.fingerprints Option<Vec<(u8,[u8;4])>> tlv.rs:24-39; no fingerprints-require-pubkeys rule; is_wallet_policy keys on pubkeys; expected strips fingerprints → ids differ → false/exit 4).
- **Test #8** constructible (emitted templates ship elided origin, synthesize C1 :1216; encode_payload writes path_decl verbatim no fill, encode.rs:99-126/:119; strict decode accepts explicit-canonical, validate.rs:221-224; re-exports lib.rs:62).
- **Test #9** constructible, GREEN both sides (SS_MD1_ORIGIN verbatim cli_repair_dead_card_strict.rs:20-24; encode_payload no explicit-origin validator encode.rs:103-111; wsh(pk) non-canonical; master WireVersionMismatch→fallthrough→exit 2; after Facet1 MissingExplicitOrigin→same).
- **Test #10** sound (verify_singlesig_template never decodes args.mk1; only byte-compare :697-700 → uppercase → false → exit 4; fallback unnecessary but harmless).
- **Sequencing**: Task-1 #3 RED on master (fallthrough msg names --template not --from); Task-2 #1 RED after Task1; no existing test locks non-chunked verify-bundle exit-2.
- **Task 3 other sites**: Cargo.toml:3=0.89.0; READMEs (README.md:13, crate README:9); install.sh:32 self-pin; serde_json+md_codec already test deps.

(Full proof-of-work table in transcript.)
