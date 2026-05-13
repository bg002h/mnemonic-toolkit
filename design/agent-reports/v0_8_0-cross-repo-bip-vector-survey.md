# v0.8.0 cross-repo BIP-vector adoption survey

**Date:** 2026-05-13
**Surveyor:** `general-purpose` agent (Sonnet 4.6), dispatched per
the v0.8.0 cycle's Phase 0 brainstorm.
**Scope:** every BIP cited in source across `mnemonic-toolkit`,
`mnemonic-secret`, `mnemonic-key`, `descriptor-mnemonic`; plus a
short list of uncited BIPs that plausibly apply.
**Output:** the gaps named in
`design/SPEC_test_vector_audit_v0_8_0.md` §1 and the phase structure
in `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`.

## Inputs to the survey

- BIPs cited (from `grep -rhoE "BIP[ -]?[0-9]+"` across all 4 repos):
  BIP-32, BIP-38, BIP-39, BIP-44, BIP-48, BIP-49, BIP-65, BIP-84,
  BIP-86, BIP-93, BIP-112, BIP-173, BIP-340, BIP-341, BIP-342, BIP-350,
  BIP-379, BIP-380, BIP-388.
- Existing audit matrix: toolkit-only, at
  `mnemonic-toolkit/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`.
- Pre-existing pinned upstream-BIP files (verified during survey):
  `bip32_vectors.rs`, `bip39_trezor_vectors.json` +
  `cli_convert_bip39_vectors.rs`, `cli_convert_bip38.rs`,
  `cli_convert_address.rs` (BIP-49 / -84 / -86 inline values),
  `cli_derive_child.rs` (BIP-85), `cli_export_wallet.rs` (BIP-380
  §380.1 + BIP-388 template-shape pins), `bip93_cross_format.rs`,
  `bip39_integration.rs`.
- `mk-codec/tests/vectors/v0.1.json` and
  `md-codec/tests/vectors/*.{template,descriptor.json}` are
  **repo-internal goldens**, NOT upstream BIP vectors.

## Adoption matrix (one row per cited BIP + uncited additions)

| BIP | Publishes vectors? | Upstream URL | Adopted? | If not, where | ROI |
|---|---|---|---|---|---|
| **BIP-32** | Yes (inline §Test Vectors) | `bip-0032.mediawiki` | Yes — TV1–4 in `bip32_vectors.rs`. TV5 (invalid keys) OUT-OF-SCOPE-PER-SPEC. | n/a | n/a |
| **BIP-38** | Yes (inline) | `bip-0038.mediawiki` | Mostly — non-EC V1/V2/V4/V5 + EC1–EC4 decrypt covered; V3 ignored-but-pinned; ENCRYPT EC-mult is a tracked v0.8 feature FOLLOWUP, not vector adoption. | n/a (feature, not vector adoption) | low |
| **BIP-39** | Yes (Trezor sidecar at `github.com/trezor/python-mnemonic/blob/master/vectors.json` + JP wordlist sidecar) | (see Trezor / JP repos) | Partial — 6/24 English-Trezor pinned. **18 English MISSING; one-line-per-vector cost.** JP vectors require JP wordlist (not in scope). | mnemonic-toolkit (extend JSON fixture, generalize loader). | **high** (English fill — cheap closure) |
| **BIP-44** | No formal vectors | — | Examples-only spec; path-shape exercised via BIP-49/84/86. | n/a | n/a |
| **BIP-48** | No formal vectors | — | Examples-only. | n/a | n/a |
| **BIP-49** | Yes (inline, testnet-only) | `bip-0049.mediawiki` | Yes — both spec vectors pinned in `cli_convert_address.rs`. | n/a | n/a |
| **BIP-65** | No vectors | — | Script-level, OOS for codec. | n/a | n/a |
| **BIP-84** | Yes (inline, mainnet) | `bip-0084.mediawiki` | Yes — 84.1–84.4 pinned. | n/a | n/a |
| **BIP-85** | Yes (inline) | `bip-0085.mediawiki` | Mostly — 85.1/.2/.4/.5/.6/.7/.8 covered; **85.3 (24-word BIP-39) MISSING** (one-cell add); 85.9 DICE refused-with-cell. | mnemonic-toolkit (one cell in `cli_derive_child.rs`). | med (low cost, completes BIP) |
| **BIP-86** | Yes (inline) | `bip-0086.mediawiki` | Yes — 86.1–86.4 pinned. | n/a | n/a |
| **BIP-93** | Yes (inline 5 valid + 64 invalid) | `bip-0093.mediawiki` | Partial — §93.4 (`leet` 256-bit) byte-pinned in `bip93_cross_format.rs`. **§93.1–.3, §93.5 + 64 invalid MISSING** (delegated to `rust-codex32 =0.1.0` upstream; local pins close the upstream-drift surface). Count corrected from "42" (v0.7.1 footnote) to "64" via Phase 0 architect-review verification using `gh api repos/bitcoin/bips/contents/bip-0093.mediawiki`. | `mnemonic-secret/crates/ms-codec/tests/bip93_inline_vectors.rs`. | **high** (defends against `rust-codex32` semantic-drift on a future bump; 69 vectors) |
| **BIP-112** | No vectors | — | Script-level, OOS for codec. | n/a | n/a |
| **BIP-173** | Yes (inline valid + invalid corpus) | `bip-0173.mediawiki` | Apparently no direct pin in any of the 4 repos; exercised transitively via `bitcoin::Address` / `rust-bech32`. | Optional direct pin in `mnemonic-toolkit/tests/bip173_bech32_vectors.rs`; or rely on upstream `rust-bech32` corpus. | low |
| **BIP-340** | Yes (sidecar CSV at `bip-0340/test-vectors.csv`) | `bip-0340/` | No — and **no signing surface** in the constellation. File a FOLLOWUP `bip340-schnorr-signing-surface-evaluation` for explicit closure rather than silent skip. | (filed in SPEC §3) | low |
| **BIP-341** | Yes (sidecar JSON at `bip-0341/wallet-test-vectors.json`) | `bip-0341/` | No — md-codec assembles `tr(K, {…})` trees but only BIP-86 key-spend path is pinned today. **Largest unclaimed BIP corpus directly load-bearing.** 7 vectors per current snapshot. | `descriptor-mnemonic/crates/md-codec/tests/bip341_wallet_vectors.rs` + sidecar JSON pinned by sha256sum. | **high** |
| **BIP-342** | No own vectors (points to BIP-341 sidecar) | (see BIP-341) | Same file as BIP-341 row. | (covered by BIP-341 row) | (subsumed) |
| **BIP-350** | Yes (inline valid + invalid + v0–v16 segwit addr) | `bip-0350.mediawiki` | No direct pin; exercised transitively via `bitcoin v0.32` + BIP-86 happy path. | Optional direct pin; likely redundant with BIP-86 + BIP-173. | low |
| **BIP-379** | TBD upstream | `bip-0379.md` | n/a — blocked upstream. | When upstream publishes, mirror to `md-codec`. | n/a |
| **BIP-380** | Yes (inline: 8 checksum + 19 valid + 19 invalid key-exprs) | `bip-0380.mediawiki` | Partial — §380.1 pinned; 45 key-expr vectors OUT-OF-SCOPE-PER-LAYER (`rust-miniscript` surface). Checksum vectors 380.2–380.8 could move to `md-cli` but not required at v0.8. | (deferred; not on critical path) | low–med |
| **BIP-388** | Inline reference policies (no key-source seed) | `bip-0388.mediawiki` | Mostly — 388.3 / 388.5 byte-covered; 388.2 / 388.4 template-shape; 388.1 / .6 / .7 / .8 deferred. Byte-exact spec xpub round-trip structurally impossible (continues v0.7.1). | n/a | low |

## Uncited BIPs

| BIP | Relevance | Disposition |
|---|---|---|
| **BIP-174** PSBT | Tracked as separate v0.8 product-line FOLLOWUP at `mnemonic-toolkit/design/FOLLOWUPS.md`. | OOS for this cycle. |
| **BIP-322** | No signing surface yet; sidecar vectors exist if/when one lands. | OOS. |
| **BIP-67** sortedmulti | Transitively covered via `rust-miniscript`. No BIP-level vectors. | OOS. |
| **BIP-21** URI | Not in scope. | OOS. |

## Top 3 highest-ROI gaps (input to SPEC §1)

1. **BIP-341 `wallet-test-vectors.json` → `md-codec`.** Largest unclaimed
   corpus directly applicable. Today's only taproot pin (BIP-86)
   exercises key-spend only; script-tree assembly + tweaked-output-key
   derivation are unverified against an upstream-authoritative source.
2. **BIP-39 Trezor English fill (18/24 missing) → `mnemonic-toolkit`.**
   JSON loader exists; ~6 lines per vector; loader generalization is
   the one structural change. v0.7.1 §5 carry-over already named this
   item.
3. **BIP-93 full inline corpus (§93.1–.3 + §93.5 + 42 invalid) →
   `ms-codec`.** Today only §93.4 is pinned; the other length-buckets
   rely on `rust-codex32 =0.1.0` not silently drifting on a future
   bump. Local pins close the upstream-drift surface.

## Cycle ROI accounting

Adopting all three high-ROI items + BIP-85 v85.3 + the cross-repo
audit-matrix lift adds ≥ 65 active test cells across three repos at
median cost ~10 minutes per cell. None of the work touches wire
formats, public APIs, or flag surfaces. The cycle's net effect is
larger published-vector surface coverage at no semver risk.

## Methodological notes for future cycles

- The earlier `survey across all BIPs` request that produced this
  report is well-suited to a `general-purpose` agent with explicit
  WebFetch + Grep + Read access. Each BIP's vector publication should
  be verified by URL, not asserted from training data — several BIPs
  publish vectors at sidecar paths not always linked from the
  main `.mediawiki` body (BIP-340's CSV, BIP-341's JSON).
- Sibling-repo audit matrices (mk-codec, md-codec, ms-codec) are
  cross-citation-grade artifacts; the toolkit-only matrix at v0.7.1
  understates constellation coverage by not naming them inline.
  Phase 4 of this cycle fixes that.
- `mk-codec/tests/vectors/v0.1.json` and
  `md-codec/tests/vectors/*.template` are easy to misread as upstream
  BIP vectors at a glance; they are repo-internal goldens for the m1
  / md1 wire formats. Phase 4 audit-matrix prose should call this out
  explicitly so a future surveyor doesn't double-count.
