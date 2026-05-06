# Changelog

All notable changes to `mnemonic-toolkit` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## mnemonic-toolkit [0.3.0] — 2026-05-05

### What's new

- **`--descriptor "<string>"` and `--descriptor-file <path>`** flags accept any BIP-388 descriptor whose miniscript AST is supported by the v0.3 walker. Toolkit synthesizes md1 + mk1 + ms1 bundles for any combination of full / watch-only × single-sig / multisig modes detected from the descriptor's `@N` placeholder count (n=1 → single-sig regardless of outer wrapper; n≥2 → multisig).
- **Walker covers the BIP-388 surface:** all v0.2 wrappers (`wpkh`, `pkh`, `wsh+(Ms|SortedMulti)`, `sh+(Wpkh|Wsh|Ms|SortedMulti)`, `tr` keypath + single-leaf miniscript), plus 23 v0.3-NEW miniscript fragments — hash terminals (`sha256`, `hash256`, `hash160`, `ripemd160`), timelocks (`after`, `older`), wrappers (`v:`, `s:`, `a:`, `j:`, `n:`, `c:`), boolean ops (`and_v`, `and_b`, `andor`, `or_b`, `or_c`, `or_d`, `or_i`), and `thresh()`.
- **`@N[fp/path]/<multipath>/*` annotation syntax.** Full-mode `@0` requires the `[fp/path]` annotation; toolkit derives the xpub at the annotated path and cross-checks the fingerprint against the seed-derived master fp. Multi-cosigner `@N≥1` annotations are cross-checked against `--cosigner` triples.
- **`verify-bundle --descriptor`** mirror of the bundle path. Re-runs the descriptor pipeline, builds the expected ms1/mk1/md1, and compares byte-equality to the supplied cards. New `DescriptorReparseFailed` error variant (exit 4) for re-parse failures.
- **`SELF-MULTISIG WARNING`** detection extended to descriptor mode (fires when full-mode multisig descriptor has any cosigner xpub equal to the seed-derived `@0` xpub).
- **Bundle JSON schema bumped to `"3"`.** `template` field becomes nullable; new top-level `descriptor` field carries the user-supplied descriptor verbatim. Both fields ALWAYS emit (`null` when not set).

### Breaking changes (callers)

- `BundleArgs::template`: `CliTemplate` → `Option<CliTemplate>`. Clap attr `required_unless_present_any = ["descriptor", "descriptor_file"]`. Same change applied to `VerifyBundleArgs::template`.
- `BundleJson::template`: `&'static str` → `Option<&'static str>`. New `descriptor: Option<String>` field.
- `VerifyBundleJson::schema_version` and `BundleJson::schema_version`: `"2"` → `"3"`.

### Wire-bit-identical guarantee

Encoded card strings (ms1 / mk1 / md1) for any v0.2 invocation under the v0.3 binary remain byte-identical. Only the JSON envelope differs: `schema_version "2"→"3"` and a new `"descriptor": null` field appears. The v0.2 fixture corpus is preserved verbatim and continues to validate.

For descriptor-mode invocations that exactly express a v0.2 template (canonical `[fp/path]` annotation matching the BIP-44/49/84/86 paths), the resulting md1 is byte-identical to template-mode emission. Three regression tests confirm this for bip44 / bip84 / bip86 (`descriptor_bipXX_matches_template_bipXX_md1` in `parse_descriptor::tests`).

### Out of scope (deferred to v0.4)

- `tr(@0, sortedmulti_a(...))` — rust-miniscript v13.0.0 has no parser for `sortedmulti_a` in tap-leaves. Tracked in `design/FOLLOWUPS.md` (`tr-sortedmulti-a-via-upstream`); v0.4 gates on upstream parser support.
- Multi-leaf taproot trees (`tr(K, {A,B})` with N≥2 leaves). Deferred per SPEC §6.8 (Merkle-root logic).
- Engraving card in descriptor mode. Existing card builder is template-coupled; v0.4 will add a descriptor-aware card. Tracked in FOLLOWUPS (`descriptor-mode-engraving-card`).
- Full v0.4-style 9 / 3+6N descriptor-aware verify-bundle check ladder. v0.3 ships a 3-element direct-byte-equality ladder (ms1_match, mk1_match, md1_match). Functional but coarser than template-mode's 9-check schema.
- `RawPkH` and `DupIf` `Terminal` arms — descriptor-unreachable in rust-miniscript v13.0.0 (RawPkH only via raw script decode; DupIf type-restrictive). Walker handles them for completeness; tests `#[ignore]`.

### Test corpus

156 unit tests + 9 v0.3 mode-violation integration tests + all v0.2 integration tests (cli_bundle_*, cli_verify_bundle_*, cli_mode_violations_v0_2, cli_json_envelopes, etc.) green; v0.2 fixture matrix continues to pass byte-identically.

### Reproduction

Build: `cargo build --release`. Test: `cargo test --package mnemonic-toolkit`.

The v0.3 SPEC at `design/SPEC_mnemonic_toolkit_v0_3.md` (rounds 1-7, architect-reviewed 0C/0I) is normative for all descriptor-mode behavior. The implementation plan at `design/IMPLEMENTATION_PLAN_v0_3_descriptor_passthrough.md` records phase-by-phase architect-review verdicts (mid-phase + end-of-phase per phase, all addressed to 0C/0I).

---

## mnemonic-toolkit [0.2.0] — 2026-05-05

### What's new

- **Multisig templates (6 BIP-388 wrappers):** `wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`. Threshold `1 ≤ K ≤ N ≤ 16`.
- **`--account <u32>`:** non-zero account index threading; replaces v0.1's hardcoded `account=0`.
- **`--xpub-input` multisig (watch-only):** `--cosigner <xpub>:<fp>:<path>` (repeatable) + `--cosigners-file <path>` for bulk JSON ingestion. Per-cosigner path overrides supported; `--multisig-path-family {bip48,bip87}` selects the global default (default `bip87`).
- **`--privacy-preserving`:** whole-bundle privacy boolean. Suppresses `master_fingerprint` from mk1 origins (multisig only); single-sig watch-only with `--xpub` rejects the flag (would produce inconsistent bundle vs. md1's `tlv.fingerprints`).
- **`--self-check`:** post-emit synthesize-then-verify pass on the bundle just produced. Catches synthesis/verify drift before the user engraves.

### Wire-bit-identical guarantee

Encoded card strings (ms1 / mk1 / md1) are byte-identical to v0.1's output for any v0.1-equivalent invocation (single-sig, account=0, no `--privacy-preserving`, no `--self-check`). v0.1 decoders consuming v0.2-emitted encoded strings work unchanged. The 16-cell v0.1 fixture corpus at `tests/vectors/v0_1/` is preserved verbatim and gated by `cli_bundle_full.rs` as a regression set; SHA-256 pin `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` continues to hold for that subdirectory.

### JSON envelope evolution

- `schema_version` bumps `"1"` → `"2"`.
- New `bundle` fields: `multisig` (discriminated-union: `null` for single-sig; `{ k, n, template, path_family, cosigners: [...] }` for multisig), `privacy_preserving` (bool), `origin_paths` (per-cosigner path list when divergent from family default).
- `mk1` field becomes a `oneOf` shape: flat object for single-sig, array of N grouped chunk-set objects for multisig.

### v0.1 SHA pin retired; v0.2 SHA pin

The v0.1 fixture pin (`81828299...`) is retired as the active regression baseline (it remains as the `tests/vectors/v0_1/` byte-identity check). The v0.2 corpus adds 34 new multisig + axis cells under `tests/vectors/v0_2/`. Reproduction command (resolves v0.1 FOLLOWUPS N-1, the missing SHA-reproduction recipe):

```bash
shasum -a 256 crates/mnemonic-toolkit/tests/vectors/v0_2/*.txt | sort | shasum -a 256
# a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171
```

### Tests

76 unit + 31 integration test functions = 107 total (`cargo test --workspace`). The 31 integration functions cover ~54 parametric cells across 13 test binaries. New v0.2 integration tests:
- `cli_bundle_multisig_full.rs` — 24-cell multisig fixture parametric (6 templates × 4 networks).
- `cli_account_flag.rs` — 4-cell `--account 5` parametric.
- `cli_privacy_preserving.rs` — 4-cell `--privacy-preserving` parametric.
- `cli_self_check.rs` — 2 happy-path self-check fixtures (single-sig + multisig).
- `cli_mode_violations_v0_2.rs` — 7 v0.2 NEW SPEC §6.6 mode-violation rows (byte-exact text + exit-2 contract).

### Known limitations (v0.3+ deferred)

- K-of-N share encoding (split mk1 / split ms1 / split md1) deferred — ms1 first per BIP-93.
- `--cosigners-file` user-supplied file output / multi-file output deferred.
- Hash-locks / timelocks / advanced descriptor variants deferred.
- `cargo publish` of the toolkit still gated on `ms-codec` / `mk-codec` / `md-codec` reaching crates.io. v0.2.0 distributed via GitHub tag `mnemonic-toolkit-v0.2.0`.

### Wire-format SHA pin

```text
sha256(crates/mnemonic-toolkit/tests/vectors/v0_2/) = a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171
```

## mnemonic-toolkit [0.1.0] — 2026-05-04

### What's new

- Initial release. Top-level integration crate of the m-format star.
- 2 subcommands: `bundle` (encode-side: emit 3-card engraving bundle) and `verify-bundle` (round-trip integrity check).
- 2 input modes per command: full (`--phrase`) and watch-only / key-only (`--xpub --master-fingerprint`).
- 4 single-sig wallet templates: BIP-44 (pkh), BIP-49 (sh-wpkh), BIP-84 (wpkh), BIP-86 (tr).
- 4 networks: mainnet / testnet / signet / regtest.
- Account hardcoded `0` in v0.1; `--account` flag deferred to v0.2.
- All 10 BIP-39 wordlists supported via `--language`.
- Multi-section stdout (`# ms1` / `# mk1` / `# md1` headers + chunked engraving form).
- Byte-exact engraving-card stderr per SPEC §5.2.
- `--json` envelope schemas for both subcommands.
- Exit codes 0 / 1 / 2 / 3 / 4 / 64 per SPEC §6.
- Byte-deterministic mk1 `chunk_set_id` derived from the 4-byte `policy_id_stub` (mirrors md-codec's deterministic CSI derivation), so toolkit output is byte-reproducible across runs and the SHA-pinned regression corpus is meaningful.

### Tests

17 integration tests (assert_cmd) + 54 unit tests. Trezor 24-word zero-entropy vector pinned across 16 (template × network) cells.

### Known limitations

- Multisig templates, non-zero account, file output, recovery flow: deferred to v0.2+.
- `cargo publish` blocked until ms-codec / mk-codec / md-codec hit crates.io. v0.1.0 distributed via GitHub tag `mnemonic-toolkit-v0.1.0`.

### Wire-format SHA pin

The 16 fixture files at `crates/mnemonic-toolkit/tests/vectors/v0_1/*.txt` are SHA-256-pinned at this release. Subsequent corpus changes that alter the SHA require a SemVer minor bump per the pre-1.0 breaking-change-axis convention.

```text
sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/) = 81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6
```
