# P5 — funds-safety DIFFERENTIAL + PROPERTY + CORPUS (#28 phase 2) — per-phase R0 EXECUTION review (opus architect, verbatim)

> Reviewer: opus architect (R0 per-phase EXECUTION review) — #28 phase 2, P5 (funds-safety differential + property + corpus + P4 M-1 fold), branch `feature/bundle-md1-template-multisig`, commit `26687c5b` (HEAD), reviewed delta `git diff 9007799a..HEAD` (single commit). Source-verified against the working tree; all tests + clippy run by the reviewer.

**Verdict: GREEN — 0 Critical, 0 Important.**

P5 is a clean, tests-only phase. Every oracle is genuinely independent (rust-miniscript `Descriptor::from_str` → `derive_at_index` → `.address()`, or Bitcoin Core `deriveaddresses`), each is proven discriminating by a permanent anti-vacuity self-test, the swaps actually swap and actually differ, the property exercises the NET-NEW keyless-template completion path (not the keyed path), and the bitcoind gate cannot pass-by-skip. The gate may advance to P6 (ship).

---

## Per-deliverable non-vacuity assessment (the heart of this review)

### (1) `prop_template_completion_roundtrip.rs` (NEW, 5 cells) — NON-VACUOUS

**Independence of the golden (a).** `golden_addresses` (test:280-294) parses the ORIGINAL concrete descriptor string with `Descriptor::<DescriptorPublicKey>::from_str`, `.into_single_descriptors().remove(0)`, `derive_at_index(i).address(Bitcoin)`. This is rust-miniscript end-to-end — NOT an md-codec reassemble, NOT the toolkit synth path. The completion side (`complete_id_search`, test:299-341) reads `j["wallets"][0]["first_addresses"]` from the toolkit's `restore --json` stdout. The two sources are fully disjoint: toolkit completion output vs. independent miniscript derivation of the original. Confirmed independent.

**Anti-vacuity swaps actually differ (b).** `oracle_swapped_assignment_changes_address_order_dependent` (test:428-450) builds `wsh-multi` {A@0,B@0} vs the swapped {B@0,A@0} and `assert_ne!`s their first addresses; and the general `or_i` archetype (`gen_general(1)`) {A,B,C} vs {B,A,C} likewise. Both are order-DEPENDENT shapes (`multi`, not `sortedmulti`; `or_i` with role-distinct branches), so the swap genuinely changes the witness script → the address. I confirmed by running: the cell passes (the addresses DO differ). The swap is a real positional swap of the key-build inputs, and the differing-address claim is the exact property that would make the headline oracle vacuous if it were false. Non-vacuity is anchored.

**`swapped_assignment_no_match_refuses` (test:468-498)** proves the runtime failure direction: an id recorded for {A,B} + a cosigner carrying C's key (NOT B) → `assert().failure()`. Because C's key is not in the {A,B} set, no permutation reproduces the recorded id — this is a genuine wrong-KEY-SET refuse, not an order artifact (and it uses `sortedmulti`, where order alone never refuses, so the refuse can only come from the wrong key). This is the funds-safety counterpart to the oracle self-test.

**Generator validity + `.success()` failure policy (c).** `build_case` (test:349-383) materializes valid completable shapes: canonical (4 scripts × {2of2,2of3} at BIP-48) and general (`or_i` 2-key, `or_i` 3-key, `thresh(2,...)` at BIP-84). `complete_id_search` asserts `.success()` (FAILURE POLICY mirrors `prop_backup_restore_roundtrip.rs`) — but a shape that "silently can't complete" is NOT masked: a completion bug surfaces in the *address differential* (`prop_assert_eq!(got, golden)`, test:413-416), and a shape that the gate REFUSED would fail the `.success()` assert loudly (panic), not pass. The two directions (wrong-but-completes → caught by the address diff; refused → caught by `.success()`) are both covered. `every_generated_shape_completes` (test:503-526) is the non-proptest coverage floor: it deterministically sweeps EVERY family/variant (4 canonical + 3 general) and asserts each completes to its golden — so a generator that silently only ever emitted one shape would be caught here, not hidden behind a low `PROP_CASES`.

**Right path (NET-NEW surface).** Every case runs `bundle --md1-form=template` (the keyless template emit) → `restore --md1 … --from … --cosigner … --expect-wallet-id … --json` (the completion/id-search path). `run_bundle` substitutes `--md1-form` between `template` (md1) and `policy` (per-cosigner mk1s) (test:249-259); the `FORM` placeholder substitution is correct. This is the completion path, NOT the keyed path. Confirmed.

**Case budget (d).** `cases: 32` (PROP_CASES-overridable), `failure_persistence: None`, `max_global_rejects: 4`. The input space is tiny and fully enumerable: `family∈{0,1} × variant∈0..4 × {own,b,c}_acct∈0..3` = 2×4×27 = 216 points; 32 cases is a meaningful sample and `every_generated_shape_completes` covers the structural axis exhaustively. ~4 CLI spawns/case → the headline finished in 8.55s. Not so tiny it tests nothing, not so huge it times out. Sane.

**Subtle robustness note (not a finding):** `complete_id_search` indexes `cosigner_groups[idx]` assuming the policy bundle emits mk1 groups in `@N` slot order. I confirmed a 3-key general policy emits exactly 3 `# mk1` groups. But even if that ordering assumption were wrong, the test would remain correct and non-vacuous: the supplied cosigners feed the permutation SEARCH (which resolves the correct assignment for a correct key SET) and the final assertion is vs. the independent golden — a wrong slot→group map carrying the right key set still resolves; a wrong key set refuses (proven). The oracle's correctness does not depend on the helper's indexing. Robust.

### (2) `bitcoind_differential.rs` template rows — NON-VACUOUS, gate cannot pass-by-skip

**CONNECT-ONLY contract preserved.** `bitcoind_template_completion_differential` (test:830+) is `#[ignore]`-gated and calls `read_wiring()` (test:275-293): NONE set → `None` → skip; ALL set → `Some`; partially set → `panic!`. There is no green-by-skip when env is provided-but-broken — a partial provision panics, and a broken connection panics inside `bitcoin_cli` (`out.status.success()` checked, test:305-312). The only runtime gate is `chain == "main"` (test, asserted right after connect).

**Anti-vacuity asserted BEFORE the Core compare.** Verified the statement order in `bitcoind_template_completion_differential`: `derive_receive(&case.descriptor, N+1)` (line 835) → `complete_template` (line 836) → `assert_eq!(reported, independent, "…anti-vacuity…BEFORE Core")` (lines 837-841) — all BEFORE the first `core_addresses` call (line 845). A silently-wrong or unreachable bitcoind therefore cannot make the row pass vacuously; the independent rust-miniscript equality fires first. The row additionally cross-checks the completed addrs vs Core on the ORIGINAL descriptor (lines 859-861), catching a completion that reconstructs a different-but-Core-valid wallet.

**Default-CI leg runs UNCONDITIONALLY.** `template_completion_anti_vacuity_leg` (test:785-) is NOT `#[ignore]` — it runs `derive_receive(&case.descriptor)` vs `complete_template(&case)` for every corpus case, and additionally re-derives from the *reconstructed* descriptor (`recon_independent`, lines 803-808) and asserts it equals the original derivation. So the completion↔independent-oracle equivalence is gated on every CI run with no node. I ran it: PASS (2 default-CI cells green, 2 `#[ignore]` skipped).

**`derive_receive` is genuinely independent** (test:233-250): `Descriptor::from_str` → `into_single_descriptors`/`try_from` → `derive_at_index` → `.address(Bitcoin)`. Not md-codec, not toolkit synth. `core_addresses` shells `deriveaddresses` (test:319-336). Both disjoint from the toolkit's restore output.

**Satellite v0.2.4 vs pinned v27.0 — acceptable; label is cosmetic.** The implementer ran the gated row against a local Bitcoin Satellite v0.2.4 node (a Core v25 fork) rather than the pinned v27.0. The only RUNTIME gate is `chain == "main"`; `deriveaddresses` is a stable, long-settled RPC whose output for these descriptor classes (wsh-sortedmulti, wsh(or_i(...))) is identical across v25→v27. As an *opportunistic* confirmation this is fine — and it is not the CI gate (the default-CI leg is). The hardcoded `"v27.0"` string in the eprintln (test:870-ish) is a cosmetic LABEL, not a version CHECK — leaving it untouched is harmless. **Minor (informational, no fix required):** the eprintln could note the actual node version, but it is non-load-bearing.

### (3) `degrade2_structured_completes_to_golden` (cli_restore, test:1556+) — NON-VACUOUS, realizes the SPEC §7 I-A pin

**Genuine degrade2 analog.** `degrade2_desc` (test, ~1520+) builds `wsh(or_i(and_v(v:after(1000000),and_v(v:sha256(H),pk(@0))), or_i(and_v(v:older(65535),multi(2,@1,@2)), and_v(v:pk(@3),after(1893456000)))))` over 4 BIP-84 slots at accounts {0,1,2,3}. This carries every structural hallmark the SPEC §7 I-A pin demands: an `after()` absolute timelock, an `older()` relative timelock, a `sha256()` hashlock, an inner order-dependent `multi(2,...)`, and FOUR distinct BIP-84 origins (divergent, non-canonical). The test asserts at runtime that it is genuinely general — `canonical_origin(&decoded.tree).is_none()` (md-codec reassemble used only as a *structure assertion*, NOT as the address oracle), `!is_wallet_policy()`, `n == 4`. The address oracle is `general_golden_addresses` (test:1251-1265), independent rust-miniscript. n! = 24 (tractable), matching the SPEC's stated size.

**Multi-own path is real.** `--account 0,3` (test) supplies the SAME seed SEED_A at two accounts (slots @0 and @3); the two cosigners @1 (SEED_B) / @2 (SEED_C) are supplied as `--cosigner`. This exercises the multi-own-account resolution on a divergent-origin general shape — the load-bearing I-A surface (own origins built from `--account` honoring purpose 84', NOT forced to BIP-48). The completion reaches `general_golden_addresses` byte-equal — a `compute_default_origin_path` (BIP-48) rebuild would FAIL the search here, so this is a positive proof the per-slot-build is implemented.

**Anti-vacuity swaps differ.** `degrade2_structured_anti_vacuity_swapped_assignment_differs` (test, ~1600+) proves TWO independent role distinctions: (a) the @0↔@3 own-key swap (SEED_A@0 ↔ SEED_A@3) — the two own keys live in different spending roles (@0 = sha256-gated after(1000000) branch; @3 = pk + after(1893456000) branch) — `assert_ne!` on first addresses; (b) the @1↔@2 cosigner swap inside the order-dependent `multi(2,...)` — `assert_ne!`. Both are genuine positional swaps of the key-build inputs, both genuinely differ (I confirmed the cell passes). The own-key swap is the critical one: it proves a completion that placed the own keys in the wrong roles could NOT match the golden vacuously.

### (4) `verify_bundle_canonical_wsh_multi_template_id_search_ok` (cli_verify, test:373+) — NON-VACUOUS, faithfully closes P4 M-1

**Order-DEPENDENT wsh-multi (not sortedmulti).** `golden_addresses("wsh-multi", 2, cos, false, 1)` (test, `sorted=false`) builds `wsh(multi(2,…))` (test:219-228) — order-dependent. The existing `..._id_search_ok` happy-path used `wsh-sortedmulti`; this closes the verify-side parity matrix symmetrically, which is exactly what the P4 review's M-1 finding requested (verified against `template-multisig-p4-verify-bundle-exec-review.md:64`).

**Real byte-equal parity.** Three distinct toolkit/independent comparisons: (i) `j["first_receive"] == golden[0]` (verify recompose == independent golden in the resolved order); (ii) the binding checks `md1_template_match` + `mk1_template_stub_bind` are present AND `passed==true`; (iii) PARITY: `restore_first_address(&rargs)` (toolkit restore's `first_addresses[0]`, test:278-287) `== j["first_receive"]` (toolkit verify's). The parity is a genuine byte-equal assertion between two distinct toolkit subcommand outputs, each also anchored to the independent golden. Confirmed.

---

## Tests-only confirmation

- `git show 26687c5b --name-only` touches exactly four files, all under `crates/mnemonic-toolkit/tests/`: `prop_template_completion_roundtrip.rs` (NEW), `bitcoind_differential.rs`, `cli_restore_md1_template_multisig.rs`, `cli_verify_bundle_md1_template_multisig.rs`. +1145 lines, 0 deletions.
- `git show 26687c5b --name-only | grep -E 'src/|mlock'` → NONE. No production code, no `mlock.rs`.
- `git diff --name-only -- crates/mnemonic-toolkit/src/` → empty. `git diff HEAD -- …/parse_descriptor.rs` → empty; `parse_descriptor.rs` is NOT in the P5 commit (the orchestrator's stray-whitespace revert is confirmed clean — working tree has no source diff). The only untracked files are unrelated cycle-prep recon `.md` docs.
- Crate version is still `0.59.1` (correct — P6 bumps to 0.60.0).

## Regression / build evidence (counts the reviewer ran)

| target | expected | observed |
|---|---|---|
| `--test prop_template_completion_roundtrip` | 5 | **5 passed** (0 failed, 0 ignored; 8.55s) |
| `--test cli_restore_md1_template_multisig` | 27 | **27 passed** |
| `--test cli_verify_bundle_md1_template_multisig` | 7 | **7 passed** |
| `--test bitcoind_differential` | 2 default-CI + 2 ignored | **2 passed, 2 ignored** (`*_anti_vacuity_leg` + `divergent_differential_golden` pass; both `#[ignore]` rows skip) |
| `--test prop_backup_restore_roundtrip` | 13 | **13 passed** (unchanged) |
| `--test cli_bundle_md1_template_multisig` | 13 | **13 passed** |
| `--lib` | 158 | **158 passed** (3 ignored) |
| `clippy --tests --bins --lib -- -D warnings` | clean | **clean** (0 warnings) |

Every count matches the plan's claim. The property test did not flake or time out.

## Determinism

The property uses `failure_persistence: None` + `PROP_CASES`-overridable `cases: 32`, mirroring the sibling `prop_backup_restore_roundtrip.rs` (config at :405-414/:463-466). Reproducibility here is actually STRONGER than seed-based proptest: (1) the input space is 216 points and `every_generated_shape_completes` deterministically (non-proptest) enumerates the full structural axis; (2) a headline counterexample prints `w.descriptor` (the full concrete descriptor string, test:415-416) — a complete repro recipe, not an opaque seed — so a found counterexample is trivially reproducible as an explicit `#[test]`. No heisenbug surface.

## New findings

- **Critical:** none.
- **Important:** none.
- **Minor (informational, no fix required):** the gated `bitcoind_template_completion_differential` eprintln hardcodes the `"v27.0"` label while the opportunistic run used Satellite v0.2.4 (Core v25 fork). It is a cosmetic label, not a check; `deriveaddresses` output for these shapes is version-stable; the CI gate is the unconditional anti-vacuity leg, not this row. Acceptable as-is; optionally surface the live node version in the message in a future cycle.

## Gate decision

**GREEN — 0 Critical / 0 Important.** P5 genuinely ADDS over P3/P4: a randomized property over the keyless-completion surface, a degrade2-structured multi-own general differential (the SPEC §7 I-A funds-safety pin), a bitcoind template-completion corpus with an unconditional default-CI anti-vacuity gate, and the faithful P4 M-1 fold. No oracle is vacuous or self-referential; every anti-vacuity swap actually differs; the property tests the correct (template-completion) path; the bitcoind gate cannot pass-by-skip. Tests-only, no regression, clippy clean. **The hard gate may advance to P6 (ship).**
