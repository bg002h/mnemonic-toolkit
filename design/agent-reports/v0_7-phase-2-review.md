# v0.7 Phase 2 — Code Quality Review (Casascius mini-key decode)

**Date:** 2026-05-06
**Files:** `crates/mnemonic-toolkit/src/cmd/convert.rs`, `crates/mnemonic-toolkit/tests/cli_convert_minikey.rs`
**Tests:** 376 → 386 (+10 new). 0 failed.

## Implementation summary

- New direct edge `(MiniKey, Wif)` in `is_supported_direct_edge` (`convert.rs:295`). One-way; no `(Wif, MiniKey)` reverse.
- `classify_edge` (`convert.rs:308-313`) intercepts both `* → MiniKey` and `MiniKey → !Wif` with `refusal_minikey_one_way`.
- `compute_outputs` `MiniKey` arm (`convert.rs:817-849`):
  1. Length check ∈ {22, 26, 30} AND `value.starts_with('S')` — refuse `refusal_minikey_invalid_format` on miss.
  2. Self-checksum: `sha256::Hash::hash(value || b'?').as_byte_array()[0] == 0x00` — refuse `refusal_minikey_invalid_checksum` on miss.
  3. Privkey scalar = `sha256::Hash::hash(value).to_byte_array()`.
  4. `PrivateKey { compressed: false, network: args.network.network_kind(), inner: SecretKey::from_slice(&raw)? }.to_wif()` — `compressed: false` per SPEC §13 (Casascius predates BIP-32 compressed-pubkey convention).
- 3 new refusal helpers (`refusal_minikey_one_way`, `refusal_minikey_invalid_format`, `refusal_minikey_invalid_checksum`); byte-text matches the task spec / SPEC §3.d / SPEC §13.
- `is_secret_bearing` and `is_side_input_only` confirmed UNCHANGED (MiniKey is decode-only, so it never appears as an output node — moot).

## Test coverage

Reference vectors (`cli_convert_minikey.rs`):

- **22-char canonical** `SzavMBLoXU6kDrqtUVmffv` (Casascius wiki: <https://en.bitcoin.it/wiki/Mini_private_key_format>) → mainnet uncompressed WIF `5Kb8kLf9zgWQnogidDA76MzPL6TsZZY36hWXMssSzNydYXYB9KF`.
- **30-char canonical** `S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy` (Casascius wiki) → mainnet uncompressed WIF `5JPy8Zg7z4P7RSLsiqcqyeAF1935zjNUdMxcDeVrtU1oarrgnB7`; testnet uncompressed `92AbiJVfaHTFPVrAMBWkrEiCeoPo9tufyJpZJGrNECkrMrR4VGx`.
- **26-char fixture** `S2WSthnpsFbmS1btGUBjCNjG5r` — **brute-forced** (Python `random.Random(seed=1)` selecting from base58 alphabet after literal `S`; first candidate satisfying SHA256(key+"?")[0]==0x00). Public canonical 26-char Casascius vectors are not widely cataloged on the wiki; this fixture is test-only with the brute-force seed cited in the test comment for reproducibility.

Refusal byte-pins (4 cases):

- `refusal_minikey_invalid_format` — wrong S-prefix (`NotS22Charsxxxxxxxxxx`); wrong length (23-char `Sxxxxxxxxxxxxxxxxxxxxxx`).
- `refusal_minikey_invalid_checksum` — 22-char S-prefixed but `Sxxxxxxxxxxxxxxxxxxxxx` fails self-checksum (verified: SHA256(key+"?")[0]=0x20).
- `refusal_minikey_one_way` — `minikey → xpub`, `minikey → phrase`, `wif → minikey` all funnel into the §3.d catch-all.

Network coverage: at least one mainnet (`5...`) AND one testnet (`9...`) WIF prefix asserted.

## Self-review findings

**Strengths**

1. **DRY constants.** All 3 length-class fixtures + their expected mainnet/testnet WIFs declared once at top of test file; reused across 4 happy-path tests + 4 refusal tests.
2. **No new clippy lints.** `cargo clippy -p mnemonic-toolkit --tests` shows zero new warnings on the touched lines. (Pre-existing baseline lints at `convert.rs:726`, `verify_bundle.rs:938`, etc. unchanged — out of Phase 2 scope.)
3. **`unreachable!` is correctly scoped.** The inner `match t { Wif => ..., _ => unreachable!("classify_edge intercepts (MiniKey, !Wif)") }` is sound — `classify_edge` runs in `run()` BEFORE `compute_outputs`, so any non-Wif target is rejected upstream.
4. **No panics; no `unwrap` on user input.** `from_slice` returns `Result` (mapped to `BadInput`); the only infallibility is the SHA-256 hash itself.

**Issues**

- *None at Critical/Important tier.* Two Low-tier observations:

  - **[L1] 26-char vector is brute-forced, not canonical.** Public 26-char Casascius keys are rare in published references (the wiki spec section gives 22- and 30-char examples without a 26-char one). The test cites the brute-force seed (Python `random.Random(seed=1)` over the base58 alphabet). Algorithm correctness for the 26-char class is still confirmed: the same SHA-256 self-checksum + SHA-256 privkey rules apply uniformly to all three length classes; cite-source-or-brute-force is exactly the SPEC-permitted alternative. Future canonical-vector discovery can replace the fixture without code change.
  - **[L2] Brute-force reproducibility is Python-side, not Rust-side.** A reviewer who wants to reproduce the 26-char fixture must run the Python generator (commented in the test). A Rust-side property test that re-mines a 26-char key under a fixed RNG and re-asserts the WIF would be slightly stronger; deferred — the wire-format invariant is already covered by 22- and 30-char canonical vectors.

## Caveats

- **26-char fixture is test-only.** See [L1] above. Privkey hex (`SHA256(key)`) and resulting WIF were computed by an independent Python implementation (`/tmp/wif_encode.py` during fixture generation) and cross-checked against the toolkit's actual decode output during test development. If the Casascius wiki adds a canonical 26-char vector later, replacement is a 2-line change.
- **Compressed flag locked to `false`.** Per SPEC §13 + task spec, Casascius mini-keys decode to uncompressed WIFs. Mainnet uncompressed → `5...` prefix; testnet uncompressed → `9...` prefix. There is no `--compressed` flag for this edge in v0.7; if user demand surfaces, file a v0.8 FOLLOWUP.

## Assessment

**Shippable.** All 10 new tests GREEN; no clippy regressions; no SPEC/text drift. `(MiniKey, Wif)` edge implements §13 exactly.
