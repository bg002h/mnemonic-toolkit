# SPEC — cross-start convergence + standalone-bijection test set

- **Date:** 2026-05-25
- **Source SHA (citations verified against):** `2dc1276` (toolkit `master`)
- **Status:** approved (brainstorm + opus architect blueprint); **R0 architect gate waived by explicit user instruction** — substituting per-cell run-and-triage during implementation.
- **Type:** primarily test coverage — BUT cell A7 surfaced a real product defect (**F3**, below) that was **fixed in-cycle**. The original brainstorm's "no production change" framing no longer holds: the F3 fix to `resolve_slots` (multisig `--multisig-path-family` derivation) ships with this cycle as PATCH v0.37.4. Retroactive architect R0 persisted at `design/agent-reports/v0_37_4-f3-fix-review.md`. Convergence-cell failures are surfaced as findings, not silently patched (F1 documented, F2 test-fix, F3 product-fix).

## Purpose

The m-format constellation's premise is "one secret, expressed four ways." This spec adds the **highest-assurance test set that is complete but not redundant** for two properties the existing suite does not cover:

- **Property A — cross-start convergence.** The same key/policy, entered as a **seed**, an **xpub**, a **wallet descriptor**, or a **wallet file**, must produce **byte-identical `mk1` + `md1`** cards (and the xpub + master-fingerprint they embed).
- **Property B — the two missing standalone bijections.** `xpub → mk1 → xpub` and `descriptor → md1 → descriptor`, byte-identical.

### Scope boundaries (decided — do not widen)
- **`ms1` is excluded from convergence**: only the seed/entropy starts carry entropy; watch-only starts carry `ms1=[""]`. Asserted as the sentinel marker only, never compared across starts.
- Byte-exactness **is** achievable for cards. The semantic-only round-trip caveat (`wallet_import/roundtrip.rs:4-9`) applies to wallet-file→wallet-file export, **not** wallet-file→cards.
- **Out of scope** (already covered or one-way): phrase↔entropy↔ms1↔seedqr, seed-xor, slip39, SLIP-0132 variants, nostr, silent-payment, bip85, minikey, electrum-decrypt, address leaves, wallet-file→wallet-file export.

## Consistency contract (why convergence can hold)
`mk1` binds `xpub + origin(fingerprint,path) + policy stub`; `md1 = split(canonical descriptor)`. Byte-identity across starts holds **iff every start feeds the same `(xpub, fingerprint, path)` and the same canonical descriptor.** Three load-bearing constraints:
1. A bare xpub carries **no master fingerprint** (`bundle.rs` → `Fingerprint::default()` when `@N.fingerprint=` absent). Watch-only starts MUST be fed `@N.fingerprint=` matching the seed's master fp.
2. Multisig template path comes from `--multisig-path-family` (bip87 → `m/87'/0'/0'`; bip48 → `m/48'/0'/0'/2'`). Watch-only/descriptor starts MUST supply `@N.path=` matching the family.
3. Compare **card arrays via `--json`**, not raw stdout (seed start emits an `ms1` section the watch-only starts omit).

## Test matrix — 14 new cells, 2 files

### Property A — `tests/cli_cross_start_convergence.rs` (8 cells)
Anchor = the seed start (the only start with entropy). Each cell asserts byte-identical `mk1` + `md1` across the named starts.

| Cell | Starts compared | Class |
|---|---|---|
| A1 | seed ≡ xpub | single-sig BIP-84 |
| A2 | seed ≡ descriptor | single-sig BIP-84, canonical |
| A4 | seed ≡ wallet-file | bitcoin-core single-sig BIP-84 |
| A5 | seed ≡ xpub ≡ descriptor ≡ wallet-file (transitive 4-way) | single-sig BIP-84 |
| A6 | seed ≡ xpub ≡ descriptor | multisig **BIP-87** wsh-sortedmulti 2-of-3 (`MkField::Multi`) |
| A7 | seed ≡ wallet-file | multisig **BIP-48** BSMS 2-of-3 |
| A8 | descriptor ≡ wallet-file | **non-canonical** `wsh(andor)` watch-only |
| A1-neg | xpub-start *without* `@0.fingerprint=` **≠** seed-start | pins fp is load-bearing (anti-vacuity) |

### Property B — `tests/cli_standalone_bijections.rs` (6 cells)

| Cell | Bijection | Class |
|---|---|---|
| B1 | xpub → mk1 → xpub | single-sig |
| B2 | xpub → mk1 → (xpub, fingerprint, path) | all three `(Mk1,*)` reverse edges (`convert.rs:632-634`) |
| B3 | xpub → mk1 → xpub, per-cosigner | multisig (`MkField::Multi` chunking) |
| B4 | descriptor → md1 → descriptor | canonical single-sig; `md_codec::chunk::reassemble`, `Descriptor: PartialEq` |
| B5 | descriptor → md1 → descriptor | non-canonical `wsh(andor)` |
| B6 | descriptor → md1 → descriptor | multisig 2-of-3; policy-id equality |

### Dropped as already-covered (reuse, do not re-assert)
- **A3** entropy≡phrase — `cli_unified_slot.rs` (extend only if it does not already assert mk1+md1 byte-identity).
- **B7** descriptor→md1 verify-bundle closure — `cli_descriptor_mode.rs:75`.
- **B-neg-1** `convert xpub→mk1` refusal (exit 2) — `cli_convert_refusals.rs:77`.

## Mechanism
**All cells use canonical, hand-enumerated fixtures — no proptest.** These are exact byte-identity claims over a fixed key universe; random keys add runtime, not equivalence classes. (proptest precedent `tests/lib_slip39_roundtrip.rs` is for structural share-recombination variety, which does not apply here.)

## Fixtures (consistency-first; no new on-disk fixtures except one reuse)
- Trezor-24 "abandon×23 art", entropy `0×64`, fp `5436d724`, BIP-84 acct xpub `xpub6CatW…PC7PW6V`, BIP-48 `m/48'/0'/0'/2'` xpub — all already pinned in-repo (`cli_convert_round_trips.rs`, `cli_descriptor_mode.rs:48`, `cli_import_wallet_seed_overlay.rs:30-34`).
- Multisig cosigner keys: **derive in-test** from 3 known seeds via `bitcoin::bip32` (the `cli_bundle_multisig.rs:25-36` pattern) → seed↔xpub consistency provable by construction.
- BSMS blobs (A7, A8): **build in-test** via the `bsms_2line` checksum helper (`cli_import_wallet_seed_overlay.rs:38`).
- A4 wallet file: reuse `tests/fixtures/wallet_import/core-bip84-mainnet.json` **iff** its key matches Trezor-24; otherwise build the blob via `export-wallet --format bitcoin-core` from the Trezor-84 xpub+fp (the `export_core_bip84_single_sig` pattern, `cli_import_wallet_roundtrip.rs:298`) to guarantee consistency. **This consistency check is Phase-1 gating work.**

## Placement & self-containment
- Two new files under `crates/mnemonic-toolkit/tests/`, following `cli_*.rs` + `assert_cmd::cargo_bin("mnemonic")`.
- In-test card decoding uses `md_codec`/`mk_codec` as **library** calls (already crates.io `[dependencies]`; precedent `cli_bundle_watch_only.rs:20`, `cli_bundle_import_json.rs:82`).
- **Fully self-contained**: drives only the `mnemonic` binary; no sibling `md`/`ms`/`mk` binary, no network. Runs under default `cargo test`, **no `#[ignore]`**.

## Findings surfaced during implementation

- **F1 — bitcoin-core splits the `<0;1>` multipath.** A bitcoin-core `importdescriptors` wallet file represents receive/change as **two separate single-path descriptors** (`…/0/*` and `…/1/*`), not the unified `…/<0;1>/*` multipath that the seed/xpub/descriptor template starts produce. Consequently a bitcoin-core wallet file does **not** converge byte-identically with the canonical multipath bundle — `md1` differs because the descriptor shape differs. This is a **format property of bitcoin-core**, not a toolkit bug (BSMS preserves the full `…/<0;1>/*` descriptor on one line and *does* converge). **Resolution (user-approved): honest scoped convergence** — A4 asserts the wallet-file converges with a seed/descriptor start that uses the descriptor the format actually carries (bitcoin-core → the `/0/*` form); A7 (multisig BSMS) converges on the full multipath. Convergence holds whenever both sides share the same canonical descriptor.
- **F2 (test-only, fixed) — mislabeled fixture.** `xpub6CatWdiZiodmU…PC7PW6V` is the **12-word** "abandon…about" seed's bip84 xpub (fp `73c5da0a`, per `cli_import_wallet_roundtrip.rs:287`), but `cli_descriptor_mode.rs:48` comments it as the trezor-**24** seed's. That test passes only because it never cross-checks against the seed's real derivation. Our convergence cells **derive the xpub in-test** from the seed (the `cli_bundle_multisig.rs` pattern), so they are immune. The mislabeled comment in `cli_descriptor_mode.rs:48` is a candidate minor-docs FOLLOWUP, not fixed here.

## Highest-risk cell
**A4/A5** (wallet-file→cards ≡ seed→cards) is the most likely to expose a real divergence: if bitcoin-core import→bundle does not produce byte-identical cards to template-mode seed→bundle, that is a genuine product convergence defect. Phase 1 confirms fixture-key consistency end-to-end; if the cell then fails, the divergence is reported, not papered over.

## Implementation sequence
- **Phase 1 — helpers + fixtures + A4 consistency gate.** Shared helpers (bundle-json card extraction, in-test xpub derivation, `bsms_2line`, envelope-to-tempfile). Inspect `core-bip84-mainnet.json`; pin the consistent A4 path. RED-first per the repo's TDD convention.
- **Phase 2 — Property A** (A1, A2, A4, A5, A6, A7, A8, A1-neg). Run after each; triage any failure (test bug vs product finding).
- **Phase 3 — Property B** (B1–B6).
- **Phase 4 — full-suite verification.** `cargo test -p mnemonic-toolkit` green (or a documented, surfaced finding); `cargo clippy --all-targets -D warnings`. Extend `cli_unified_slot.rs` for A3 only if needed.

No version bump assumed (test-only; land on `master`, like the prior docs PATCH). Confirm ship mode with the user after the suite is green.
