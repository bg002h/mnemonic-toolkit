# mnemonic-toolkit v0.7.1 SPEC — published-vector pinning audit

**Version:** 0.7.1
**Date:** 2026-05-07
**Status:** SHIPPED (Phase 8 close-out; vectors-only patch atop v0.7.0).
**Predecessors:** [SPEC_export_wallet_v0_7.md](SPEC_export_wallet_v0_7.md), [SPEC_derive_child_v0_7.md](SPEC_derive_child_v0_7.md), [SPEC_convert_v0_6.md](SPEC_convert_v0_6.md).
**Audit matrix:** [`design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`](agent-reports/v0_7_1-bip-test-vector-audit-matrix.md).

## §1 Purpose

v0.7.1 is a vectors-only patch cycle: pin every published §Test Vectors entry from the BIPs, SLIPs, and non-BIP specs the toolkit cites, in a named test fn, byte-exact against the spec source. No behavior change; no wire-format change; no new subcommand or flag. The cycle closes ambiguity about what is and is not vector-pinned, surfaces 3 substantive findings that landed as SPEC corrections + v0.8 FOLLOWUPs, and seeds a v0.8 carry-over slate.

## §2 Coverage by spec

Lifted from the audit matrix §Summary. Counts are individual published vectors (not test functions).

| Spec | Total | Covered | Missing (v0.8 carry) | OOS-per-user | OOS-per-spec |
|---|---|---|---|---|---|
| BIP-32 | 18 | 17 | 0 | 0 | 1 (TV5 invalid keys) |
| BIP-38 | 9 | 9 (5 non-EC + 4 EC-decrypt; V3 `#[ignore]`) | 0 (ENCRYPT EC-mult tracked) | 0 | 0 |
| BIP-39 | 24 | 6 | 18 | 0 | 0 |
| BIP-44 | 0 | — | — | — | examples-only |
| BIP-49 | 4 | 2 | 0 | 0 | 2 (no mainnet) |
| BIP-84 | 4 | 4 | 0 | 0 | 1 (no testnet) |
| BIP-85 | 9 | 7 | 1 (85.3) | 1 (DICE) | 0 |
| BIP-86 | 4 | 4 | 0 | 0 | 0 |
| BIP-93 | n/a | — | — | — | delegated to ms-codec |
| BIP-380 | 46 | 1 (380.1 checksum) | 0 | 0 | 45 (rust-miniscript surface) |
| BIP-388 | 8 | 4 SHAPE | 0 | 4 | 0 |
| SLIP-0132 | 9 | 3 | 0 | 0 | 6 (no spec xpub published) |
| Electrum | 4 | 4 | 0 | 0 | 0 |
| Casascius | 3 | 2 IMPL | 0 | 0 | 1 (no canonical) |
| **TOTAL** | **101** | **63** | **19** | **5** | **14** |

Test corpus: v0.7.0 ship at 444 / 0 / 2 → v0.7.1 ship at 484 / 0 / 4 (+40 active, +2 ignored). Active deltas distribute across Phases 1 (BIP-32 + BIP-39), 2 (BIP-49/84/86), 3 (BIP-38 V3 cite-only + V5 + EC1–EC4), 4 (BIP-380 + BIP-388), and 5 (SLIP-0132). Phases 6 (Electrum) and 7 (Casascius) closed gaps via citation strengthening + matrix correction (no new tests).

## §3 Discoveries

Three substantive findings surfaced during the audit; each is recorded inline below with its disposition.

### §3.a BIP-38 EC-multiplied DECRYPT actually works (Phase 3.B erratum)

The v0.7.0 SPEC §12 + Phase 1 BIP-38 security review (`design/agent-reports/v0_7-phase-1-bip38-security-review.md`) stated that `bip38 = "1.1"`'s `Decrypt` impl rejected EC-multiplied codes with a typed error variant. Empirical Phase 3 testing disconfirmed: all 4 BIP-38 §"Test vectors" EC-multiplied vectors (EC1–EC4) decrypt correctly through the toolkit's existing `(Bip38, Wif)` arm. SPEC §12 corrected in commit `2c59b27`; FOLLOWUP `bip38-spec-section-12-ec-multiplied-erratum` resolved at the same commit. 4 cells flipped OUT-OF-SCOPE-PER-USER → COVERED (DECRYPT). Encrypt-side EC-mult (intermediate-code workflow) becomes the new gap, tracked as v0.8 FOLLOWUP `bip38-ec-multiplied-encrypt-mode-support`.

### §3.b BIP-38 V3 Unicode-NFC passphrase contains U+0000 NULL (Phase 3.A)

BIP-38 §"Test vectors" vector 3 specifies a 5-codepoint passphrase including U+0000 between U+0301 and U+10400. POSIX `execve` truncates argv at NULL; the existing `--passphrase=-` stdin path applies `.trim()`. The encrypt + decrypt cells are pinned with spec values verbatim in `#[ignore]`'d test bodies (`tests/cli_convert_bip38.rs::{encrypt,decrypt}_..._spec_vector3_unicode_nfc_passphrase`); flipped MISSING → COVERED-IGNORED. v0.8 FOLLOWUP `bip38-spec-vector-3-null-byte-passphrase` tracks exposing a NULL-safe input channel (e.g. `--passphrase-bytes-hex <hex>`).

### §3.c SLIP-0132 publishes only 3 xpub examples (Phase 5)

Phase 0 WebFetch had returned a SLIP-0132 document body with truncated xpub strings (4-char prefix dropped). Phase 5 re-fetched via `gh api repos/satoshilabs/slips/contents/slip-0132.md` and confirmed §"Bitcoin Test Vectors" only publishes 3 mainnet single-sig xpubs (BIP-44 / BIP-49 / BIP-84). All 3 are now COVERED in `src/slip0132.rs::tests::slip0132_spec_bitcoin_test_vector_*`. The 6 multisig + testnet variants have no published spec xpubs and are exercised behaviorally by the existing `apply_*_variants` tests — reclassified OUT-OF-SCOPE-PER-SPEC.

## §4 Out-of-scope classifications

Per-spec OUT-OF-SCOPE entries with rationale:

- **BIP-32 TV5** — invalid extended-key examples; toolkit doesn't expose a generic "decode arbitrary extended key" surface; `bitcoin v0.32` enforces these invariants at parse time.
- **BIP-38 EC-multiplied ENCRYPT** — requires intermediate-code workflow (passphrase code → 3rd party adds entropy); new subcommand-grade feature deferred to v0.8 (FOLLOWUP `bip38-ec-multiplied-encrypt-mode-support`).
- **BIP-44 §Examples** — illustrative path table only; no concrete address/expected-value assertions; path-shape conformance exercised transitively by BIP-49/84/86 vectors.
- **BIP-49 mainnet** — spec only publishes testnet vectors.
- **BIP-84 testnet** — spec is mainnet-only.
- **BIP-85 DICE / RSA / RSA-GPG** — DICE per-user direction (niche); RSA / RSA-GPG OUT-OF-SCOPE-PER-USER (rsa crate not in dep tree). All three share refusal cell `cli_derive_child.rs::cell_7_unsupported_application_rsa_refusal`.
- **BIP-93** — delegated to `mnemonic-secret/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`. Toolkit consumes ms-codec; does not separately implement BIP-93.
- **BIP-380 reject-checksum + key-expression vectors (45 of 46)** — `rust-miniscript` is the source-of-truth for checksum + key-expression parsing; pinning upstream's contract would be redundant.
- **BIP-388 templates 388.1 / 388.6 / 388.7 / 388.8** — `pkh` / miniscript-thresh / tap-tree multisig / musig2 — none in v0.7 export-wallet scope.
- **BIP-388 byte-exact spec xpub byte-pinning** — spec gives `[6738736c/...]` xpubs without an underlying seed; "round-trip the spec xpub through our derivation" not testable. v0.7.1 settles for COVERED-TEMPLATE-SHAPE + spec-xpub-quoted-in-source.
- **SLIP-0132 multisig + testnet (132.4–132.9)** — spec only publishes the version-byte registry, no example xpubs. Behavior covered by `apply_*_variants` tests.
- **Casascius 26-char (C.2)** — no public canonical reference; impl-generated value retained for length-class coverage only.

## §5 v0.8 carry-overs

In-scope vectors deferred to a future cycle:

- **BIP-39 — 18 of 24 Trezor reference vectors** unpinned (numbers 2/3/5/6/7/8/10/11/14/16/17/18/19/20/21/22/23/24). Phase 1 pinned 6 (numbers 1/4/9/12/13/15) covering 12-word + 24-word × 3 passphrase variants. Remaining 18 are mechanical line-items; folded as a single v0.8 FOLLOWUP if surfaced.
- **BIP-85 vector 85.3** (24-word BIP-39 application) — gap, easy add.
- **`bip38-ec-multiplied-encrypt-mode-support`** — emit BIP-38 EC-multiplied form via intermediate codes (FOLLOWUP filed Phase 3.B).
- **`bip38-spec-vector-3-null-byte-passphrase`** — NULL-safe passphrase input channel (FOLLOWUP filed Phase 3.A).
- **`bip38-spec-section-12-ec-multiplied-erratum`** — already resolved at `2c59b27` this cycle; carried in FOLLOWUPS for audit history continuity.
- **BIP-85 DICE refusal text split** — v0.8 FOLLOWUP `bip85-dice-application-impl-and-refusal-message-split` tracks splitting DICE refusal text from RSA's so the byte-exact stderr distinguishes the two cases (or implementing DICE).

## §6 Cross-refs

- Audit matrix: [`design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`](agent-reports/v0_7_1-bip-test-vector-audit-matrix.md).
- BIP-38 SPEC: [`design/SPEC_convert_v0_6.md`](SPEC_convert_v0_6.md) §12 (v0.7.1 erratum block).
- v0.7 plan source: `/home/bcg/.claude/plans/let-s-work-on-the-soft-waterfall.md`.
- Sibling repos' audit matrices (cross-format star): `mnemonic-secret/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md` (BIP-93), `descriptor-mnemonic/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md` (md-codec / mk-codec siblings).
