# v0.7 Phase 6 — `mnemonic derive-child` self-review

**Status:** Implementation complete. Self-review only; orchestrator dispatches independent reviews after this report lands.
**Predecessor:** Phase 5 (commit c03881b).
**Subcommand:** `mnemonic derive-child`.
**SPEC:** `design/SPEC_derive_child_v0_7.md` (9 sections, 9 test cells in §6).

## Implementation summary

New module `crates/mnemonic-toolkit/src/bip85.rs` (~210 LOC) carries:
- `derive_entropy(master, app_code, app_params, index) -> [u8; 64]` — common BIP-85 §"Specification" primitive: hardened path `m/83696968'/<app>'/<params...>'/<index>'` via `Xpriv::derive_priv`, then `HMAC-SHA512(b"bip-entropy-from-k", child.private_key)`.
- 6 application dispatchers: `format_bip39_phrase`, `format_hd_seed_wif`, `format_xprv_child`, `format_hex_bytes`, `format_password_base64`, `format_password_base85`.
- Hand-rolled `base64_standard` (RFC 4648 alphabet) and `base85_btc` (RFC 1924 / Python `base64.b85encode` alphabet) encoders — neither base64 nor base85 are toolkit deps; encoders are ~25 LOC each.
- 4 inline unit tests: BIP-39 12-word entropy, HEX 64 entropy, PWD BASE64, PWD BASE85, all matching BIP-85 §"Test Vectors" verbatim.

New subcommand at `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (~140 LOC):
- `DeriveChildArgs` clap struct: `--from`, `--application`, `--length` (Optional u32), `--index` (u32), `--network` (reserved), `--language` (reserved).
- `run()` dispatcher: rejects non-xprv `--from`, parses master, dispatches by application string; `rsa`/`rsa-gpg`/`dice` surface `DeriveChildUnsupportedApp` (SPEC §5/§7); per-app `--length` validators emit `DeriveChildLengthOutOfRange`; `hd-seed` and `xprv` reject any supplied `--length` via `DeriveChildLengthNotApplicable`. Always emits the SPEC §4 secret-on-stdout warning on success (all 6 in-scope apps emit secret material).

Wired to clap via:
- `cmd/mod.rs`: `pub mod derive_child;`.
- `main.rs`: `mod bip85;` declaration + new `Command::DeriveChild(...)` variant + dispatch arm.
- `error.rs`: 3 new `DeriveChildRefusal` family variants — `DeriveChildUnsupportedApp(&'static str)`, `DeriveChildLengthOutOfRange { app, length, valid_text }`, `DeriveChildLengthNotApplicable(&'static str)`. All exit 2; `kind()` and `message()` arms wired; messages built in-place to match SPEC §7 byte-exact stderr verbatim.

Tests: `crates/mnemonic-toolkit/tests/cli_derive_child.rs` (9 cells — 6 reference vectors with cell 6 split as 6a + 6b + 3 refusals; 10 test functions).

## SPEC compliance table

| SPEC clause | Test cell | Behavior |
|---|---|---|
| §2 grammar (`--from xprv=`, `--application`, `--length`, `--index`) | implicit (cells 1–6) | clap parses all flags |
| §3 BIP-85 path + HMAC primitive | implicit (cells 1–6) | `derive_entropy` produces spec-pinned 64-byte vector for every application |
| §4 BIP-39 dispatcher (12/15/18/21/24-word) | cells 1, 2 | English-only path component `0'`; entropy bytes = `words * 4 / 3` |
| §4 HD-Seed WIF dispatcher | cell 3 | first 32 bytes → mainnet compressed-pubkey WIF (`Kzyv4uF39...` matches spec) |
| §4 XPRV dispatcher | cell 4 | chain code + privkey reconstructed; depth-0 mainnet xprv matches spec |
| §4 HEX dispatcher | cell 5 | `length`-byte slice of 64-byte entropy hex-encoded |
| §4 PWD BASE64 dispatcher | cell 6a | RFC 4648 base64 of 64 bytes; truncated to `length` chars (`dKLoepugzdVJvdL56ogNV` matches spec for length 21) |
| §4 PWD BASE85 dispatcher | cell 6b | RFC 1924 / Python `b85encode` of 64 bytes; truncated to `length` chars (`_s\`{TW89)i4\`` matches spec for length 12) |
| §4 secret-on-stdout warning emitted | cell 1 | stderr contains `warning: secret material on stdout — consider redirecting ...` |
| §5 / §7 unsupported-app refusal byte-exact | cell 7 | exit 2; stderr verbatim per SPEC §7 |
| §7 `--length` out-of-range refusal byte-exact (bip39) | cell 8 | exit 2; stderr verbatim per SPEC §7 |
| §7 `--length` not-applicable refusal byte-exact (hd-seed) | cell 9 | exit 2; stderr verbatim per SPEC §7 |

## BIP-85 reference-vector citations

All 6 in-scope vectors come verbatim from <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki#test-vectors>:

| Cell | App code | Path | Output |
|---|---|---|---|
| 1 | `39'` (BIP-39) | `m/83696968'/39'/0'/12'/0'` | `girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose` |
| 2 | `39'` (BIP-39) | `m/83696968'/39'/0'/18'/0'` | `near account window bike charge season chef number sketch tomorrow excuse sniff circle vital hockey outdoor supply token` |
| 3 | `2'` (HD-Seed WIF) | `m/83696968'/2'/0'` | `Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp` |
| 4 | `32'` (XPRV) | `m/83696968'/32'/0'` | `xprv9s21ZrQH143K2srSbCSg4m4kLvPMzcWydgmKEnMmoZUurYuBuYG46c6P71UGXMzmriLzCCBvKQWBUv3vPB3m1SATMhp3uEjXHJ42jFg7myX` |
| 5 | `128169'` (HEX) | `m/83696968'/128169'/64'/0'` | `492db4...82a5c` (full 64 bytes) |
| 6a | `707764'` (PWD BASE64) | `m/83696968'/707764'/21'/0'` | `dKLoepugzdVJvdL56ogNV` |
| 6b | `707785'` (PWD BASE85) | `m/83696968'/707785'/12'/0'` | `_s\`{TW89)i4\`` |

All vectors share the spec-provided master xprv `xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb`.

## Hazards encountered + resolutions

1. **BIP-85 path-parameter ordering varies per app:** BIP-39 needs both language (`0'` for English) and word-count between `39'` and the index, while HEX/PWD apps need only `length` between app-code and index, and HD-Seed/XPRV have no extra parameters. **Resolution:** the `derive_entropy` helper takes a `&[u32]` of "app params" inserted between `<app>'` and `<idx>'`; each dispatcher passes the correct shape. Verified against all 6 spec test vectors.

2. **BIP-39 entropy byte count:** Initial implementation used `(2 * words) / 3` (matches the SPEC §3 prose "2 * length_in_words / 3 bytes" but yields 8 / 12 / 16 / 14 / 16 — wrong). The correct formula is `words * 4 / 3` (matching BIP-39 entropy bit count `words * 32 / 3`). Caught at the very first RED→GREEN turn for cells 1+2; fixed in `format_bip39_phrase` with an inline comment. **Filing FOLLOWUP for v0.8 to clarify the SPEC §3 prose** so future implementers don't repeat the off-by-half mistake.

3. **clap deviation from SPEC §5:** SPEC §5 prescribes "clap's enum parser rejects the value at parse time" for out-of-scope `--application rsa|rsa-gpg|dice`. But clap's default error formatter would conflict with the SPEC §7 byte-exact stderr text. **Resolution per orchestrator instruction:** clap accepts the application string raw (not as ValueEnum), and the runtime dispatcher emits the SPEC §7 byte-exact refusal. SPEC §5 wording stays as-is; the deviation is documented here. Verified by cell 7.

4. **`--network` and `--language` flags reserved but unused:** BIP-85 spec test vectors all pin mainnet WIF/xprv and English BIP-39, and v0.7 ships only those. The flags exist in the clap struct (matching SPEC §2 grammar) but are `#[allow(dead_code)]` annotated — testnet emission and non-English BIP-39 wordlists deferred to v0.8 FOLLOWUPS. The flags do NOT panic when supplied; they're silently inert in v0.7.

5. **Hand-rolled base64/base85:** Neither encoder is in the toolkit dep tree. Both are <30 LOC, the SPEC `length` ranges sit safely in the unpadded portion of the encoded output (base64 length ≤ 86 < 88 chars), and base85 input is always 64 bytes (4-aligned, no trailing-padding logic). Verified byte-exact against BIP-85 §"Test Vectors" via cells 6a + 6b. **Resolution:** keeps the dep tree clean; FOLLOWUP filed for v0.8 if additional base-N encoders surface elsewhere (e.g., in the hypothetical RSA application).

## v0.8 FOLLOWUPS to file in Phase 8

1. **`bip85-rsa-rsa-gpg-dice-applications`** — implement BIP-85 apps `828365'` (RSA), `67797633'` (RSA-GPG), `89101'` (DICE). RSA + RSA-GPG require the `rsa` crate (~5 transitive deps); DICE is niche but trivial (`% 6 + 1` reduction over entropy). Gated on user demand signal.

2. **`bip85-passphrase-protected-master`** — BIP-85 spec is silent on whether `--from xprv=` should accept a passphrase-encrypted master (e.g. supplying a phrase + BIP-39 passphrase rather than a pre-derived xprv). v0.7 routes the user through `mnemonic convert --from phrase=... --to xprv` first. Could be smoothed into `derive-child` by accepting `--from phrase=` + `--passphrase` directly.

3. **`bip85-non-english-bip39-language-codes`** — `--language` flag is plumbed but inert; English (code `0'`) is always used. Add Japanese / Korean / Spanish / Chinese (Simplified+Traditional) / French / Italian / Czech / Portuguese language codes per BIP-85 §"BIP39" table. Requires `bip39 = { features = ["all-languages"] }` (already present) — purely a clap-routing change.

4. **`bip85-testnet-emission`** — BIP-85 test vectors all pin mainnet, but the spec doesn't normatively specify which network the WIF/xprv applications emit. v0.7 hardcodes mainnet. Add `--network testnet` support (will need to skip the spec test-vector cells when on testnet).

5. **`bip85-spec-prose-byte-formula-clarification`** — SPEC §3 prose says "2 * length_in_words / 3 bytes (e.g., 12 words → 16 bytes; 24 words → 32 bytes)". The numeric examples (12→16, 24→32) are correct; the formula `2*words/3` evaluates to `8` and `16` respectively. Should be `words * 4 / 3` or written as `words / 3 * 4`. Tracking for SPEC delta in Phase 8.

6. **`bip85-stdin-master-xprv`** — `--from xprv=-` (stdin) is supported via the shared `parse_from_input` parser, BUT the `derive-child` `run()` does not currently read stdin (cf. `convert.rs::run` stdin handling). This is a UX-symmetry follow-up: an scripted pipeline `cat xprv.txt | mnemonic derive-child --from xprv=- ...` will fail today.

## Verification

```fish
# from /scratch/code/shibboleth/mnemonic-toolkit
cargo build --workspace --tests
cargo test --workspace --no-fail-fast
# 442 passed / 0 failed / 2 ignored (was 428 / 0 / 2 baseline; +10 cli + +4 inline = +14)
cargo clippy -p mnemonic-toolkit --tests
# zero net-new warnings on touched files (bip85.rs, derive_child.rs, error.rs, main.rs, cmd/mod.rs)
```

Manual smoke (matches Phase 9 plan):

```fish
mnemonic derive-child --from xprv=xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb --application bip39 --length 12 --index 0
# girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose
# warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')
```

## Files touched

- New: `crates/mnemonic-toolkit/src/bip85.rs` (~210 LOC).
- New: `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (~140 LOC).
- New: `crates/mnemonic-toolkit/tests/cli_derive_child.rs` (~230 LOC).
- Modified: `crates/mnemonic-toolkit/src/cmd/mod.rs` (1 line — `pub mod derive_child;`).
- Modified: `crates/mnemonic-toolkit/src/main.rs` (3 hunks — `mod bip85;`, `Command::DeriveChild(...)` variant, dispatch arm).
- Modified: `crates/mnemonic-toolkit/src/error.rs` (3 hunks — variants, exit-code, kind+message).

## Post-review fixes

Two parallel reviewers (spec-compliance + code-quality) ran after `edaa959` and surfaced 3 Important + 3 Lows + 1 Nit. All addressed in a single follow-up commit on master; net effect on tests is 442 → 443 (1 new integration test cell 9b; no regressions).

### I1 (spec-compliance) — `--length` was `Option<u32>` without `required = true`, violating SPEC §2 grammar-uniformity

`crates/mnemonic-toolkit/src/cmd/derive_child.rs:32-33` declared `pub length: Option<u32>` without `required = true`. SPEC §2 mandates: "All four core flags (`--from`, `--application`, `--length`, `--index`) are MANDATORY"; the impl let users omit `--length` on bip39 (yielding `BadInput` exit 1 instead of clap-layer exit 64). Worse, cells 3 and 4 (hd-seed, xprv) reference-vector tests omitted `--length` entirely, masking the bug.

**Fix applied:** changed `pub length: Option<u32>` → `pub length: u32` with `#[arg(long = "length", required = true)]`. Adopted the SPEC §7 `--length 0` sentinel convention for hd-seed/xprv: clap-required `--length` value is irrelevant for fixed-output apps unless non-zero; non-zero values still trigger `DeriveChildLengthNotApplicable`. The `reject_length(length: u32)` helper now operates on the raw u32 and only fires the refusal when `length != 0`. The `require_length` helper was removed (callers read `args.length` directly). Cells 3 and 4 updated to pass `--length 0`; cells 6a + 6b (already in scope) were already passing concrete values. Cell 9 (hd-seed `--length 32`) refusal assertion preserved; new cell 9b mirrors it for xprv (see L2 below).

### I2 (spec-compliance) — secret-on-stdout warning tested for only 1 of 6 reference-vector cells

SPEC §4 last paragraph mandates the warning fires on every successful invocation. Only cell 1 asserted `stderr.contains("warning: secret material on stdout")`.

**Fix applied:** added the same `assert!(stderr.contains("warning: secret material on stdout"), ...)` to cells 2, 3, 4, 5, 6a, 6b. All 6 reference-vector cells now exercise the §4 invariant.

### I3 (code-quality) — cell 3 doc-comment contradicted actual behavior

`tests/cli_derive_child.rs:75-78` doc-comment promised a `--length 0` sentinel that the test did not pass. After I1 (the test now passes `--length 0`), the doc-comment matches the new behavior; rewritten for clarity to reference SPEC §2 grammar-uniformity + §7 sentinel semantics, with cross-link to cell 9 for the non-zero refusal path.

### L1 (spec-compliance) — refusal tests used `contains` instead of `assert_eq!` byte-exact

SPEC §6/§7 require byte-exact stderr. Cells 7 (rsa), 8 (bip39 length out-of-range), 9 (hd-seed length not-applicable) used `assert!(stderr.contains(...))`.

**Fix applied:** all 3 cells now use `assert_eq!(stderr.trim(), "<spec-byte-exact-text>")`. The `.trim()` keeps the assertion robust against trailing newlines from `clap`'s default error formatter; the `<error: ...>` prefix is included verbatim per SPEC §7.

### L2 (code-quality) — new cell 9b mirroring cell 9 for xprv branch

SPEC §7 not-applicable refusal text reads `<hd-seed|xprv>` but only the hd-seed branch was tested. Added `cell_9b_xprv_length_not_applicable_refusal` invoking `--application xprv --length 32`; asserts byte-exact stderr matching SPEC §7 + exit 2.

### L3 (code-quality) — unused `&'static str` payloads dropped

`ToolkitError::DeriveChildUnsupportedApp(&'static str)` and `DeriveChildLengthNotApplicable(&'static str)` payloads were never read (`message()` arms used the SPEC §7 byte-exact text directly; `details()` had no entries). Variants converted to fieldless: `DeriveChildUnsupportedApp` and `DeriveChildLengthNotApplicable`. Updated all match arms in `error.rs` (2 hunks: exit-code + kind + message) and the 3 constructor sites in `cmd/derive_child.rs` (collapsed `rsa | rsa-gpg | dice` arm into single match, and dropped the `app` argument from `reject_length`). `DeriveChildLengthOutOfRange { app, length, valid_text }` keeps its struct payload (used by `message()` formatter).

### N1 — review report test-count wording corrected

Self-review §"Implementation summary" claim "10 cells — 7 reference vectors + 3 refusals" was inconsistent with §6 of the SPEC (9 cells: 6 reference + 3 refusals; cell 6 split as 6a + 6b → 7 reference-vector test functions, 10 total). Reworded to "9 cells — 6 reference vectors with cell 6 split as 6a + 6b + 3 refusals; 10 test functions."

### Verification post-fix

```fish
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --workspace --tests   # GREEN
cargo test --workspace --no-fail-fast   # 443 passed / 0 failed / 2 ignored
                                          # (= 442 baseline + 1 new cell 9b)
cargo clippy -p mnemonic-toolkit --tests   # pre-existing warnings unchanged;
                                            # 0 net-new on touched files
                                            # (cmd/derive_child.rs / error.rs / cli_derive_child.rs)
```

### Phase 8 SPEC delta to file

- The §2 + §7 grammar-uniformity text "supplying any value emits the refusal" had internal tension with §6 cells 3 + 4 reference-vector cells (which would be unreachable under that strict reading). The implementation adopts a sentinel-0 convention: `--length` is required at clap (§2 grammar-uniformity), the value is ignored for hd-seed/xprv when `0`, and any non-zero value triggers the §7 not-applicable refusal. SPEC text edits deferred to Phase 8 (§2 / §7 sentinel-0 wording clarification).
