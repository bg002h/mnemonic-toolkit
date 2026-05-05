# Phase B — foundation scaffolding Review — r1

**Date:** 2026-05-05
**Commit under review:** `70cbec7` (parent: `b7fe10d`)
**Reviewer:** opus phase-review

## Verdict

0 critical / 0 important / 1 low / 2 nits

✅ **Phase B r1 terminator reached** — cleared to advance to Phase C synthesis.

## Critical / Important

(none)

## Low / Nit

- **L-1 (FIXED inline post-r1):** `template.rs::CliTemplate::TrMultiA` and `TrSortedMultiA` doc-comments said `tr(@0, multi_a(K, @1, ..., @N))` (mis-suggesting `@0` = NUMS internal key, `@1..@N` = signing keys). SPEC §2.1.3 + the actual `wrapper_node` impl emit `tr(multi_a(K, @0, ..., @N-1))` (all signing keys indexed from `@0`). The misleading inline comment about "BIP-86 NUMS internal key" at lines 195-199 was also corrected. No runtime effect (Phase B has no synthesis); fixed inline so Phase C implementer doesn't get confused by the convention.
- **N-1 (no action):** `format::chunk_set_id_extract` is `#[allow(dead_code)]` correctly — it's Phase B scaffolding for Phase C/D consumption.
- **N-2 (no action):** `mode_text::ACCOUNT_INCOMPATIBLE_TEMPLATE` is `#[allow(dead_code)]` with explanatory comment — never fires for v0.2's templates (all have account positions). Reserved for v0.3+ template additions. Correct.

## Verified

- **`wrapper_node(k, n)` correctness for all 10 templates**: spot-checked WshSortedMulti 2-of-3 (`Tag::Wsh + Body::Children([Tag::SortedMulti + Variable{k:2, children:3 PkK}])`), TrSortedMultiA 2-of-2 (`Tag::Tr + Body::Tr{key_index:0, tree:Some(SortedMultiA + Variable)}`), ShWshMulti 1-of-2 (depth-2 nesting `Tag::Sh + Children([Tag::Wsh + Children([Tag::Multi + Variable])])`). All match SPEC §4.6.3.
- **B.1 mini-spike test**: `tr_sortedmulti_a_2_of_2_round_trips_via_md_codec` exists at `template.rs:391`; constructs valid 65-byte xpub with secp256k1 generator G (correct per spike memo Errata 1, NOT `[0x42; 65]`); asserts `is_wallet_policy() == true` after `chunk::split` + `reassemble` round-trip; PASS.
- **`BundleJson.mk1` ownership change**: `&'a [String]` → owned `MkField` (no lifetime parameter). `MkField::Single` byte-identical-serde unit test passes (`["mk1qfoo"]` shape, no `Single` discriminator).
- **Mode-violation pre-checks (7 v0.2 §6.6 rows)**: all 7 present in both `bundle::run` and `verify_bundle::run` with byte-exact text and exit 2. Trigger conditions verified.
- **`is_multisig()` predicate**: `false` for the 4 single-sig variants; `true` for the 6 multisig variants. 8-case test covers all.
- **`parse_cosigner_spec`**: 4 paths covered (`<xpub>:<fp>` 2-part, `<xpub>:<fp>:<path>` 3-part, empty-fp rejected, malformed-xpub rejected).
- **`MultisigPathFamily::default_origin_path`**: BIP-87 mainnet/0 → `m/87'/0'/0'`; BIP-48 testnet/5/2' → `m/48'/1'/5'/2'`. Per SPEC §4.1 / §4.2.
- **`chunk_set_id_extract`**: calls `mk_codec::string_layer::{decode_string, StringLayerHeader::from_5bit_symbols}`; matches `Chunked => Some`, `SingleString => None`, `_ => None` (non_exhaustive fallthrough). Per SPEC §2.2.1 step 1.
- **Stub-error semantics**: multisig dispatch returns `Err(ToolkitError::MultisigConfig { message: "v0.2 multisig synthesis pending Phase C" })`. No panics.
- **No Phase C leakage**: no `synthesize_multisig_*`, no multi-cosigner mk1 emission, no `self_check_bundle` body, no SELF-MULTISIG WARNING stderr. `--self-check` field declared but not acted upon (Phase C wires it).
- **v0.1 wire-bit-identical regression**: 16/16 PASS per implementer report. `synthesize.rs::build_descriptor` calls `wrapper_node(1, 1)` for single-sig; `emit()` wraps with `MkField::Single(bundle.mk1.clone())`; `multisig: None` + `privacy_preserving: false` for single-sig.
- **`schema_version` bump**: `BundleJson.schema_version: "2"` (and `VerifyBundleJson` similarly). `cli_json_envelopes.rs` integration test updated to assert `"2"`.

## Smoke checks

- `cargo test -p mnemonic-toolkit`: 89 passing (72 v0.1 + 17 new Phase B).
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean.
- `cargo fmt --check -p mnemonic-toolkit`: clean.
- v0.1 wire-bit-identical regression: 16/16 PASS.
- Stub-error smoke tests: multisig invocations exit 1 with "v0.2 multisig synthesis pending Phase C"; mode-violation rows fire byte-exact at exit 2.
