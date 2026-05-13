# v0.8.0 BIP test vector audit matrix — mnemonic-toolkit (cross-repo hub)

Built 2026-05-13 per the v0.8.0 cross-repo audit cycle.
**Predecessor (still authoritative for everything v0.8.0 did not
change):** [`v0_7_1-bip-test-vector-audit-matrix.md`](v0_7_1-bip-test-vector-audit-matrix.md)
(marked SUPERSEDED at v0.8.0 in lockstep with this file).

**Cycle SPEC:** [`design/SPEC_test_vector_audit_v0_8_0.md`](../SPEC_test_vector_audit_v0_8_0.md).
**Cycle plan:** `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`.
**Survey precursor:** [`v0_8_0-cross-repo-bip-vector-survey.md`](v0_8_0-cross-repo-bip-vector-survey.md).
**Per-phase reports:**
- Phase 0: [`v0_8_0-phase-0-spec-plan-r1.md`](v0_8_0-phase-0-spec-plan-r1.md)
- Phase 1 (md-codec): `descriptor-mnemonic/design/agent-reports/v0_8_0-phase-1-bip341-wallet-r1.md`
- Phase 2 (ms-codec): `mnemonic-secret/design/agent-reports/v0_8_0-phase-2-bip93-corpus-r1.md`
- Phase 3 (mnemonic-toolkit): [`v0_8_0-phase-3-bip85-fill-r1.md`](v0_8_0-phase-3-bip85-fill-r1.md)

## §0 Cross-repo coverage (new at v0.8.0)

The v0.7.1 audit matrix in this repo was toolkit-only. The v0.8.0
cycle lifts it to first-class cross-repo by naming each sibling
repo's matrix file inline.

| Repo | v0.8.0 matrix file | Cycle phase | Delta vs v0.7.1 |
|---|---|---|---|
| `mnemonic-toolkit` | this file | Phase 3 + Phase 0 cycle ownership | +1 cell (BIP-85 v85.3); §0 added |
| `descriptor-mnemonic` | `descriptor-mnemonic/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` | Phase 1 | +7 cells (BIP-341 `scriptPubKey`) + 2 invariants |
| `mnemonic-secret` | `mnemonic-secret/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` | Phase 2 | +4 valid + 64 invalid + 1 invariant cell (BIP-93) |
| `mnemonic-key` | `mnemonic-key/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` | no scope | 0 (symmetry-only no-scope companion) |

**Cycle cell-count net:** +18 BIP-39 (pre-cycle closure at
`85694b2`) + 1 BIP-85 + 4 BIP-93 valid + 64 BIP-93 invalid + 7
BIP-341 = **94 vectors** newly covered across the constellation
vs the v0.7.1 baseline. (BIP-39 +18 was already on master before
the cycle started; SPEC §2 records this.)

## §1 mnemonic-toolkit coverage delta vs v0.7.1

### BIP-85 — deterministic entropy (v0.7.1: 7/9 → v0.8.0: 8/9)

| # | Description | v0.7.1 | v0.8.0 | Test fn |
|---|---|---|---|---|
| 85.1 | 12-word BIP-39 | COVERED | COVERED | `cell_1_bip39_12_words_reference_vector` |
| 85.2 | 18-word BIP-39 | COVERED | COVERED | `cell_2_bip39_18_words_reference_vector` |
| **85.3** | **24-word BIP-39** | **MISSING** | **COVERED (NEW)** | **`cell_2b_bip39_24_words_reference_vector`** |
| 85.4 | HD-Seed WIF | COVERED | COVERED | `cell_3_hd_seed_wif_reference_vector` |
| 85.5 | XPRV | COVERED | COVERED | `cell_4_xprv_reference_vector` |
| 85.6 | HEX 16/32/64 bytes | COVERED | COVERED | `cell_5_hex_reference_vector` |
| 85.7 / .8 | Password Base64/Base85 | COVERED | COVERED | `cell_6a/b_pwd_base*_reference_vector` |
| 85.9 | DICE | OUT-OF-SCOPE-PER-USER | (unchanged) | refused via `cell_7_unsupported_application_*` |

### BIP-39 — Trezor English corpus (pre-cycle closure)

The v0.7.1 §5 carry-over (6/24 → 24/24) was already closed by
`feat(v0.8-phase-8)` commit `85694b2` *before* the cycle
started. The parametric loader at
`tests/cli_convert_bip39_vectors.rs::bip39_trezor_english_corpus_full`
iterates all 24 entries in `tests/bip39_trezor_vectors.json`.
SPEC §2 row updated to record this; cycle's Phase 3 did not need
to touch BIP-39.

## §2 BIP coverage unchanged from v0.7.1

All other BIP / SLIP / non-BIP-spec coverage in the v0.7.1
toolkit matrix carries forward unchanged at v0.8.0:

- BIP-32 TV1–4 covered, TV5 OUT-OF-SCOPE-PER-SPEC
- BIP-38 V1/V2/V4/V5 + EC1–EC4 decrypt covered, V3 ignored-but-pinned,
  ENCRYPT EC-mult tracked as separate FOLLOWUP
- BIP-49 testnet 2/2 covered, mainnet OUT-OF-SCOPE-PER-SPEC
- BIP-84 mainnet 4/4 covered
- BIP-86 4/4 covered
- BIP-380 §380.1 covered, 45/46 OUT-OF-SCOPE-PER-LAYER
- BIP-388 4 SHAPE-covered, 4 deferred-per-scope
- SLIP-0132 3/9 covered, 6 OUT-OF-SCOPE-PER-SPEC
- Electrum 4/4, Casascius 2/3 IMPL + 1 OUT-OF-SCOPE-PER-SPEC

## §3 New OUT-OF-SCOPE classifications (filed for explicit closure)

Per SPEC §3, three new OUT-OF-SCOPE classifications surfaced
during the Phase 0 cross-repo survey and were filed as FOLLOWUPS
for explicit closure rather than silent skips:

- **BIP-340 (Schnorr signatures)** — no signing surface in any of
  the four sibling crates. Sidecar `bip-0340/test-vectors.csv`
  cannot be adopted without first introducing signing.
  FOLLOWUP: `bip340-schnorr-signing-surface-evaluation` (toolkit).
- **BIP-341 `keyPathSpending`** — same gating as BIP-340; the
  `scriptPubKey` corpus was adopted but the sibling
  `keyPathSpending` (1 vector, signing flow) requires Schnorr.
  FOLLOWUP: `bip341-keypath-signing-vector-coverage` (md-codec).
- **BIP-39 Japanese wordlist** — ms-codec is English-only at
  v0.8.x; Japanese vectors require JP wordlist plumbing.
  FOLLOWUP: `bip39-japanese-wordlist-support` (toolkit, with
  ms-codec impl scope).

## §4 Cycle FOLLOWUPS state

`bip-vector-adoption-v0_8` entries land in all four repos:

- `mnemonic-toolkit/design/FOLLOWUPS.md` ✓ (`d269dda`)
- `descriptor-mnemonic/design/FOLLOWUPS.md` ✓ (`b464f3f`)
- `mnemonic-secret/design/FOLLOWUPS.md` ✓ (`7101c16`)
- `mnemonic-key/design/FOLLOWUPS.md` ✓ (`37d4fca`, no-scope companion)

All four entries close in lockstep when Phase E patch tags land.

## §5 v0.8.0 → v0.9.0 carry-overs

Filed as FOLLOWUPS for the next cycle:

- `bip93-invalid-corpus-granular-error-pin` (ms-codec) — tighten
  the parametric invalid-corpus `is_err()` assertion to per-entry
  `codex32::Error` variant classification.
- BIP-380 §380.2–380.8 checksum vectors — could move into md-cli
  if the descriptor-checksum surface is exercised independently of
  `rust-miniscript`. Survey-noted, not yet filed.
- BIP-322 (signed messages) — same gating as BIP-340. Carry-over
  if signing surface lands.
- BIP-174 PSBT — separate product-line FOLLOWUP, not vector
  adoption.
