# v0.7.1 BIP test vector audit matrix — mnemonic-toolkit

Built 2026-05-07 per the v0.7.1 audit cycle plan
(`/home/bcg/.claude/plans/let-s-work-on-the-soft-waterfall.md`).

Scope: every BIP / SLIP cited in `crates/mnemonic-toolkit/src/**` or
`design/SPEC_*.md`. Per BIP: published §Test Vectors enumerated verbatim;
each vector marked COVERED / MISSING / OUT-OF-SCOPE.

Status legend:
- COVERED — pinned in a named test fn (path::fn).
- MISSING — vector exists in spec, applies to v0.7.x scope, not yet pinned.
  Closes in the named phase.
- OUT-OF-SCOPE-PER-USER — user-confirmed skip (e.g. BIP-38 EC-multiplied).
- OUT-OF-SCOPE-PER-SPEC — vector targets surface the toolkit doesn't expose
  (e.g. BIP-32 invalid-key vectors that exercise low-level key parsing).

---

## BIP-32 — HD wallets

Source: <https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki> §Test Vectors.

### Test Vector 1 — seed `000102030405060708090a0b0c0d0e0f`

| # | Chain | Expected (xpub head) | Status | Notes |
|---|---|---|---|---|
| 1.1 | m | `xpub661MyMwAqRbcF...EGMcet8` | MISSING | Phase 1 (`tests/cli_convert_bip32_vectors.rs`) |
| 1.2 | m/0H | `xpub68Gmy5EdvgibQ...vgGDnw` | MISSING | Phase 1 |
| 1.3 | m/0H/1 | `xpub6ASuArnXKPbfE...puCkwQ` | MISSING | Phase 1 |
| 1.4 | m/0H/1/2H | `xpub6D4BDPcP2GT57...fcLW5` | MISSING | Phase 1 |
| 1.5 | m/0H/1/2H/2 | `xpub6FHa3pjLCk84B...iyLHV` | MISSING | Phase 1 |
| 1.6 | m/0H/1/2H/2/1000000000 | `xpub6H1LXWLaKsWFh...drTHy` | MISSING | Phase 1 |

### Test Vector 2 — seed `fffcf9f6...4e4b4845...4542`

| # | Chain | Expected (xpub head) | Status | Notes |
|---|---|---|---|---|
| 2.1 | m | `xpub661MyMwAqRbcF...EGuduB` | MISSING | Phase 1 |
| 2.2 | m/0 | `xpub69H7F5d8KSRgm...kQTPH` | MISSING | Phase 1 |
| 2.3 | m/0/2147483647H | `xpub6ASAVgeehLbnw...nBC5y4a` | MISSING | Phase 1 |
| 2.4 | m/0/2147483647H/1 | `xpub6DF8uhdarytz3...dhHKon` | MISSING | Phase 1 |
| 2.5 | m/0/2147483647H/1/2147483646H | `xpub6ERApfZwUNrhL...vRcEL` | MISSING | Phase 1 |
| 2.6 | m/0/2147483647H/1/2147483646H/2 | `xpub6FnCn6nSzZAw5...gAtqt` | MISSING | Phase 1 |

### Test Vector 3 — seed `4b381541...c73235be`

| # | Chain | Expected (xpub head) | Status | Notes |
|---|---|---|---|---|
| 3.1 | m | `xpub661MyMwAqRbcE...epUt13` | MISSING | Phase 1 — leading-zero chain code edge |
| 3.2 | m/0H | `xpub68NZiKmJWnxxS...19Zm4Y` | MISSING | Phase 1 |

### Test Vector 4 — seed `3ddd5602...db19b678`

m, m/0H, m/0H/1H. MISSING — Phase 1.

### Test Vector 5 — invalid extended key examples

OUT-OF-SCOPE-PER-SPEC. Toolkit does not expose a generic "decode arbitrary
extended key" surface; bitcoin v0.32 enforces these invariants at parse
time and is dependency-pinned.

---

## BIP-38 — passphrase-protected private key

Source: <https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki> §Test vectors.

### Non-EC-multiplied (3 + 2 = 5 published)

| # | Pass | WIF | BIP-38 | Status | Notes |
|---|---|---|---|---|---|
| V1 | TestingOneTwoThree | `5KN7Mzq...QQi5CVR` | `6PRVWUbk...Nh2ZoGg` | COVERED | `tests/cli_convert_bip38.rs::encrypt_wif_to_bip38_vector1_no_compression` + `decrypt_..._vector1_...` |
| V2 | Satoshi | `5HtasZ6...gi5` | `6PRNFFkZ...PX1dWByq` | COVERED | `tests/cli_convert_bip38.rs::*_vector2_no_compression` |
| V3 | unicode (U+03D2 U+0301 U+0000010400 U+0001F4A9) | `5Jajm8e...SZ4` | `6PRW5o9F...apcDQn` | MISSING | Phase 3 (Unicode-NFC edge worth pinning) |
| V4 | TestingOneTwoThree (compressed) | `L44B5gG...VpP` | `6PYNKZ1E...tpUeo` | COVERED | `tests/cli_convert_bip38.rs::*_vector3_compressed` |
| V5 | Satoshi (compressed) | `KwYgW8g...SK7` | `6PYLtMnX...PmY7` | MISSING | Phase 3 |

### EC-multiplied (4 published)

| # | Pass | BIP-38 | Status | Notes |
|---|---|---|---|---|
| EC1 | TestingOneTwoThree | `6PfQu77y...gTX` | OUT-OF-SCOPE-PER-USER | `bip38 = "1.1"` crate does not expose EC-multiplied; v0.7.1 Phase 3 pins ONE refusal byte-exact + filed v0.8 FOLLOWUP `bip38-ec-multiplied-mode-support` |
| EC2 | Satoshi | `6PfLGnQs...sH` | OUT-OF-SCOPE-PER-USER | same |
| EC3 | MOLON LABE (Lot 263183/Seq 1) | `6PgNBNN...Ypo1j` | OUT-OF-SCOPE-PER-USER | same |
| EC4 | ΜΟΛΩΝ ΛΑΒΕ (Lot 806938/Seq 1) | `6PgGWtx...ngH` | OUT-OF-SCOPE-PER-USER | same |

---

## BIP-39 — mnemonic seed

Source: <https://raw.githubusercontent.com/trezor/python-mnemonic/master/vectors.json>
(BIP-39 §Test Vectors delegates to this corpus).

24 vectors total in english array. Each is `[entropy_hex, mnemonic, seed_hex, xprv]`.

| # | Entropy (head) | Words | Status | Notes |
|---|---|---|---|---|
| 1 | `00000000...` (16 B) | 12 | COVERED-PARTIAL | TREZOR_12 ("abandon...about") used implicitly via `cli_convert_address.rs`; entropy-↔-seed-↔-xprv tuple NOT pinned. Phase 1 pins full quad in `tests/cli_convert_bip39_vectors.rs`. |
| 2 | `7f7f7f7f...` (16 B) | 12 | MISSING | Phase 1 |
| 3 | `80808080...` (16 B) | 12 | MISSING | Phase 1 |
| 4 | `ffffffff...` (16 B) | 12 | MISSING | Phase 1 |
| 5 | `00000000...` (24 B) | 18 | MISSING | Phase 1 |
| 6 | `7f7f7f7f...` (24 B) | 18 | MISSING | Phase 1 |
| 7 | `80808080...` (24 B) | 18 | MISSING | Phase 1 |
| 8 | `ffffffff...` (24 B) | 18 | MISSING | Phase 1 |
| 9 | `00000000...` (32 B) | 24 | COVERED-PARTIAL | TREZOR_24 ("abandon...art") used in derive.rs + cli_convert_happy_paths.rs (xprv head matches V9 row); Trezor entropy/seed quad NOT pinned. Phase 1 closes. |
| 10 | `7f7f7f7f...` (32 B) | 24 | MISSING | Phase 1 |
| 11 | `80808080...` (32 B) | 24 | MISSING | Phase 1 |
| 12 | `ffffffff...` (32 B) | 24 | MISSING | Phase 1 |
| 13 | `9e885d95...` (16 B) | 12 | MISSING | Phase 1 |
| 14 | `6610b259...` (24 B) | 18 | MISSING | Phase 1 |
| 15 | `68a79eac...` (32 B) | 24 | MISSING | Phase 1 |
| 16 | `c0ba5a8e...` (16 B) | 12 | MISSING | Phase 1 |
| 17 | `6d9be1ee...` (24 B) | 18 | MISSING | Phase 1 |
| 18 | `9f6a2878...` (32 B) | 24 | MISSING | Phase 1 |
| 19 | `23db8160...` (16 B) | 12 | MISSING | Phase 1 |
| 20 | `8197a4a4...` (24 B) | 18 | MISSING | Phase 1 |
| 21 | `066dca1a...` (32 B) | 24 | MISSING | Phase 1 |
| 22 | `f30f8c1d...` (16 B) | 12 | MISSING | Phase 1 |
| 23 | `c10ec20d...` (24 B) | 18 | MISSING | Phase 1 |
| 24 | `f585c11a...` (32 B) | 24 | MISSING | Phase 1 |

Plan §Phase 1 pins 6 entries (12-word + 24-word × 3 passphrase variants).
Remaining 18 stay MISSING (FOLLOWUP — full corpus pin deferred to v0.8 unless
free to fold in during Phase 1).

Trezor passphrase: every Trezor vector uses passphrase `TREZOR`. Phase 1
includes at least one passphrase-non-empty case to break the silent
empty-passphrase assumption.

---

## BIP-44 — multi-account hierarchy

Source: <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki> §Examples.

BIP-44 has no §Test Vectors section. The §Examples section is an illustrative
table only:

| coin | account | chain | address | path |
|---|---|---|---|---|
| Bitcoin | first | external | first | m/44'/0'/0'/0/0 |
| Bitcoin | first | external | second | m/44'/0'/0'/0/1 |
| Bitcoin | first | change | first | m/44'/0'/0'/1/0 |

Status: OUT-OF-SCOPE-PER-SPEC for vector-pinning (no concrete address
expected). BIP-44 path-shape conformance is exercised transitively by the
BIP-49/84/86 vectors (which inherit BIP-44 path notation). Phase 2 adds
NO direct BIP-44 vector tests.

---

## BIP-49 — P2WPKH-in-P2SH

Source: <https://github.com/bitcoin/bips/blob/master/bip-0049.mediawiki> §Test vectors.

Single published vector against TREZOR_12 mnemonic on testnet:

| # | Path | Expected | Status | Notes |
|---|---|---|---|---|
| 49.1 | m/49'/1'/0'/0/0 | `2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2` | COVERED | `tests/cli_convert_address.rs::phrase_to_address_bip49_p2sh_p2wpkh_reference_testnet` |
| 49.2 | account-level upub at m/49'/1'/0' | `upub5EFU65HtV5TeiSHmZZm7FUffBGy8UKeqp7vw43jYbvZPpoVsgU93oac7Wk3u6moKegAEWtGNF8DehrnHtv21XXEMYRUocHqguyjknFHYfgY` | MISSING | Phase 2 — pin testnet account upub via `--to xpub --xpub-prefix upub --network testnet` |
| 49.3 | mainnet receive index 0 (computed) | (no spec-published mainnet vector) | OUT-OF-SCOPE-PER-SPEC | BIP-49 spec only publishes testnet |
| 49.4 | mainnet receive index 1 (computed) | (none) | OUT-OF-SCOPE-PER-SPEC | same |

Phase 2 deliverable: pin 49.2 (account-level upub).

---

## BIP-84 — P2WPKH native segwit

Source: <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki> §Test vectors.

| # | Path | Expected | Status | Notes |
|---|---|---|---|---|
| 84.1 | m/84'/0'/0' (account zpub) | `zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs` | COVERED-PARTIAL | `cli_convert_slip0132.rs::*` exercise the same value as `TREZOR_24_BIP84_MAINNET_ZPUB` — but BIP-84 spec uses TREZOR_12. PIN-DRIFT: confirm mainnet zpub matches both seed lengths under the same path; if not, the v0.7 const is mis-named. **DISCOVERY-FLAG.** |
| 84.2 | m/84'/0'/0'/0/0 | `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` | COVERED | `tests/cli_convert_address.rs::xpub_to_address_bip84_p2wpkh_reference` (and `phrase_to_address_bip84_composite_with_template_inferred_script_type`) |
| 84.3 | m/84'/0'/0'/0/1 (second receive) | `bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g` | MISSING | Phase 2 |
| 84.4 | m/84'/0'/0'/1/0 (first change) | `bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el` | MISSING | Phase 2 |
| 84.5 | testnet receive 0 (no spec-published) | n/a | OUT-OF-SCOPE-PER-SPEC | BIP-84 spec is mainnet-only |

---

## BIP-85 — deterministic entropy

Source: <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki> §Test Vectors.

Master xprv per spec:
`xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb`

| # | Application | Path | Expected (head) | Status | Notes |
|---|---|---|---|---|---|
| 85.1 | BIP-39 12-word | m/83696968'/39'/0'/12'/0' | entropy `6250b68d...` | COVERED | `src/bip85.rs::tests::bip39_12_words_entropy_matches_spec` + `cli_derive_child.rs::cell_1_bip39_12_words_reference_vector` |
| 85.2 | BIP-39 18-word | m/83696968'/39'/0'/18'/0' | entropy `938033ed...` | COVERED | `cli_derive_child.rs::cell_2_bip39_18_words_reference_vector` |
| 85.3 | BIP-39 24-word | m/83696968'/39'/0'/24'/0' | entropy `ae131e23...` | MISSING | Phase 1 (cell_2-style) — gap, easy add |
| 85.4 | HD-Seed WIF | m/83696968'/2'/0' | `Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp` | COVERED | `cli_derive_child.rs::cell_3_hd_seed_wif_reference_vector` |
| 85.5 | XPRV | m/83696968'/32'/0' | `xprv9s21Z...mp3uEjXHJ42jFg7myX` | COVERED | `cli_derive_child.rs::cell_4_xprv_reference_vector` |
| 85.6 | HEX (64 B) | m/83696968'/128169'/64'/0' | `492db469...82a5c` | COVERED | `src/bip85.rs::tests::hex_64_bytes_entropy_matches_spec` + `cli_derive_child.rs::cell_5_hex_reference_vector` |
| 85.7 | PWD-BASE64 (21 chars) | m/83696968'/707764'/21'/0' | `dKLoepugzdVJvdL56ogNV` | COVERED | `src/bip85.rs::tests::pwd_base64_matches_spec` + `cli_derive_child.rs::cell_6a_pwd_base64_reference_vector` |
| 85.8 | PWD-BASE85 (12 chars) | m/83696968'/707785'/12'/0' | `_s`{TW89)i4`` | COVERED | `src/bip85.rs::tests::pwd_base85_matches_spec` + `cli_derive_child.rs::cell_6b_pwd_base85_reference_vector` |
| 85.9 | DICE (6-sided, 10 rolls) | m/83696968'/89101'/6'/10'/0' | `1,0,0,2,0,1,5,5,2,4` | OUT-OF-SCOPE-PER-USER | Refused at runtime alongside `rsa`/`rsa-gpg` per `cli_derive_child.rs::cell_7_unsupported_application_rsa_refusal`. v0.7 user direction: DICE deferred (niche); v0.8 FOLLOWUP `bip85-dice-application-impl-and-refusal-message-split` to (a) implement, OR (b) split DICE refusal text from RSA's so the byte-exact stderr distinguishes the two cases. |

In-scope gap: 85.3 (24-word) — Phase 1. OUT-OF-SCOPE: 85.9 (DICE) per user direction;
RSA + RSA-GPG also OUT-OF-SCOPE-PER-USER (rsa crate not in dep tree). All three
share a single refusal cell at `cli_derive_child.rs::cell_7_unsupported_application_rsa_refusal`.

---

## BIP-86 — Taproot single-key

Source: <https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki> §Test vectors.

| # | Path | Expected | Status | Notes |
|---|---|---|---|---|
| 86.1 | m/86'/0'/0' (account xpub) | `xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ` | COVERED-IMPLICIT | const `BIP86_ACCOUNT_XPUB` in `cli_convert_address.rs` matches; xpub-equality check is implicit via address-derivation pin |
| 86.2 | m/86'/0'/0'/0/0 | `bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr` | COVERED | `cli_convert_address.rs::xpub_to_address_bip86_p2tr_reference` + `entropy_to_address_bip86_composite` |
| 86.3 | m/86'/0'/0'/0/1 | `bc1p4qhjn9zdvkux4e44uhx8tc55attvtyu358kutcqkudyccelu0was9fqzwh` | MISSING | Phase 2 |
| 86.4 | m/86'/0'/0'/1/0 | `bc1p3qkhfews2uk44qtvauqyr2ttdsw7svhkl9nkm9s9c3x4ax5h60wqwruhk7` | MISSING | Phase 2 |

---

## BIP-93 — codex32 (master seed)

Source: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki> §Test Vectors.

OUT-OF-SCOPE-PER-SPEC at the toolkit level: ms1 (HRP `ms`) is BIP-93 directly
via `rust-codex32`, audited in the **mnemonic-secret** repo's audit-matrix
(`mnemonic-secret/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`).
Toolkit consumes ms-codec; it does not separately implement BIP-93.

---

## BIP-380 — descriptor expressions (checksum + key expressions)

Source: <https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki> §Test Vectors.

The spec publishes ~46 vectors total: 8 checksum vectors + 19 valid key-expression
forms + 19 invalid key-expression forms. v0.7 toolkit's surface area for BIP-380
is **descriptor emission with checksum**, not key-expression parsing — the latter
lives in rust-miniscript upstream.

### Checksum vectors (8)

| # | Descriptor | Status | Notes |
|---|---|---|---|
| 380.1 | `raw(deadbeef)#89f8spxm` (valid checksum) | MISSING | Phase 4 — pin in `tests/cli_export_wallet.rs` (toolkit emits descriptors with `#checksum`) |
| 380.2 | `raw(deadbeef)` (no checksum, REJECT) | OUT-OF-SCOPE-PER-SPEC | toolkit always emits the checksum form on export |
| 380.3 | `raw(deadbeef)#` (empty checksum, REJECT) | OUT-OF-SCOPE-PER-SPEC | same |
| 380.4 | `raw(deadbeef)#89f8spxmx` (9-char, REJECT) | OUT-OF-SCOPE-PER-SPEC | rust-miniscript enforces |
| 380.5 | `raw(deadbeef)#89f8spx` (7-char, REJECT) | OUT-OF-SCOPE-PER-SPEC | rust-miniscript enforces |
| 380.6 | `raw(deedbeef)#89f8spxm` (payload-error, REJECT) | OUT-OF-SCOPE-PER-SPEC | rust-miniscript enforces |
| 380.7 | `raw(deedbeef)##9f8spxm` (checksum-error, REJECT) | OUT-OF-SCOPE-PER-SPEC | rust-miniscript enforces |
| 380.8 | `raw(Ü)#00000000` (non-ASCII, REJECT) | OUT-OF-SCOPE-PER-SPEC | rust-miniscript enforces |

### Key-expression vectors (19 valid + 19 invalid = 38)

All 38 key-expression vectors (`KEY_EXPRESSION` patterns: hex pubkeys, WIF
privkeys, xpubs with various derivation suffixes, malformed origin metadata,
etc.) — OUT-OF-SCOPE-PER-SPEC. The toolkit does not parse key-expressions
directly; it constructs them from typed `bitcoin::bip32::Xpub` + `Fingerprint`
+ `DerivationPath` and lets rust-miniscript serialize them via
`miniscript::Descriptor::to_string()`. rust-miniscript's own test corpus exercises
these vectors upstream. Pinning them in the toolkit would be redundant.

Phase 4 deliverable: pin 380.1 against the toolkit's emitted descriptor for
the wpkh single-sig export (verifies our `#checksum` is BIP-380-conformant
end-to-end). Remaining 45 vectors are upstream-rust-miniscript contract tests.

---

## BIP-388 — wallet policies for descriptors

Source: <https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki> §Test Vectors / reference policies.

Spec lists 7 reference wallet policy patterns. Of these the toolkit's v0.7
`mnemonic export-wallet` covers exactly the singlesig + 2-of-N classes:

| # | Template | Reference key info | Status | Notes |
|---|---|---|---|---|
| 388.1 | `pkh(@0/**)` (BIP-44 legacy) | `[6738736c/44'/0'/0']xpub6Br37...` | OUT-OF-SCOPE-PER-USER | wpkh + p2sh-p2wpkh + p2tr only in v0.7 export-wallet; pkh deferred to v0.8 |
| 388.2 | `sh(wpkh(@0/**))` (BIP-49 nested) | `[6738736c/49'/0'/1']xpub6Bex1...` | MISSING | Phase 4 — pin via export-wallet against constructed bundle at m/49' |
| 388.3 | `wpkh(@0/**)` (BIP-84 native) | `[6738736c/84'/0'/2']xpub6CRQz...` | COVERED | `tests/cli_export_wallet.rs::cell_1_bitcoin_core_single_sig_wpkh_round_trip` (template-shape match; spec key info not byte-pinned because toolkit-derived xpub differs by seed) |
| 388.4 | `tr(@0/**)` (BIP-86 taproot) | `[6738736c/86'/0'/0']xpub6CryU...` | MISSING | Phase 4 — pin tr-template export-wallet round-trip |
| 388.5 | `wsh(sortedmulti(2,@0/**,@1/**))` (BIP-48 P2WSH) | 2 xpubs (cosigner) | COVERED | `tests/cli_export_wallet.rs::cell_2_bip388_wallet_policy_multisig_wsh_sortedmulti` |
| 388.6 | `wsh(thresh(3,pk(@0/**),s:pk(@1/**),s:pk(@2/**),sln:older(12960)))` (miniscript decay) | 3 xpubs at `48'/0'/0'/100'` | OUT-OF-SCOPE-PER-USER | toolkit v0.7 only emits sortedmulti template families; miniscript thresh deferred |
| 388.7 | `tr(@0/**,{sortedmulti_a(1,@0/<2;3>/*,@1/**),or_b(pk(@2/**),s:pk(@3/**))}` (taproot tree) | 4 xpubs | OUT-OF-SCOPE-PER-USER | tap-tree multisig deferred to v0.8 (per `mnemonic_toolkit_v0_7_plan` mem) |
| 388.8 | musig2 keypath/scriptpath | 3 xpubs | OUT-OF-SCOPE-PER-USER | musig2 not in any v0.7.x scope |

Phase 4 deliverables: pin 388.2 (BIP-49 nested wpkh-in-sh template export)
+ 388.4 (BIP-86 taproot template export). 388.3 + 388.5 are template-shape
COVERED but the spec's exact `[6738736c/...]` xpub values are not pinned
(toolkit derives from its own TREZOR_24/12 seed, not from BIP-388's
unspecified seed). Documented as COVERED-TEMPLATE-SHAPE-ONLY; full xpub
byte-pinning would require fabricating a BIP-388 seed (no spec value).

---

## SLIP-0132 — registered HD version bytes

Source: <https://github.com/satoshilabs/slips/blob/master/slip-0132.md>.
The doc has no formal §Test Vectors header; its "Bitcoin Test Vectors"
table is the canonical reference.

| # | Path | Prefix | Example head | Status | Notes |
|---|---|---|---|---|---|
| 132.1 | m/44'/0'/0' | xpub | `BosfCnif...` | OUT-OF-SCOPE-PER-SPEC | the doc's body shown above is missing the leading `xpub6` framing; toolkit's xpub round-trips exercise the BIP-32 neutral form against TREZOR_24 instead |
| 132.2 | m/49'/0'/0' | ypub | `Ww3ibxVf...` | COVERED-IMPLICIT | `src/slip0132.rs::tests` exercise ypub↔xpub via TREZOR_24 BIP-84-derived xpub (NOT the SLIP-0132 spec example xpub itself); matrix cell PIN-DRIFT — phase 5 pins the canonical SLIP-0132 ypub |
| 132.3 | m/84'/0'/0' | zpub | `rFR7y4Q2...` | COVERED | `src/slip0132.rs::tests::*` use BIP84_REF_ZPUB matching the spec example (cross-checked above with BIP-84 84.1) |
| 132.4 | m/49'/0'/0' Ypub multisig | Ypub | (no spec example in the doc body fetched) | MISSING | Phase 5 — pin Ypub mainnet from spec full §Examples table |
| 132.5 | m/48'/0'/0'/2' Zpub multisig | Zpub | (no spec example in fetched body) | MISSING | Phase 5 — pin Zpub mainnet |
| 132.6 | testnet upub | upub | (none in fetched body) | MISSING | Phase 5 |
| 132.7 | testnet vpub | vpub | (none in fetched body) | MISSING | Phase 5 |
| 132.8 | testnet Upub | Upub | (none in fetched body) | MISSING | Phase 5 |
| 132.9 | testnet Vpub | Vpub | (none in fetched body) | MISSING | Phase 5 |

DISCOVERY-FLAG: WebFetch returned a SLIP-0132 body that **garbled** the xpub
strings (dropped the `xpub6...` / `ypub6...` 4-char prefix). Phase 5 must
re-fetch from the raw markdown via `gh api repos/satoshilabs/slips/contents/...`
or local clone before pinning, NOT from the fetched text in this matrix.

---

## Electrum seed corpus (non-BIP)

Source: `electrum/tests/test_mnemonic.py::Test_seeds.mnemonics`. Captured
verbatim in `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`.

4 spike phrases (one per SeedVersion 01/100/101/102):

| # | Version | Status | Notes |
|---|---|---|---|
| EL.1 | 01 (standard) | MISSING | Phase 6 — promote spike phrase to `cli_convert_electrum.rs` decode + round-trip pin |
| EL.2 | 100 (segwit) | MISSING | Phase 6 |
| EL.3 | 101 (2FA-standard) | MISSING | Phase 6 — refusal byte-exact |
| EL.4 | 102 (2FA-segwit) | MISSING | Phase 6 — refusal byte-exact |

Existing `cli_convert_electrum.rs` tests use *toolkit-internal* phrases (not
the Electrum-published canonical 4). Phase 6 promotes the canonical 4.

---

## Casascius minikey (non-BIP)

Source: <https://en.bitcoin.it/wiki/Mini_private_key_format> + Casascius
canonical references.

| # | Length | Canonical | Status | Notes |
|---|---|---|---|---|
| C.1 | 22-char | `S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy` (wiki canonical) | COVERED-IMPL | `cli_convert_minikey.rs::decode_minikey_22char_to_wif_mainnet` |
| C.2 | 26-char | (no public canonical reference) | OUT-OF-SCOPE-PER-SPEC | Phase 7 documents the gap; impl-generated value retained |
| C.3 | 30-char | `SzavMBLoXU6kDrqtUVmffv` + impl-generated | COVERED-IMPL | `cli_convert_minikey.rs::decode_minikey_30char_to_wif_*` |

Phase 7 audits for any *additional* public canonical entries; pins if found.

---

## Summary

| Category | Total vectors | Covered | Missing (in-scope) | Out-of-scope-per-user | Out-of-scope-per-spec |
|---|---|---|---|---|---|
| BIP-32 | 18 | 0 | 16 (Phase 1) | 0 | 2 (vector 5 invalid keys) |
| BIP-38 | 9 | 4 | 2 (Phase 3) | 4 (EC-mult) | 0 |
| BIP-39 | 24 | 2 PARTIAL | 22 (Phase 1 covers 6; 16 carry over to v0.8) | 0 | 0 |
| BIP-44 | 0 | — | — | — | examples-only, no vectors |
| BIP-49 | 2 | 1 | 1 (Phase 2) | 0 | 2 (no mainnet) |
| BIP-84 | 4 | 2 | 2 (Phase 2) | 0 | 1 (no testnet) |
| BIP-85 | 9 | 7 | 1 (Phase 1: 85.3) | 1 (DICE 85.9) | 0 |
| BIP-86 | 4 | 2 | 2 (Phase 2) | 0 | 0 |
| BIP-93 | n/a | — | — | — | delegated to ms-codec audit |
| BIP-380 | 46 | 0 | 1 (Phase 4: checksum 380.1) | 0 | 45 (7 reject-checksum + 38 key-expression: rust-miniscript surface) |
| BIP-388 | 8 | 2 SHAPE | 2 (Phase 4) | 4 | 0 |
| SLIP-0132 | 9 | 1 + 1 IMPLICIT | 7 (Phase 5) | 0 | 1 |
| Electrum | 4 | 0 (canonical) | 4 (Phase 6) | 0 | 0 |
| Casascius | 3 | 2 IMPL | 0 | 0 | 1 (no canonical) |
| **TOTAL** | **101** | **~25** | **~50** | **~8** | **~13** |

Phase 1–6 target: close the ~50 in-scope MISSING entries. v0.8 carry: ~16
(rest of BIP-39 corpus).

---

## Discoveries (require architect review before pinning)

1. **DISCOVERY-FLAG (84.1 / 132.2 / 132.3) — TREZOR_24 vs TREZOR_12 zpub
   collision check.** Toolkit's `cli_convert_slip0132.rs` constants name
   `TREZOR_24_BIP84_MAINNET_ZPUB` but the value is identical to the
   BIP-84 spec example, which uses the TREZOR_12 mnemonic (not 24).
   Phase 2 must verify whether the constant is mis-named (24-word seed
   produces a *different* zpub at `m/84'/0'/0'`) OR the constant is the
   BIP-84 spec value mis-attributed in code. Likely mis-naming bug:
   the existing test passes because it never re-derives from a 24-word
   seed; it just consumes the constant as input. **Action:** rename
   const + add a 24-word-seed derivation pin distinct from the spec
   value. Not a wire bug, but a fixture-attribution drift.

2. **DISCOVERY-FLAG (SLIP-0132 fetch).** WebFetch returned a SLIP-0132
   document body with truncated xpub strings (prefix `xpub`/`ypub`/`zpub`
   stripped from the canonical `xpub6...` form to bare `BosfCnif...`).
   Phase 5 must re-fetch from raw GitHub or locally before pinning;
   the values in §SLIP-0132 above are NOT byte-pinnable as transcribed.
   Not an impl bug — a tooling caveat for the matrix builder.

3. **AMBIGUOUS spec section — BIP-380.** The §Test Vectors section is
   sparse: lists 1 valid checksum + 7 invalid forms + key-expression
   bullet patterns. No exhaustive corpus of (descriptor, checksum) pairs
   like BIP-93 has. The toolkit does not need more — `rust-miniscript`
   is the test-source-of-truth for descriptor checksums. Documented
   here so a future reviewer doesn't expect a 50-vector matrix.

4. **AMBIGUOUS spec section — BIP-388.** The §Test Vectors section
   gives 7 reference templates with concrete `[6738736c/44'/...]`
   xpubs but no underlying seed. Without a seed, "round-trip the spec
   xpub through our derivation" is not testable. v0.7.1 settles for
   "template-shape COVERED" + spec-xpub-quoted-in-source — same
   resolution as BIP-388 reference impl tests at upstream rust-miniscript.
