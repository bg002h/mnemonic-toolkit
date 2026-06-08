# R0 Architect Review — SPEC_trmultia_nums_internal_key.md — Round 1

> Reviewer had Read/Grep across both repos (no Bash/cargo — runtime decode-verify is the implementer's §7 job). Source basis: SPEC's `b642fbe`.

## Verdict: RED — 0 Critical / 2 Important / 3 Minor

The core wire change is correct and safe (`is_nums:true, key_index:0` valid in md-codec; tr-multi-a renders; restore re-scope accurate; no backward-compat consumer breaks). But two Important: Item 3 rests on a false premise (affected goldens are ORPHANED → suite false-greens → ships stale fixtures w/ no NUMS assertion), and Item 6's framing is inverted (manual already documents NUMS).

## Critical
None.

## Important

### I1 — Item 3's premise is false: tr-multisig goldens are ORPHANED → suite reports zero failures → ships stale fixtures + no NUMS-value assertion
Traced every consumer of `tests/vectors/v0_2/`: `cli_self_check.rs:13` reads only `bip84-mainnet-0-false-true.txt`; `cli_bundle_full.rs:17` reads only `vectors/v0_1/{bip44,49,84,86}` single-sig; NO `read_dir`/`insta::glob`/`WalkDir` harness anywhere; the `tr-multi-a-mainnet-0-false-false.txt` md1 prefix appears in exactly one file (itself). So the 4 `tr-multi-a-*-0-false-false.txt` + 4 `tr-sortedmulti-a-*` vectors are DEAD goldens with no harness. An implementer runs `cargo test` after the code change → GREEN → concludes Item 3 is a no-op → ships, leaving those vectors showing the OLD `is_nums:false` wire (now contradicting the code) AND zero byte-level assertion the new md1 is NUMS. (Live EXECUTION coverage exists — `cli_restore_multisig.rs::tr_multisig_refused_exit2`, `cli_tr_bip48_advisory.rs` ×4 bundle tr-sortedmulti-a — but their assertions are is_nums-independent (restore-refusal exit 2, advisory string), so they pass unchanged without pinning is_nums.)
**Fix:** rewrite Item 3 to (1) decisively dispose of the orphaned `tr-*-0-false-false.txt` vectors (delete them as dead, OR regenerate AND wire a reading test — pick one); (2) promote Item 4 from "Add/confirm" to a MANDATORY gating characterization test decoding a freshly-emitted `bundle --template tr-multi-a` md1 and asserting `is_nums == true` (+ leaf `indices: 0..n`, first cosigner xpub unchanged) — the only thing pinning the new wire; (3) note the change shifts the whole bundle (M1), so any regen is full re-bless.

### I2 — Item 6 inverted: the manual ALREADY documents the bundle as NUMS
`docs/manual/src/30-workflows/33-taproot-multi.md` already states NUMS: `:13-14` "NUMS internal key — a verifiable nothing-up-my-sleeve point"; `:50` table `nums | BIP-341 reference NUMS point`; `:54-58` "NUMS variant — the *default* for tr-multi-a and tr-sortedmulti-a... The bundle embeds the BIP-341 reference NUMS point as the internal key"; `:83-85` "`tr(NUMS_POINT,sortedmulti_a(2,@0,@1,@2))`"; `:70-81` a `bundle --template tr-sortedmulti-a --self-check` example whose `:84` claim is only true AFTER this fix. So the code has been documenting-NUMS-but-emitting-@0 — a latent docs-vs-code drift this fix CLOSES. Item 6's "hunt for @0-prose, update to NUMS" reaches the right action (no edit) for the wrong reason and skips verifying the now-load-bearing claim.
**Fix:** reframe Item 6 — the manual already claims NUMS; this fix makes the code conform (correctness win). Make it positive verification: confirm `33-taproot-multi.md:84` + the `:70-81` self-check example are accurate post-fix (run the example, md1 decodes to `tr(NUMS,…)`). No manual edit needed (confirmed: no stale @0-internal bundle prose exists — technical-manual documents the wire mechanism generically; `tr(@0/**)` hits in `45-foreign-formats.md`/`cli_export_wallet.rs` are BIP-86 single-sig template-mode, unaffected) — but say so for the right reason.

## Minor
- **M1 — blast radius understated.** `wrapper_node → build_descriptor → compute_wallet_policy_id → 4-byte stub` (`synthesize.rs:156-159,188-191`) seeds BOTH mk1 `chunk_set_id` (`derive_mk1_chunk_set_id`, `:44-46,166`) and md1. NUMS changes the descriptor → policy_id → BOTH md1 AND mk1 cards for any tr-multisig bundle. Self-check stub-linkage stays consistent. SPEC's "alters the md1 wire bytes" should read "md1 AND mk1".
- **M2 — §7.5 schema_mirror.** Correct (wire output, not flag names/value-enums) — add the half-sentence rationale.
- **M3 — Item 5 concreteness.** Promote "consider a sibling md-codec FOLLOWUP" → definitely file the md-codec SortedMultiA-rendering FOLLOWUP with a `Companion:` cross-cite. And name `restore.rs:777` (`d.tree.tag == md_codec::Tag::Tr` refuses ALL Tr md1) as the specific tr-multi-a pre-gate-lift target.

## Verified-correct (cross-repo source)
- `is_nums:true, key_index:0` VALID — `validate.rs:94` gates the `key_index>=n→NUMSSentinelConflict` on `if !*is_nums`; corroborated `parse_descriptor.rs:468-470`.
- tr-multi-a renders `tr(NUMS, multi_a(…))` (`to_miniscript.rs:161-165` NUMS key, `:394-398` MultiA leaf — independent paths).
- tr-sortedmulti-a still errors regardless of is_nums (`:406-410` unconditional). **Restore re-scope ACCURATE.**
- Leaf stays correct: cosigner @0 remains a script-path key (`indices: 0..n` unchanged); only internal key moves @0→NUMS; `tlv.pubkeys.first()` unchanged.
- No backward-compat break: `restore.rs:777` refuses all Tr before to_miniscript; `self_check_bundle` (`bundle.rs:2045-2179`) does no whole-md1 byte-compare (recomputes stub from same fresh bundle); `verify-bundle md1_xpub_match` (`verify_bundle.rs:2052-2068`) compares `tlv.pubkeys.first()` (unchanged). No test pins the old `tr(@0)` bundle shape.
- Version: `Cargo.toml:3` = 0.47.4; MINOR→0.48.0+tag correct on a 0.x crate for a wire-content change. NO md-codec change needed (already supports is_nums:true end-to-end; `is_nums` is a v0.30+ on-wire field → no wire-version bump).
- Test-lock flip correct: `template.rs:446` `assert!(!is_nums)` → `assert!(is_nums)`; keep `assert_eq!(key_index,0)` + MultiA-leaf assertion.

## Required to GREEN
Fold I1 (Item 3 rewrite + mandatory gating Item-4 test + whole-bundle note) + I2 (Item 6 reframe to positive verification). M1-M3 same pass. Persist + re-dispatch R0.
