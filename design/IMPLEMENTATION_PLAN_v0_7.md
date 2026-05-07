# mnemonic-toolkit v0.7 — implementation plan

**Status:** CONVERGED (3 user-rounds + 2 architect-rounds; R2 returned 0C/0I/3L all addressed in-plan; ready for execution).

## Context

mnemonic-toolkit v0.6.2 SHIPPED 2026-05-06. This cycle will produce tag `mnemonic-toolkit-v0.7.0`; predecessor tag `mnemonic-toolkit-v0.6.2` sits at `1fddf3b`. The post-release wallet-types audit cataloged 8 v0.7-tier candidates; the user locked v0.7 scope through 3 brainstorm rounds:

1. Round 1 (slip39-vs-ms1-shares gate): user originally chose "ship SLIP-39 in v0.7 as major addition."
2. Round 2 (cycle shape): user originally chose "SLIP-39 + wallet-export + address-derivation."
3. Round 3 (post-lib-audit revision): SLIP-39 lib audit returned **hand-roll required** (no maintained Rust crate; closest candidate `sssmc39` is "untested WIP", license-incompatible alternatives elsewhere). User reshaped: **drop SLIP-39, do all other key formats**, and **add** them to the prior xpub-artifact pair.

**v0.7 final scope (6 features + 2 carry-overs):**

| # | Feature | Tier | Approach | Est. LOC |
|---|---|---|---|---|
| 1 | `bip38-encrypted-wif` | v0.7 | Use `bip38 v1.1.1` crate (Apache-2.0, GREEN maintenance) | ~80 |
| 2 | `casascius-mini-private-key` | v0.7 | Hand-roll (SHA256 self-checksum; tiny) | ~80 |
| 3 | `electrum-native-seed-format` | v0.7 | Hand-roll (own wordlist + HMAC-SHA512 version-prefix) | ~300 |
| 4 | `bip85-deterministic-entropy` | v0.7 | Hand-roll subcommand (HMAC-SHA512 + 6 data-derivation app dispatchers; `bip85` crate v0.1.1 is RED-unmaintained; RSA + RSA-GPG out-of-scope per app-coverage decision) | ~200 |
| 5 | `address-derivation-from-xpub-path` | v0.7 (was v0.6.2-deferred) | Reuse `bitcoin v0.32` crate's `Xpub::derive_pub` + `Address::*` constructors | ~120 |
| 6 | `wallet-export-industry-formats` | v0.6.2 (deferred) | New subcommand `mnemonic export-wallet`; Bitcoin Core `importdescriptors` (MVP) + BIP-388 `wallet_policy`; Sparrow/Specter optional | ~500 |
| 7 | `slip0132-info-line-spec-text-not-byte-pinned` | v0.7-nice-to-have (carry-over) | SPEC-text pinning test (small) | ~20 |
| 8 | `verify-bundle-discards-slip0132-input-variant-asymmetry` | v0.7-nice-to-have (carry-over) | UX policy decision: surface or document the asymmetry | ~10–80 |

**Deferred to v0.8 (or wont-fix):**
- `slip39-shamir-secret-sharing` — hand-roll-required; user direction is to defer to a focused v0.8 cycle OR close as wont-fix in favor of ms1-shares (the family's v0.2 share-encoding mechanism via codex32). Decision deferred to v0.7 cycle close-out brainstorm.
- `miniscript-beyond-bip388` — open-ended scope; v0.8 brainstorm.
- `vault-construction-covenant-based` — gated on Bitcoin consensus activation.

**SPEC cross-links:**

- Convert SPEC delta — `design/SPEC_convert_v0_6.md` (v0.7 amendments §1, §2, §3, §10, §11, §12, §13, §14).
- New subcommand SPECs — `design/SPEC_export_wallet_v0_7.md` and `design/SPEC_derive_child_v0_7.md`.

**Why v0.7 is large but justified:** The user explicitly accepted the largest-cycle option after the lib audit; the bip38-crate path is cheap; address-derivation reuses existing deps; Electrum + BIP-85 are bounded hand-rolls (no group/member 2-level scheme like SLIP-39 had). Net add: **+1 external crate (`bip38`)**; toolkit LOC growth ~8% (~7,900 → ~8,540).

## Spike result (lib audit, 2026-05-06)

Two Explore-agent passes verified:

- **`bip38 v1.1.1`** — crates.io. Apache-2.0. Last update May 2024. Provides `Encrypt`/`EncryptWif`/`Decrypt`/`Generate` traits with correct Scrypt parameters (n=16384, r=8, p=8) per BIP-38 spec for non-EC-multiplied form. NFC passphrase normalization built in. 2 PRs pending; 0 issues; 10K total downloads. **GREEN, use as dep.**
- **`bip85 v0.1.1`** — crates.io. MIT. Last commit Apr 2021 (4+ years). 17 commits total; 2 unresolved issues; zero crates.io releases. **RED, hand-roll.**
- **No `electrum-mnemonic` / `electrum-seed`** — query 404. Hand-roll against Electrum's own Python reference (`spesmilo/electrum/electrum/mnemonic.py`).
- **`bitcoin v0.32`** (already in tree) — provides `Xpub::derive_pub`, `Address::p2wpkh/p2shwpkh/p2tr` constructors. **REUSE.**
- **No new crypto primitives required** — `bitcoin_hashes` (already in tree via `bip39`/`bitcoin`) provides HMAC-SHA512 for Electrum + BIP-85.

## Edge / NodeType matrix

New convert edges (extend `is_supported_direct_edge`):

| from | to | shape | refusal class if reverse |
|---|---|---|---|
| Wif | Bip38 | encrypt with `--passphrase` | reverse exists: `(Bip38, Wif)` decrypt |
| Bip38 | Wif | decrypt with `--passphrase` | — |
| MiniKey | Wif | decode (SHA256 self-checksum + raw → privkey) | one-way (typo-checksum unrecoverable) |
| Phrase | ElectrumPhrase | sibling-pivot **REFUSE** | use bundle for cross-format pivot |
| ElectrumPhrase | Entropy | decode (own wordlist + HMAC-SHA512 verify) | reverse exists: `(Entropy, ElectrumPhrase)` if seed-version 01/100 |
| Entropy | ElectrumPhrase | encode (forward) | — |
| Xpub | Address | derive at `--path` + `--script-type` | one-way (address is a hash) |

Plus composite edges via existing intermediates flow naturally (e.g., `phrase → bip38` via `phrase → wif → bip38`).

**Refusal table completeness (per architect R1-L11):** The new NodeTypes interact with existing edges through the catch-all `refusal_one_way` path. SPEC delta in `SPEC_convert_v0_6.md` §3 must explicitly enumerate the new "obvious" refusal pairs so the code-reviewer can verify coverage:

| from | to | refusal class |
|---|---|---|
| `Bip38` | `ElectrumPhrase`, `Address`, `Mk1`, `Ms1`, etc. | one-way / sibling-pivot via `Wif` intermediate |
| `MiniKey` | anything except `Wif` | sibling-pivot via `Wif` intermediate |
| `Bip38` | `Bip38` | identity-pivot refusal |
| `ElectrumPhrase` | `ElectrumPhrase` | identity-pivot refusal |
| `Address` | `*` | one-way (address = hash) |
| `*` | `MiniKey` | one-way (mini-key requires brute-force search for typo-checksum) |
| `*` | `Address` (without `--path`) | missing-arg refusal (`refusal_address_no_path`) |
| `2FA Electrum phrase` | `Entropy` | `refusal_electrum_2fa_unsupported` |

## Implementation plan

**TDD discipline:** Each phase begins with RED tests. Per-phase code-reviewer round persists to `design/agent-reports/v0_7-phase-N-review.md`. Brainstorm/SPEC/plan reviews stay in transcript per memory `feedback_iterative_review_every_phase.md`.

### Phase 0 — Compile-time scaffold (minimal, no behavior tests)

Per architect R1 finding **C1**: Phase 0 does NOT ship a unified stub dump with 25 deliberate-failure RED tests. Stub semantics broke RED→GREEN discipline (tests asserting `"not yet implemented"` stderr would flip GREEN twice — once at stub-ship, again at real-impl). Each feature phase (1–6) is now atomic: opens with its own NodeType + edge + RED tests against the **real expected output**, then ships impl in the same phase commit.

Phase 0's scope is the bare-minimum compile-time scaffold:

- `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType` — add `Bip38`, `MiniKey`, `ElectrumPhrase`, `Address` variants. Update `as_str()` and `from_token()` mirrors. **Lock `from_token` strings (per architect R1-I7):** `"bip38" → Bip38`, `"minikey" → MiniKey` (no hyphen, consistent with `mk1`/`ms1`), `"electrum-phrase" → ElectrumPhrase` (hyphen acceptable; multi-word with no ambiguity), `"address" → Address`.
- Update the `parse_from_input` error-message literal at `convert.rs:103-105` to enumerate all new tokens.
- Update `NodeType::is_secret_bearing` (per architect R1-L12): add `Bip38` to the secret-bearing arm. Bip38 is an encrypted privkey; downstream `secret-on-stdout` warning depends on this.
- `compute_outputs` — add `match` arms for each new variant returning `unreachable!("Phase N implements this — Phase 0 scaffold only")` (NOT `Err(BadInput(...))`). The `unreachable!` arms are guaranteed to never fire because Phase 0 does NOT add edges to `is_supported_direct_edge`; convert won't dispatch to them. They exist purely for compile-time exhaustive-match on `NodeType`.
- **Zero new tests in Phase 0.** No RED tests assert the scaffold; tests live in Phase 1–6 against real expected outputs.

**Phase 0 exit gate:** `cargo build --workspace --tests` + `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` ALL GREEN. No behavior change visible to users; new variants exist but are unreachable.

### Phase 1 — BIP-38 encrypt/decrypt (use `bip38` crate, with security review)

Per architect R1-I4: adding a single-author crypto crate to a backup tool requires a focused source-level review. Phase 1 task list:

1. **Source review of `bip38 v1.1.1`** (~500 LOC). Read the full crate; confirm:
   - Scrypt params hardcoded to `n=16384, r=8, p=8` per BIP-38 spec for non-EC-multiplied form.
   - NFC passphrase normalization is applied at the encrypt + decrypt entry points.
   - EC-multiplied-form inputs (intermediate codes) are either correctly handled OR rejected with a clean error (NOT silently mis-processed).
   - No surprising deps (the crate should pull only `bs58`, `aes`, `scrypt`, `sha2`, `secp256k1`, `bitcoin_hashes` or similar standard primitives).
   - Output: `design/agent-reports/v0_7-phase-1-bip38-security-review.md` with the verdict (use as planned / use with caveats / hand-roll required).
2. **Integration:** Add `bip38 = "1.1"` to `crates/mnemonic-toolkit/Cargo.toml`. Implement `(Wif, Bip38)` and `(Bip38, Wif)` arms in `compute_outputs` via the crate's traits.
3. **Error mapping:** `bip38::Error::Pass` → `ToolkitError::PassphraseMismatch` (add the variant if not present; otherwise reuse).
4. **TDD:** Phase 1 RED integration tests for `(Wif, Bip38)` / `(Bip38, Wif)` (one per direction, plus refusal tests for `Bip38 → Bip38` and one-way edges). Round-trip via BIP-38 spec's 3 test vectors (cite spec §"Test vectors" in test comments).

**Phase 1 exit gate:** security review checked in; bip38 edges GREEN; `cargo clippy` clean.

### Phase 2 — Casascius mini-key decode

- Hand-roll: SHA256 self-checksum rule (`SHA256(mini_key + "?")[0] == 0x00`); decode privkey via `SHA256(mini_key)`.
- `compute_outputs` — implement `(MiniKey, Wif)` arm.
- One-way: no `(Wif, MiniKey)` edge.
- Phase 0 RED test for `(MiniKey, Wif)` turns GREEN.
- Reference vectors: 22-char + 26-char + 30-char Casascius keys from public sources (cite source URLs in test comments).

**Phase 2 exit gate:** Casascius edge GREEN; coverage of all 3 length classes.

### Phase 3 — Electrum seed format (hand-roll, with 2FA refuse + corpus spike)

Per architect R1-I5: 2FA seed versions (`101` standard, `102` segwit) require a second factor not present in the phrase alone; silently decoding them produces garbage entropy. Phase 3 must explicitly REFUSE those, not parse-fail. Plus a wire-format spike against Electrum's own test corpus before locking encode direction (matches the SLIP-0132 spike discipline from v0.6.2).

1. **Spike (read-only, Phase 3 prerequisite):** Pull 4–6 Electrum seed phrases (one per seed-version: `01`, `100`, `101`, `102`) from Electrum's own `tests/test_wallet.py` or equivalent test corpus. Verify HMAC-SHA512 prefix predictions match. Document spike result in `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`. Lock encode/decode direction only after spike confirms.
2. **New module `crates/mnemonic-toolkit/src/electrum.rs`:**
   - Embedded English wordlist (constant array, ~2KB). Source: Electrum's `electrum/mnemonic.py::ElectrumMnemonicEnglish` wordlist (cite SHA + retrieval date inline).
   - `enum SeedVersion { Standard, Segwit, Standard2FA, Segwit2FA }`.
   - `validate_seed_version(phrase) -> Result<SeedVersion, ElectrumError>` via HMAC-SHA512(`"Seed version"` || phrase) hex-prefix dispatch.
   - `phrase_to_entropy(phrase, version) -> Result<Vec<u8>, _>`.
   - `entropy_to_phrase(entropy, version) -> Result<String, _>`.
3. **Refusal helpers added:**
   - `refusal_electrum_2fa_unsupported(version)` → "2FA seed (`101`/`102`) requires a second factor not present in the phrase alone; conversion not supported. Use Electrum directly for 2FA recovery." Both 2FA SeedVersion variants map to this.
   - `refusal_electrum_phrase_pivot()` → existing sibling-pivot taxonomy for `Phrase ↔ ElectrumPhrase`.
4. **`compute_outputs`:** implement `(ElectrumPhrase, Entropy)` and `(Entropy, ElectrumPhrase)` arms (default version = `Standard` for encode; `--electrum-version segwit|standard` flag to override).
5. **TDD:** Phase 3 RED tests cover (a) Standard + Segwit round-trip; (b) 2FA refusal stderr exact-text-pin for both `101` and `102`; (c) Standard ↔ Phrase sibling-pivot refusal; (d) reference-vector decode for at least one Electrum corpus phrase (cite source).

**Phase 3 exit gate:** Electrum Standard + Segwit edges GREEN; 2FA refusal pinned; corpus spike committed; HMAC-SHA512 version-prefix verified.

### Phase 4 — Address derivation (`(Xpub, Address)`)

Per architect R1-I6: input shape locked to **`--path` only** (mandatory, no default). The `--chain receive|change` + `--address-index N` shorthand is deferred to v0.8 FOLLOWUPS as a UX polish item — `--path` is already in `ConvertArgs` and rest of convert uses it; minimum-surprise shape.

1. **`compute_outputs::(Xpub, Address)` arm:** call `bitcoin::bip32::Xpub::derive_pub(secp, &path)` to get child xpub; convert to compressed pubkey; dispatch to `Address::p2wpkh/p2shwpkh/p2tr` per `--script-type`. Network from `--network` (or inferred from xpub prefix).
2. **Args:**
   - `--path <BIP32-path>` — MANDATORY for `(Xpub, Address)`. Refusal `refusal_address_no_path()` if missing: "`--to address` requires `--path` (xpub does not carry an origin path; supply BIP-32 derivation explicitly)."
   - `--script-type <p2wpkh|p2sh-p2wpkh|p2tr>` — explicit. If `--template` is supplied, infer (e.g., `wpkh` → p2wpkh, `sh-wpkh` → p2sh-p2wpkh, `tr` → p2tr); explicit `--script-type` overrides.
3. **Composite edges** from `phrase` / `entropy` via existing BIP-32 derivation pipeline (same as `Xpub` flows).
4. **Refusal:** `Address → *` (one-way; addresses are hashes; can't recover xpub or anything else).
5. **TDD:** Phase 4 RED tests cover (a) BIP-84 reference key (`m/84'/0'/0'`) + path `m/0/0` produces the canonical bech32 address from BIP-84 §"Test vectors"; (b) at least one P2SH-P2WPKH (BIP-49) reference; (c) at least one P2TR (BIP-86) reference; (d) refusal text for `--to address` without `--path`.

**Phase 4 exit gate:** Address edge GREEN for P2WPKH + P2SH-P2WPKH + P2TR; reference vectors pinned; missing-path refusal byte-exact.

### Phase 5 — `mnemonic export-wallet` subcommand

Per architect R1-C2: descriptor-checksum is NOT a public toolkit function. The export pipeline must construct a `miniscript::Descriptor<DescriptorPublicKey>` from template + slot xpubs and rely on its `Display` impl (which auto-appends the `#abcdef12` checksum suffix when the descriptor is well-formed). Per R1-I9: target Bitcoin Core 24+; lock `range` and `timestamp` defaults with override flags.

1. **Descriptor-construction pipeline (locked, per architect R1-C2):**
   - Step (a): parse template (e.g. `wpkh`, `sh-wpkh`, `wsh-sortedmulti`, `tr`) + slot inputs into a `miniscript::Descriptor<DescriptorPublicKey>` via the same path the existing `bundle` codepath uses. Reuse the `parse_descriptor` module if applicable (`crates/mnemonic-toolkit/src/parse_descriptor.rs`).
   - Step (b): call `descriptor.to_string()` — this emits the canonical form with `#checksum` suffix automatically.
   - Step (c): serialize per output format.
   - Document this 3-step pipeline in `design/SPEC_export_wallet_v0_7.md` §"Descriptor pipeline."
2. **New subcommand at `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`** with args:
   - `--format <bitcoin-core|bip388|sparrow|specter>` (default: `bitcoin-core`).
   - `--output <path|->` (default: `-` stdout).
   - `--slot @N.<subkey>=<value>` (parser reused from the `slot_input` module — `crate::slot_input::parse_slot_input`; same shared parser the `bundle` subcommand uses; watch-only-only — see §Refusal).
   - **`--range <start,end>`** (default: `0,999`). Overrides the Bitcoin Core `range` field. Per R1-I9 hazard note: a user with > 999 addresses on the receive chain has invisible funds; the override flag closes that gap.
   - **`--timestamp <unix|now>`** (default: `now`). Per R1-I9: triggers full rescan vs blocks-since-N.
   - **`--bitcoin-core-version <24|25>`** (default: `25`). Minor JSON shape differences between versions — locked at major version 24 minimum; default 25.
3. **Refusal class:** any `phrase=` / `entropy=` / `xprv=` / `wif=` slot supplied → `refusal_export_wallet_secret_input()`: "`mnemonic export-wallet` is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret material, use `mnemonic bundle`." Implemented as a slot-set validator extension.
4. **New module `crates/mnemonic-toolkit/src/wallet_export.rs`:**
   - `format_bitcoin_core_importdescriptors(descriptor, args) -> serde_json::Value` — emits `[{"desc": "...<auto-checksum>", "active": true, "internal": false, "range": [start, end], "timestamp": <"now"|unix>}]`. For multi-path descriptors (e.g., `<0;1>` syntax), splits into 2 entries: receive (`internal: false`) + change (`internal: true`).
   - `format_bip388_wallet_policy(descriptor, args) -> serde_json::Value` — emits BIP-388 `wallet_policy` JSON with `name`, `description_template`, `keys_info` (per BIP-388 §"Wallet policy descriptors").
   - Sparrow / Specter / HWI formatters: stub (return `ToolkitError::NotSupported("Sparrow format deferred to v0.8 if demand surfaces")`).
5. **Reuse:** existing `bundle::resolve_slots` (post-v0.6.2; pub(crate)) handles slot-resolution. The export-wallet subcommand calls `resolve_slots` with the watch-only validator extension.
6. **TDD:** Phase 5 RED tests cover (a) Bitcoin Core importdescriptors round-trip with single-sig wpkh; (b) BIP-388 wallet_policy round-trip with multisig wsh-sortedmulti; (c) refusal stderr for `phrase=` slot input; (d) Sparrow/Specter stub refusal stderr; (e) `--range 0,4999` override exercised; (f) `--bitcoin-core-version 24` shape diff (if version 24 differs from 25 materially — confirm during impl).

**Phase 5 exit gate:** Bitcoin Core + BIP-388 formats functional; descriptor pipeline reuses miniscript Display for canonical+checksum form; Sparrow/Specter stubs return clean refusal; `--range` and `--timestamp` overrides functional.

### Phase 6 — `mnemonic derive-child` subcommand (BIP-85, all 6 data-derivation applications)

Per architect R1-C3 + user direction "all BIP-85 applications defined in the standard": v0.7 implements **all 6 data-derivation applications** from BIP-85's main "Applications" section. RSA + RSA-GPG (apps `828365'` + `67797633'`) require an `rsa` crate not currently in tree; **explicitly out-of-scope for v0.7** — documented in `design/SPEC_derive_child_v0_7.md` §"Application scope (out-of-v0.7)" with a v0.8+ FOLLOWUPS-tier deferral.

In-scope applications (path `m/83696968'/<app>'/<idx>'`, derived via `bitcoin::bip32::Xpriv::derive_priv`, HMAC-SHA512 with key `"bip-entropy-from-k"`):

| App code | Name | Output format |
|---|---|---|
| `39'` | BIP-39 mnemonic | N-word phrase (`--length 12|15|18|21|24` words) |
| `2'` | HD-Seed WIF | WIF-encoded 64-byte master HD seed |
| `32'` | XPRV | Child xprv (Base58Check) |
| `128169'` | HEX | N raw hex bytes (`--length 16..=64`) |
| `707764'` | PWD BASE64 | Base64-encoded password (`--length 20..=86` chars) |
| `707785'` | PWD BASE85 | Base85-encoded password (`--length 10..=80` chars) |

Out-of-scope for v0.7 (defer to v0.8 FOLLOWUPS):
- `828365'` RSA — requires `rsa` crate not currently in tree.
- `67797633'` RSA-GPG — same.
- `89101'` DICE — niche application (deterministic dice rolls); marginal value for a key/wallet tool; defer pending user demand.

1. **New subcommand at `crates/mnemonic-toolkit/src/cmd/derive_child.rs`** with args:
   - `--from xprv=<master>` (mandatory).
   - `--application <bip39|hd-seed|xprv|hex|password-base64|password-base85>` (mandatory).
   - `--length <N>` (mandatory; range varies per application — see table; per-app validator).
   - `--index <N>` (mandatory; non-negative, hardened-derivation-safe).
2. **New module `crates/mnemonic-toolkit/src/bip85.rs`:**
   - 6 application dispatchers (one fn per app).
   - Common helper: `derive_entropy(master, app_code, app_params, index) -> [u8; 64]` for the HMAC-SHA512 step.
   - Per-app output formatter (e.g., `format_bip39_phrase(entropy, length)`, `format_xprv_child(entropy, network)`).
3. **TDD:** Phase 6 RED tests cover (a) one BIP-85 §"Test Vectors" reference per application × at least 1 vector each (BIP-85 spec includes test vectors for all 6 in-scope apps); (b) refusal for unsupported `--application rsa|rsa-gpg|dice` (byte-exact refusal stderr text per SPEC_derive_child_v0_7.md §5 (covers rsa, rsa-gpg, AND dice)).

**Phase 6 exit gate:** All 6 in-scope BIP-85 applications functional; reference vectors pinned per app; RSA/RSA-GPG refusal byte-exact.

### Phase 7 — Carry-overs

- **`slip0132-info-line-spec-text-not-byte-pinned`:** Add a doc-test or unit test to `crates/mnemonic-toolkit/src/slip0132.rs` that reads the canonical info-line from `design/SPEC_convert_v0_6.md` §11 (via `include_str!` of a fenced-text-block extraction) and asserts byte-equality with `render_slip0132_info_line(variant)` output. Closes the SPEC↔production drift hazard surfaced in v0.6.2 final review. The fenced block is delimited by `<!-- BEGIN: slip0132-info-line -->` / `<!-- END: slip0132-info-line -->` HTML-comment markers added to §11 in this cycle.
- **`verify-bundle-discards-slip0132-input-variant-asymmetry` — locked Option B per architect R1-I8:** Document the asymmetry as intentional. Add a clause to `design/SPEC_convert_v0_6.md` §11 (or appropriate verify-bundle SPEC location) stating: "`mnemonic verify-bundle` is structurally a checker that emits `VERIFIED` / `MISMATCH` status; SLIP-0132 input-normalization info notes are deliberately suppressed on this codepath to avoid breaking script callers that parse stderr line-by-line for status. Surfacing info-lines in `verify-bundle` would be an explicit UX policy change, not a bugfix." The 4 callsite-comments at `verify_bundle.rs:209/261/337/407` (each `// verify-bundle does not surface SLIP-0132 input-normalization signals.`) gain a one-line cross-pointer to the SPEC clause. Zero new emission code.

**Rationale for Option B over Option A** (architect R1 reasoning, transcribed for audit): Option A (surface info-line in verify-bundle) would thread `slip0132_signals` through `verify-bundle`'s render loop (~80 LOC); but `verify-bundle` output is consumed by scripts and humans checking for `VERIFIED`/`MISMATCH` signals; injecting info-lines into that path risks breaking callers that parse stderr line-by-line. Option B (document) has zero blast radius and is consistent with `verify-bundle`'s read-only checker semantics.

**Phase 7 exit gate:** Both carry-overs closed; FOLLOWUPS Status flipped.

### Phase 8 — SPEC + CHANGELOG + FOLLOWUPS close-out

- Edit `design/SPEC_convert_v0_6.md` per SPEC delta above.
- Create `design/SPEC_export_wallet_v0_7.md`.
- Create `design/SPEC_derive_child_v0_7.md`.
- `CHANGELOG.md`: new `[0.7.0] — 2026-05-XX` section with `### Added` (6 features), `### Changed` (NodeType enum extensions), `### Internal` (any pub(crate) API drift), `### Fixed` (carry-overs), `### FOLLOWUPS resolved` (6 + 2 = 8 items).
- `design/FOLLOWUPS.md`: close 8 entries with this-cycle SHAs.
- `README.md`: skim + add subcommand mentions for `export-wallet` and `derive-child`.

### Phase 9 — Release plumbing

- Bump `crates/mnemonic-toolkit/Cargo.toml` `version = "0.6.2"` → `"0.7.0"`. (Minor bump because new subcommands + new NodeTypes are user-visible features.)
- Confirm `Cargo.lock` updates.
- Final cargo gauntlet: build/test/clippy/doc clean.
- Tag `mnemonic-toolkit-v0.7.0`, push (user-confirmed).

## Critical files

**New (5):**
- `crates/mnemonic-toolkit/src/electrum.rs` (~300 LOC)
- `crates/mnemonic-toolkit/src/bip85.rs` (~200 LOC; 6 data-derivation app dispatchers)
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` (~250 LOC)
- `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (~150 LOC)
- `crates/mnemonic-toolkit/src/wallet_export.rs` (~250 LOC, format adapters)

**Modified (4):**
- `crates/mnemonic-toolkit/src/cmd/convert.rs` (NodeType extension + 6 new edges + refusal helpers + compute_outputs arms)
- `crates/mnemonic-toolkit/src/cmd/bundle.rs` (touch only if verify-bundle Phase-7 decision lands as Option A)
- `crates/mnemonic-toolkit/src/lib.rs` (re-export new modules)
- `crates/mnemonic-toolkit/src/main.rs` (subcommand dispatch for `export-wallet` + `derive-child`)

**Cargo:**
- `crates/mnemonic-toolkit/Cargo.toml` — add `bip38 = "1.1"`; bump version 0.6.2 → 0.7.0.

**Design (4):**
- `design/SPEC_convert_v0_6.md` — major edits per SPEC delta.
- `design/SPEC_export_wallet_v0_7.md` — NEW.
- `design/SPEC_derive_child_v0_7.md` — NEW.
- `design/FOLLOWUPS.md` — 8 close-outs.
- `CHANGELOG.md`.

**Tests:** ~10 new test files in `crates/mnemonic-toolkit/tests/` (one per feature surface) + reference-vector unit tests in each new module.

## Reuse opportunities (from prior cycles)

- `slip0132::neutral_for` + `render_slip0132_info_line` (v0.6.2) — pattern reference for the info-line emission discipline IF Phase 7 lands as Option A.
- `bundle::resolve_slots` + `BundleJson` — reused by `export-wallet` for slot resolution.
- `derive::derive_full` + `derive_slot::*` — reused for entropy/phrase intermediate flows in BIP-38 / Electrum composite edges.
- `convert::compute_outputs` `Xpub` arm — pattern reference for how new arms thread `--xpub-prefix` + `normalize_xpub_prefix` (v0.6.2) — relevant when Address arm adds its own normalization-trigger handling.
- BIP-84 reference vectors (`slip0132.rs:138` `BIP84_REF_ZPUB`) — reusable for address-derivation Phase 4 reference.
- `assert_cmd::Command` + `predicates::*` integration test patterns from `cli_*.rs`.

## Verification

Automated:

```fish
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --workspace
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

Expected post-Phase-9: ~430 passed / 0 failed / 2 ignored (363 baseline + ~67 new across the 6 features).

Manual smoke (post-Phase 9):

```fish
# BIP-38 round-trip
mnemonic convert --from wif=L1aW4aubDFB7yfras2S1mN3bqg9... --to bip38 --passphrase "test"
mnemonic convert --from bip38=6PfQu77ygVyJLZjfvMLyhLMQbYnu5uguoJJ4kMCLqWwPEdfpwANVS76gTX --passphrase "test" --to wif

# Casascius mini-key
mnemonic convert --from minikey=S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy --to wif

# Electrum
mnemonic convert --from electrum-phrase="wild father tree among universe such mobile favorite target dynamic credit identify" --to entropy

# BIP-85 derive
mnemonic derive-child --from xprv=xprv9s21ZrQH... --application bip39 --length 12 --index 0

# Address
mnemonic convert --from xpub=zpub6Mu... --to address --path "m/0/0" --script-type p2wpkh

# Export wallet
mnemonic export-wallet --format bitcoin-core --slot @0.xpub=zpub6Mu... --template wpkh
```

## Iterative-review log

- 2026-05-06 — Initial plan draft (post-3-round-brainstorm).
- 2026-05-06 — Architect review round 1 returned 3 Critical, 5 Important, 3 Low. Resolutions applied in-plan:
  - **C1 (Phase 0 stub semantics).** Phase 0 reduced to compile-time scaffold only (variant additions + `unreachable!` arms; no edges in `is_supported_direct_edge` until each feature phase ships); zero new tests in Phase 0. Each feature phase is now atomic (NodeType + edge + impl + tests in one phase-commit).
  - **C2 (descriptor-checksum missing piece).** Phase 5 task list now pins the 3-step pipeline: parse template + slot xpubs into `miniscript::Descriptor<DescriptorPublicKey>` → call `.to_string()` (auto-appends `#checksum`) → serialize per format. Pipeline lands in `design/SPEC_export_wallet_v0_7.md` §"Descriptor pipeline".
  - **C3 (BIP-85 application coverage).** User confirmed "all BIP-85 applications defined in the standard." Locked: 6 data-derivation apps (BIP-39, HD-Seed WIF, XPRV, HEX, PWD BASE64, PWD BASE85) in v0.7. RSA + RSA-GPG (apps `828365'` + `67797633'`) require `rsa` crate not in tree; explicitly out-of-scope, deferred to v0.8 FOLLOWUPS.
  - **I4 (bip38 source review).** Phase 1 task list now opens with a `bip38 v1.1.1` source-level security review (~500 LOC); deliverable persists to `design/agent-reports/v0_7-phase-1-bip38-security-review.md`; verdict gates the integration.
  - **I5 (Electrum 2FA refusal + corpus spike).** Phase 3 mandates a corpus spike against Electrum's `tests/test_wallet.py` before encode-direction lock; report to `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`. 2FA seed versions (`101`, `102`) explicitly refused via `refusal_electrum_2fa_unsupported`, NOT silently mis-decoded.
  - **I6 (--path semantics).** Locked `--path` mandatory; `--chain receive|change` + `--address-index N` shorthand deferred to v0.8 FOLLOWUPS as UX polish. Refusal `refusal_address_no_path` byte-pinned in tests.
  - **I7 (NodeType `from_token` strings).** Locked: `bip38`, `minikey` (no hyphen, consistent with `mk1`/`ms1`), `electrum-phrase` (hyphen acceptable; multi-word), `address`. `parse_from_input` error-message literal must update at `convert.rs:103-105`.
  - **I8 (Phase 7 carry-over).** Locked Option B: document the verify-bundle SLIP-0132 asymmetry as intentional in SPEC. 4 callsite-comments at `verify_bundle.rs` get a one-line cross-pointer to the SPEC clause. Zero new emission code.
  - **I9 (Bitcoin Core target version + range/timestamp defaults).** Locked: target Bitcoin Core 24+ (default 25); `--range <start,end>` flag default `0,999`; `--timestamp <unix|now>` flag default `now`; `--bitcoin-core-version` flag for shape-diffs.
  - **L10 (cycle size).** Noted: natural seam at Phase 4 vs Phases 5–6 (convert graph extensions vs new subcommands). If a blocker surfaces in Phase 5 or 6, ship interim v0.7 at Phase 4 exit gate; carry remaining to v0.7.1.
  - **L11 (refusal table completeness).** Refusal-table section above expanded with explicit enumeration of new "obvious" refusal pairs.
  - **L12 (`is_secret_bearing` for Bip38).** Phase 0 task list now adds `Bip38` to the `is_secret_bearing` arm.
- 2026-05-06 — Architect review round 2 verdict: **0 Critical / 0 Important / 3 Low.** All R1 resolutions verified GREEN against source. Three Low net-new fixes applied in-plan:
  - **N1 (BIP-85 DICE app omission).** Added `89101'` DICE to the Phase 6 explicit out-of-scope list with rationale (niche app; defer pending user demand).
  - **N2 (verify_bundle.rs line-number nit).** Plan cites lines 209/261/337/407; actual source has 208/260/336/406. Phase 7 task wording will direct the implementer to "locate by grep" rather than hardcoded lines (will shift after Phase 7's comment additions anyway).
  - **N3 (`--slot` reuse attribution).** Phase 5 args list now attributes `--slot` parser to the shared `slot_input` module (not "reused from `bundle`"), avoiding confusion during impl.
- Plan ready for ExitPlanMode → execution begins at Phase 0.
