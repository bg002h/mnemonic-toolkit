# cycle-4 Track A — whole-diff adversarial execution review

- **Scope:** md-codec funds-safety domain caps — H6 (encode 80-data cap), M4 (correcting-decode `>93` reject), I1 (non-correcting-decode `>93` cap).
- **Worktree:** `/scratch/code/shibboleth/wt-cycle4-md`, branch `feature/cycle4-md-codec-bch`.
- **HEAD reviewed:** `a39b935` (3 commits: `f9c1e57` H6 · `8979301` M4 · `a39b935` I1).
- **Base:** `58cc9ec` (`origin/main`).
- **Date:** 2026-06-21.
- **Reviewer:** opus software architect (independent post-implementation adversarial review; R0 validated the plan, this catches implementation-introduced defects TDD missed).
- **Build/test status:** `cargo test --workspace --all-targets` = **615 passed / 0 failed / 1 ignored**; `cargo clippy --all-targets -- -D warnings` = **clean**.

---

## Methodology

- Read the full source + test diff (`git diff 58cc9ec..a39b935`) against the brainstorm spec §3/§4/§5/§5.2.3.
- Verified every symbol the new tests reference is in scope (`BETA`, `MD_REGULAR_CONST`, `polymod_run`, `hrp_expand`, `bch_create_checksum_regular`, `Gf1024::ONE`, `symbol_to_char`, `descriptor_from_tree`, `corrupt_chunk_at`).
- **Empirically measured** the data-symbol counts of the re-routed full descriptors (instrumented throwaway test, since removed) to adjudicate "is 80 the correct cap."
- **Mutation-tested all three guards** — reverted each guard in turn (`>MAX` → `>99999`) and confirmed the corresponding RED test fails, then restored.
- Confirmed proptest generators were NOT narrowed and the parallel chunk-path proptests (P5/P5(W)) still assert exact round-trips over the same generated descriptors.

---

## Critical

**None.**

## Important

**None.**

## Minor

**M-1 (informational, no fix required) — M4 boundary guard is redundant-but-correct with the internal floors.**
When the `chunk::decode_with_correction` boundary guard (`chunk.rs:528`) is mutated out, the over-93 chunk does NOT alias: it falls through to `TooManyErrors` because the internal `decode_regular_errors`/`chien_search` `None`-floors (`bch_decode.rs:420`/`:290`) still fire. This is the intended belt-and-suspenders behavior (spec §5.2 D5): the boundary guard supplies the *accurate* typed `ChunkSymbolCountOutOfRange`, and even a caller bypassing it cannot enter the aliasing scan. The layering is correct; noted only so a future reader understands why the boundary-guard mutation degrades to `TooManyErrors` rather than mis-correcting. No action.

**M-2 (informational) — M4 RED test's corruption is belt-and-suspenders, not load-bearing.**
`decode_with_correction_rejects_over_93_symbol_chunk` (`bch_adversarial.rs`) corrupts the forged 103-symbol word to force `residue != 0`, but the boundary guard fires on `symbols.len()` BEFORE residue is computed (`chunk.rs:528` precedes `:539`). The reject therefore does not actually depend on the corruption — a clean 103-symbol chunk would reject identically. The test is still correct and the corruption documents the historical aliasing harm; noted only that the `corrupt_chunk_at` step is not what triggers the post-fix reject. No action.

---

## Adversarial findings on the highest-risk area (the 13 modified pre-existing tests)

### (a) Were all 13 modified tests legitimately RE-ROUTED, or silently WEAKENED?

**CLEAN — all legitimately re-routed; none weakened, ignored, deleted, or trivially-passing.**

Inventory of the modified-test assertions, before → after:

| file | test(s) | property before | property after | verdict |
|---|---|---|---|---|
| `address_derivation.rs` | `round_trip_then_derive_address` | single-string RT then `derive_address` equality | asserts `PayloadTooLongForSingleString` on single-string encode AND does the SAME `derive_address` equality via `split`/`reassemble` | re-routed; address-equality preserved + a NEW positive reject assertion |
| `mixed_case_reject.rs` | `one_chunk()` helper feeds 7 single-string mixed-case / uppercase-RT tests | `descriptor_with_pubkeys` (populated TLV → 104+ data syms, over cap) | `descriptor_from_tree(..., false)` (empty TLV, template mode → ~12 data syms, under cap) — the mixed-case-reject / uppercase-RT assertions are UNCHANGED and still run on a true single string | re-routed to a smaller in-domain descriptor; SAME assertions, single-string path kept alive |
| `proptest_roundtrip.rs` | `p4_string_round_trip`, `p4_w_string_round_trip` | `expect("string encodes")` + exact RT | `Ok(s) ⇒` exact RT unchanged; `PayloadTooLongForSingleString ⇒` tolerated (chunk RT covered by unchanged P5/P5(W)); ANY OTHER error ⇒ `prop_assert!(false)` | widened acceptable-outcomes for a now-capped path; exact-RT still asserted on the `Ok` arm; coverage preserved by the untouched P5 siblings |
| `proptest_to_miniscript.rs` | `p6_chain`, `assert_p7_clean_refusal`, `upstream_taptree_depth2_display_asymmetry` | inline single-string RT exact | extracted to `assert_string_round_trip_or_oversize_reject` helper: `Ok ⇒` exact RT; `PayloadTooLongForSingleString ⇒` tolerated; other ⇒ panic. The chunk RT in p6/p7 (`split`/`reassemble` exact-eq) is UNCHANGED | re-routed; exact RT still asserted when it fits; chunk RT remains the authoritative wire check (unchanged) |
| `wallet_policy.rs` | 5 round-trip tests (`smoke_1of1_cell_7`, `smoke_2of3_cell_7`, `canonicalization_stability…`, `partial_keys_2of2…`, `divergent_paths…`) + `encoder_determinism_2of3_cell_7` | single-string encode/decode `.unwrap()` then `assert_eq!(d, d2)` | `roundtrip_via_string_or_chunks` helper: `Ok ⇒` single-string decode; `PayloadTooLongForSingleString ⇒` `split`/`reassemble`. `assert_eq!(d, d2)` + `is_wallet_policy()` assertions UNCHANGED. Determinism test now asserts the reject + `split()` determinism | re-routed; structural-equality + wallet-policy-mode assertions preserved |
| `cmd_address.rs` | `address_accepts_grouped_phrase`, `address_phrase_mode_round_trips_through_encode` | encode single phrase → `md address <phrase>` → expected-address `contains` | `--force-chunked --group-size 0`, collect all `md1` lines, pass all to `md address`; the `predicates::str::contains(<expected address>)` assertion is UNCHANGED | re-routed; address-match assertion preserved |
| `cmd_encode.rs` | `encode_json_network_field_testnet` | `--json` emits `"network":"testnet"` | adds `--force-chunked`; SAME `"network":"testnet"` contains-assertion | re-routed; JSON-field assertion preserved. (`md_encode_default_rejects_oversize` is NEW, not modified) |
| `h13_hardened_multipath_reject.rs` | `encode_nonhardened_multipath_roundtrips` | encode single phrase → decode → `assert_eq!(decoded, template)` (multipath must NOT collapse to `/*`) | `--force-chunked --group-size 0`, collect `md1` chunks, decode all, SAME `assert_eq!(decoded, template, "…NOT collapse to a bare /*")` | re-routed; the load-bearing H13 anti-collapse assertion is byte-identical |

No test was `#[ignore]`d, deleted, downgraded to a weaker predicate, or turned into a tautology. Every round-trip / address-match / anti-collapse property is preserved; several gained an ADDITIONAL positive assertion that the oversize single-string encode rejects with the typed error.

### (b) Is 80 the correct cap — were those descriptors genuinely out-of-domain?

**CLEAN — empirically confirmed genuinely out-of-domain.** Instrumented measurement of the re-routed full descriptors (via `encode_payload` bit-count → `ceil(bits/5)` data symbols):

- `cell_7_wpkh_full` (1-of-1, one populated 65-byte xpub TLV): **129 data symbols** → `encode_md1_string` = `false`/reject. (129 > 80.)
- `cell_7_wsh_2of3_full` (2-of-3, three xpubs): **358 data symbols** → reject. (358 ≫ 80.)
- `cell_1_wpkh_template_only` (empty TLV, template mode): **12 data symbols** → `encode_md1_string` = `Ok`. (Still encodes single-string.)

129 and 358 data symbols cannot be valid codex32 regular codewords — BCH(93,80,8) caps data at 80. Rejecting them is the **correct latent-bug fix** (pre-fix they emitted out-of-code single strings); the cap is NOT set too low. The positive controls `wrap_payload_accepts_exactly_80_data_symbols` and `unwrap_string_accepts_exactly_93_symbol_codeword` both pass, proving the maximal-legal value (80 data / 93 codeword) still round-trips — no off-by-one over-reject. The template-mode re-route descriptors (12 data symbols) remain comfortably single-string, so `mixed_case_reject.rs` still exercises the single-string path.

### (c) Cap boundaries correct (80 data / 93 codeword)?

**CLEAN — the two quantities are correctly distinguished, never conflated:**
- **H6 encode** (`codex32.rs:89`): `data_symbols.len() > REGULAR_DATA_SYMBOLS_MAX` (=80), strict `>`, on the **data** symbol vector (pre-checksum). Correct.
- **I1 non-correcting decode** (`codex32.rs:174`): `symbols.len() > REGULAR_CODE_SYMBOLS_MAX` (=93), strict `>`, on the **full codeword** (data+checksum). Correct.
- **M4 correcting decode** (`chunk.rs:528` boundary; `bch_decode.rs:290`/`:420` internal floors): `> REGULAR_CODE_SYMBOLS_MAX` (=93) on the full codeword length. Correct.
- The constant `REGULAR_CODE_SYMBOLS_MAX = REGULAR_DATA_SYMBOLS_MAX + REGULAR_CHECKSUM_SYMBOLS` is computed, self-documenting (80 + 13 = 93); `REGULAR_CHECKSUM_SYMBOLS = 13` is the live constant. No magic numbers in the guards.
- Boundary direction confirmed by mutation: the `>` (not `>=`) boundary plus the two positive controls (exactly-80 / exactly-93 succeed) prove the maximal legal value passes and the first over-value rejects.

---

## Other checks

- **Fail-closed:** every guard returns a typed `Err` (or internal `None` floor), never a silent truncation or wrong-accept. The M4 boundary guard (`chunk.rs:528`) runs BEFORE the `residue == 0` pass-through (`chunk.rs:541`), so a CLEAN over-length chunk is rejected too — verified by reading the ordering and by the spec's intent (§5.2 item 2).
- **No new panic / unwrap / OOB in the guards:** the three guards are pure length comparisons returning `Err`/`None`; no indexing, no `unwrap` on attacker input. The pre-existing defensive `pos >= corrected.len()` check (`chunk.rs:563`) is untouched.
- **Variant placement / exhaustiveness:** md-codec `Error` uses thiserror's `#[derive(Error)]` (Display/Debug derived via `#[error(...)]` attributes) — no manual exhaustive `match self` to break. md-codec is NOT `#[non_exhaustive]`, but no intra-crate exhaustive match exists; md-cli wraps via opaque `CliError::Codec(_)` (no per-variant match). The three additive variants compile cleanly (615 tests build+pass). `PartialEq/Eq` derive holds (usize fields). Messages name `--force-chunked` (H6) / the 93 cap (M4/I1).
- **Mutation tests:** reverting each of the three guards individually makes exactly the corresponding RED test FAIL (`wrap_payload_rejects_over_80_data_symbols`; `unwrap_string_rejects_clean_over_93_symbol_string`; `decode_with_correction_rejects_over_93_symbol_chunk`). The internal `bch_decode` floors are independently covered by `decode_regular_errors_returns_none_for_len_over_93` and `chien_search_returns_none_for_len_over_93`.
- **Proptest coverage NOT reduced:** `descriptor_strategy()` / `wire_descriptor_strategy()` generators and `common/mod.rs` strategies are UNCHANGED (no `prop_filter`, no narrowed `prop_oneof`). The `Ok` arms still assert exact round-trip; the only widening is tolerating `PayloadTooLongForSingleString` (with the unchanged P5/P5(W) chunk-RT proptests covering exactly those oversize descriptors). Any other error still fails the property.
- **Out-of-scope items correctly left alone:** L7 (the `md repair` help-epilog prose) and the toolkit-side `md_codec_exit_code` exhaustive-match lockstep (§7.3) are explicitly NOT part of Track A's md-codec diff — they are consumer-side, handled in the downstream pin-bump PATCH. This diff is md-codec + md-cli only; md-cli inherits via the opaque wrapper (no per-variant arm needed), consistent with spec §7.1.

---

## Verdict

**TRACK-A WHOLE-DIFF: 0C / 0I** — **GREEN (0 Critical / 0 Important, cleared to tag/publish).**

Two informational Minors (M-1, M-2) document correct-but-subtle layering; neither requires a change. The 13 modified tests are all legitimately re-routed (not weakened); 80 is the correct cap (re-routed descriptors empirically measured at 129 / 358 data symbols, genuinely out-of-domain); the two cap boundaries (80 data for encode, 93 codeword for decode) are correctly distinguished and strict-`>` with passing positive controls at the exact boundary.
