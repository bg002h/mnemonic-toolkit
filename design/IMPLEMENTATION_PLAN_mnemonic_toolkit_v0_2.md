# IMPLEMENTATION_PLAN — `mnemonic-toolkit` v0.2

**Status:** Round 1 — pending architect review
**Date:** 2026-05-05
**SPEC:** `SPEC_mnemonic_toolkit_v0_2.md` (architect-converged at r3; 0C/0I)
**Brainstorm:** `BRAINSTORM_mnemonic_toolkit_v0_2.md` (architect-converged at r2; 0C/0I)
**Pre-SPEC spike:** `agent-reports/spike-toolkit-v0_2-pre-spec.md` (GATE GREEN)
**Audit:** `audit-v0_1-for-v0_2-extension.md` (Phase 0 closure complete; commit `9396a58`)
**v0.1 baseline:** HEAD `47bb44c` (after Phase 0 + BRAINSTORM + spike + SPEC commits)

> **For agentic workers:** REQUIRED SUB-SKILL: `superpowers:subagent-driven-development` to execute this plan task-by-task. Per-phase reviewer-loop until 0 critical / 0 important findings. Per-phase implementation review reports persist to `design/agent-reports/phase-<X>-<name>-review-rN.md`.

## Goal

Ship `mnemonic-toolkit-v0.2.0`: 5 new features atop v0.1 single-sig (multisig templates, `--account`, `--xpub`-input multisig, `--privacy-preserving`, `--self-check`) with byte-bit-identical wire compatibility for all v0.1 invocations. `cargo publish` remains gated on sibling crates landing on crates.io.

## Phase decomposition

5 phases ordered by complexity (lowest first; isolation enables bisection):

| Phase | Scope | Complexity |
|---|---|---|
| **A** | `--account` thread-through (single-sig path; isolated) | LOW |
| **B** | Multisig templates + privacy + self-check scaffolding (no synthesis logic yet) | MEDIUM |
| **C** | Synthesis expansion (multi-cosigner Bundle; build_descriptor for multisig; per-cosigner mk1; self-check helper) | HIGH |
| **D** | Command modules (BundleArgs / VerifyBundleArgs new flags; mode dispatch; --self-check inline) | MEDIUM |
| **E** | Integration tests + release prep (~50-cell fixture matrix; v0.2.0 tag) | MEDIUM |

Per-phase commit cadence: feature commit + per-round fixup commits + per-phase review verdict commit (mirroring v0.1 rhythm). Per-phase opus review iterates r1 → rN until 0C/0I (max cap r4 per `feedback_iterative_review_every_phase`); critical/important findings fixed inline; low/nit deferred to `design/FOLLOWUPS.md` at v0.2-nice-to-have tier.

---

## Phase A — `--account` thread-through (LOW complexity, isolated)

**Goal:** add a `--account: u32` flag that defaults to `0` (preserves v0.1 wire bits) and threads through to BIP-32 derivation + md1 path encoding. Ships verifiable in isolation BEFORE multisig adds complexity.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/template.rs` — `origin_path_str(network, account)`, `derivation_path(network, account)`, `md_origin_path(network, account)` all accept `account: u32`.
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs` — `BundleArgs.account: u32` (default 0); pass to derive + synthesize.
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — same flag + same passthrough.
- Modify: `crates/mnemonic-toolkit/src/synthesize.rs` — `build_descriptor` and `synthesize_*` accept `account` (for path construction in md1).
- Modify: `crates/mnemonic-toolkit/src/derive.rs` — no signature change; `derive_full` already takes a `template` which now carries the account-aware path.
- Modify: `crates/mnemonic-toolkit/src/format.rs` — `BundleJson.account` already exists at u32; just stop hardcoding 0 in the construction site.

**Tasks:**
- **A.1** — extend `template.rs::origin_path_str(network)` → `origin_path_str(network, account)`; same for `derivation_path` and `md_origin_path`. Update existing tests to pass `account: 0`. New unit test exercising `account: 5`.
- **A.2** — thread `account: u32` parameter through `derive::derive_full`, `synthesize::build_descriptor`, `synthesize_full`, `synthesize_watch_only`. Update existing v0.1 tests with explicit `account: 0` (regression).
- **A.3** — `BundleArgs` and `VerifyBundleArgs` add `#[arg(long, default_value = "0")] pub account: u32`. Pass through.
- **A.4** — Phase A commit: explicitly stage paths; test `cargo test -p mnemonic-toolkit && cargo clippy --all-targets -- -D warnings && cargo fmt --check`. **Wire-bit-identical regression** (resolves L-1 from r1 review): run the regression-checklist script (see appendix below) over all 16 v0.1 single-sig cells; the diff against v0.1 fixtures' encoded ms1/mk1/md1 lines (filtered by HRP prefix) MUST be empty. Phase A does not advance until all 16 cells diff-clean.

**Per-phase review (rA1 → rAN):** opus reviewer-loop on the Phase A commit; persist to `design/agent-reports/phase-A-account-review-r1.md`. Verify that the v0.1 fixtures still regenerate byte-identical (Q9 closure; CHANGELOG records this).

---

## Phase B — Foundation: multisig templates + privacy + self-check scaffolding (MEDIUM)

**Goal:** add the v0.2 enum variants, type-shape changes, error variants, and helper signatures WITHOUT yet wiring them into synthesis. After Phase B: code compiles + tests pass + clippy clean, but multisig invocations still error out (intentionally — Phase C wires synthesis).

**Files:**
- Modify: `crates/mnemonic-toolkit/src/template.rs` — extend `CliTemplate` enum with 6 multisig variants (`WshMulti`, `WshSortedMulti`, `ShWshMulti`, `ShWshSortedMulti`, `TrMultiA`, `TrSortedMultiA`); `wrapper_node()` adds match arms for the multisig templates per SPEC §4.6 + spike memo (`Body::Variable { k, children: <N PkK leaves> }` inside the wrapper).
- Modify: `crates/mnemonic-toolkit/src/network.rs` — no changes; multisig path families are template-orthogonal.
- Modify: `crates/mnemonic-toolkit/src/error.rs` — add `MultisigConfig`, `CosignerSpec`, `CosignersFile` variants; `exit_code()` arms (all → 1); `friendly_multisig` wrapper function. New `mode_text::*` consts for the 8 new §6.6 mode-violation rows.
- Modify: `crates/mnemonic-toolkit/src/format.rs` — `BundleJson` adds optional `multisig: Option<MultisigInfo>`, `privacy_preserving: bool` fields; `MultisigInfo { template, threshold, cosigner_count, path_family, cosigners: Vec<CosignerEntry> }`; new `EngravingMode::FullMultisig` and `EngravingMode::WatchOnlyMultisig` variants; `engraving_card` composer extends with multisig stanzas; `chunk_set_id_extract(s: &str) -> Option<u32>` helper for Phase C / D. Add `MkField` enum (`#[serde(untagged)]` Single(Vec<String>) | Multi(Vec<Vec<String>>)) for the discriminated-union `mk1` field per SPEC §5.3.
- Modify: `crates/mnemonic-toolkit/src/parse.rs` — add `parse_cosigner_spec(s: &str) -> Result<CosignerSpec, ToolkitError>` (canonical `<xpub>:<fp>:<path>` parser) and `parse_cosigners_file(path: &Path) -> Result<Vec<CosignerSpec>, ToolkitError>`; helpers for multisig-path-family enum.
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs` — `BundleArgs` declares the new flags (`--cosigner`, `--cosigners-file`, `--multisig-path-family`, `--privacy-preserving`, `--self-check`, `--threshold`, `--cosigner-count`) but `bundle::run` still rejects multisig invocations with a "v0.2 multisig synthesis pending" stub error. Mode-violation pre-checks added for the 8 new §6.6 rows.
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — same flag additions + same mode-violation pre-checks; multisig verify path stub-errors.
- Modify: `crates/mnemonic-toolkit/src/synthesize.rs` — `Bundle.mk1: Vec<String>` stays for now (Phase C reshapes it); add `MkField` re-export for format.rs.

**Tasks:**
- **B.1** — extend `CliTemplate` enum with 6 multisig variants. Update `Default` impl if any (none currently — `clap::ValueEnum` doesn't require `Default`). Update `wrapper_node` match arms to construct the multisig wrapper trees per SPEC §4.6 (resolves N-1 from r1 review — explicit nesting):
  - **wsh-multi / wsh-sortedmulti:** `Node { tag: Tag::Wsh, body: Body::Children(vec![Node { tag: Tag::{Multi|SortedMulti}, body: Body::Variable { k, children: <N PkK leaves> } }]) }`
  - **sh-wsh-multi / sh-wsh-sortedmulti:** `Node { tag: Tag::Sh, body: Body::Children(vec![<wsh node above>]) }`
  - **tr-multi-a / tr-sortedmulti-a:** `Node { tag: Tag::Tr, body: Body::Tr { key_index: 0, tree: Some(Box::new(Node { tag: Tag::{MultiA|SortedMultiA}, body: Body::Variable { ... } })) } }` — exact structure per Phase B mini-spike below.
  - Each `<PkK leaf>` is `Node { tag: Tag::PkK, body: Body::KeyArg { index: i } }` for i ∈ 0..N.

  **Mini-spike for taproot multisig** (resolves L-2 from r1 review): write a unit test in `template.rs::wrapper_node()` for `TrSortedMultiA` 2-of-2 that calls `md_codec::chunk::split` + `reassemble` and asserts `is_wallet_policy() == true`. If it fails, file a cross-repo FOLLOWUP at `/scratch/code/shibboleth/descriptor-mnemonic/design/FOLLOWUPS.md` (cross-repo tier) and pause Phase B until resolved. Phase 1.5 spike validated wsh-sortedmulti only; taproot multisig's wrapper composition needs verification before Phase C synthesis wires it.
- **B.2** — add `error.rs` variants + `mode_text::*` consts + `friendly_multisig`.
- **B.3** — add `format.rs` `MultisigInfo`, `CosignerEntry`, `MkField` (discriminated-union with `#[serde(untagged)]`), `EngravingMode::FullMultisig` + `WatchOnlyMultisig`, `chunk_set_id_extract` helper. **Ownership change** (resolves I-1 from r1 review): `BundleJson.mk1: &'a [String]` (v0.1's borrowed slice) becomes `BundleJson.mk1: MkField` (owned). Remove the `'a` lifetime from `BundleJson` if it was the only borrowed field, or keep it scoped to the remaining borrowed fields. `MkField::Single(Vec<String>)` with `#[serde(untagged)]` serializes to `["mk1q..."]` byte-identical to v0.1 — verify with a unit test in this Phase B task before advancing.
- **B.4** — add `parse.rs` `parse_cosigner_spec`, `parse_cosigners_file`, multisig-path-family enum.
- **B.5** — extend `BundleArgs` + `VerifyBundleArgs` clap declarations. Mode-violation pre-checks. Multisig synthesis paths error with "v0.2 multisig synthesis pending Phase C" stub.
- **B.6** — Phase B commit: stage paths; verify `cargo test && cargo clippy && cargo fmt --check` clean; v0.1 single-sig fixtures regenerate byte-identical (regression). Multisig invocations exit with stub error (expected).

**Per-phase review (rB1 → rBN):** persist to `design/agent-reports/phase-B-foundation-review-r1.md`.

---

## Phase C — Synthesis expansion (HIGH complexity)

**Goal:** wire the multisig synthesis paths (full mode + watch-only); implement `--self-check`; implement multi-cosigner mk1 emission; implement md1 multisig descriptor construction with `Body::Variable` + `PathDeclPaths::Divergent`; implement byte-exact engraving-card stderr for multisig.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/synthesize.rs` — **biggest file change**. Reshape `Bundle.mk1` from `Vec<String>` to `MkField` (discriminated union). New `synthesize_multisig_full(seed, account, threshold, n, template, network, path_family) -> Result<Bundle, ToolkitError>`; new `synthesize_multisig_watch_only(cosigners: Vec<CosignerSpec>, threshold, template, network, path_family, account) -> Result<Bundle, ToolkitError>`. Both emit per-cosigner `KeyCard` set + multisig `Descriptor`. New `self_check_bundle(&Bundle, args) -> Result<(), ToolkitError>` helper extracting verify-bundle's 9-check logic (already partially factored in v0.1 `verify_bundle::watch_only_checks`).
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs` — wire multisig synthesis paths from Phase B's stub-error sites; emit byte-exact §5.2 engraving card for multisig modes; emit non-suppressible SELF-MULTISIG WARNING stderr (BEFORE stdout, per SPEC §4.1) when full multisig with `--cosigner-count > 1`. Wire `--self-check` post-synthesis call to `synthesize::self_check_bundle`.
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — wire multisig verify paths. Implement the `--mk1` flat-with-internal-grouping per SPEC §2.2.1 (extract `chunk_set_id` from each chunk via mk-codec helpers; group; pass per-cosigner slices to `mk_codec::decode`). Stub-list mismatch detection (step 5b) → exit 4.
- Modify: `crates/mnemonic-toolkit/src/derive.rs` — extend `derive_full` to optionally derive N cosigner xpubs from one seed at a path-family-selected path (full multisig "self-multisig" mode per SPEC §4.1).

**Tasks:**
- **C.1** — `synthesize::Bundle` reshape: `mk1: MkField`. Update existing single-sig synthesis to wrap in `MkField::Single(Vec<String>)`. Wire `BundleJson.mk1` at the emit site in `bundle::run::emit()`; verify single-sig JSON output is byte-identical to v0.1's flat `["mk1q..."]` (`#[serde(untagged)]` confirmed correct in B.3 unit test).
- **C.2** — `synthesize_multisig_full` end-to-end: derive N xpubs (self-multisig from one seed), build N `KeyCard` instances each with the FULL N-stub list, emit per-cosigner mk1 chunks via `mk_codec::encode_with_chunk_set_id`. **Per-cosigner CSI derivation:** `csi_i = derive_mk1_chunk_set_id(&stubs[cosigner_idx])`, NOT all using `stubs[0]` (would collide). **Self-multisig CSI collision note** (resolves L-3 from r1 review): in self-multisig full mode all N cosigners share the same xpub and the same `policy_id_stubs`, so all N CSIs are identical by construction. This is correct and expected — the SELF-MULTISIG WARNING acknowledges the cards are byte-identical interchangeable copies. Build multisig md1 `Descriptor` with `Body::Variable`. Cross-binding invariant 1 (per SPEC §4.7 delta): all N cards' `policy_id_stubs` equal the descriptor-derived list. Cross-binding invariant 2: `descriptor.is_wallet_policy()`. Both `debug_assert!`.
- **C.3** — `synthesize_multisig_watch_only` mirrors C.2 but takes `Vec<CosignerSpec>` from parsed flags / file. `PathDeclPaths::Divergent` when paths differ across cosigners; `Shared` if identical.
- **C.4** — `self_check_bundle` entry-point in `cmd/bundle.rs` (resolves I-2 from r1 review). Decision: keep `watch_only_checks` in `verify_bundle.rs` as `pub(crate)` (it's already factored at v0.1's `verify_bundle.rs:484`); call it directly from `bundle::run` via `crate::cmd::verify_bundle::watch_only_checks(...)`. New `self_check_bundle(bundle: &Bundle, args: &BundleArgs) -> Result<(), ToolkitError>` lives in `cmd/bundle.rs` (it's a command-layer concern, not a synthesize-lib concern; avoids a layering inversion). Calls produce `Vec<VerifyCheck>`; if any `result: fail`, return `Err(BundleMismatch{ card: format!("self-check[{}]", check.name), message: check.detail })`.
- **C.5** — `cmd/bundle.rs::run` wires multisig paths + self-check post-call; non-suppressible SELF-MULTISIG WARNING emission.
- **C.6** — `cmd/verify_bundle.rs::run` wires multisig verify paths; implement chunk_set_id-based grouping; stub-list mismatch detection. Add per-cosigner depth check at watch-only entry per SPEC §4.5 + r3-L1 carryover.
- **C.7** — `format::engraving_card` extends with multisig stanzas; SELF-MULTISIG and HARDWARE WALLET CAVEAT lines per SPEC §5.2.
- **C.8** — Phase C commit: stage paths; verify multisig synthesis end-to-end via 1-2 `cargo run` smoke tests against the SPEC-specified flag sets.

**Per-phase review (rC1 → rCN):** persist to `design/agent-reports/phase-C-synthesis-review-r1.md`. Critical: the multi-cosigner stub-list invariant + the wire-bit-identical regression on single-sig.

---

## Phase D — Command modules (MEDIUM)

**Goal:** finish the command-layer wiring; ensure all clap flags route correctly; mode-violation pre-checks complete; help text consistent with SPEC §2.3.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs`
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`
- Modify: `crates/mnemonic-toolkit/src/main.rs` — no changes expected (top-level dispatch already routes to `bundle::run` / `verify_bundle::run`).

**Tasks:**
- **D.1** — Audit all clap flag declarations against SPEC §2.1 + §2.2 flag tables; add any missing.
- **D.2** — Audit all mode-violation pre-checks against SPEC §6.6 table; add any missing rows; verify byte-exact text via integration tests (Phase E).
- **D.3** — `--help` output spot-check against SPEC §2.3.
- **D.4** — JSON envelope construction in `emit()`; verify single-sig produces `multisig: null` + flat `mk1`; multisig produces `multisig: { ... }` + nested `mk1`. **`VerifyBundleJson.schema_version`** (resolves L-4 from r1 review): `BundleJson.schema_version` bumps to `"2"` per SPEC §5.3. `VerifyBundleJson.schema_version` ALSO bumps to `"2"` for consistency — the per-cosigner check-name multiplication (3+6N for multisig) is a SPEC-§5.4 schema change that consumers must detect. Both envelopes ride the same version dial.
- **D.5** — `--privacy-preserving` watch-only relaxation of `mk1_fingerprint_match` check (`result: skipped`) wired in `verify_bundle::run`.
- **D.6** — Phase D commit.

**Per-phase review:** persist to `design/agent-reports/phase-D-commands-review-r1.md`.

---

## Phase E — Integration tests + release prep

**Goal:** ~50-cell fixture matrix per SPEC §10 (Q10 brainstorm closure); new `assert_cmd` integration tests covering each new flag combination; CHANGELOG + README updates; `mnemonic-toolkit-v0.2.0` tag.

**Files:**
- Create: `crates/mnemonic-toolkit/tests/vectors/v0_2/` — ~50 fixture files following the v0.1 naming pattern (`<template>-<network>-<account>-<privacy>-<self-check>.txt`).
- Create: `crates/mnemonic-toolkit/tests/cli_bundle_multisig_full.rs` — full multisig per-template-cell parametric.
- Create: `crates/mnemonic-toolkit/tests/cli_bundle_multisig_watch_only.rs` — watch-only multisig with `--cosigner` + `--cosigners-file`.
- Create: `crates/mnemonic-toolkit/tests/cli_verify_bundle_multisig.rs` — round-trip verification.
- Create: `crates/mnemonic-toolkit/tests/cli_account_flag.rs` — `--account` axis tests.
- Create: `crates/mnemonic-toolkit/tests/cli_privacy_preserving.rs` — `--privacy-preserving` axis tests.
- Create: `crates/mnemonic-toolkit/tests/cli_self_check.rs` — `--self-check` positive + (intentional) negative tests.
- Create: `crates/mnemonic-toolkit/tests/cli_mode_violations_v0_2.rs` — each of the 8 new §6.6 rows.
- Modify: `crates/mnemonic-toolkit/Cargo.toml` — bump version `0.1.0` → `0.2.0`.
- Modify: `CHANGELOG.md` — v0.2.0 entry; document v0.1 SHA pin retirement; document new SHA pin.
- Modify: `crates/mnemonic-toolkit/README.md` — multisig example + privacy-preserving example.

**Tasks:**
- **E.1** — fixture generation: regenerate v0.1's 16 single-sig cells (verify ENCODED card strings byte-identical to v0.1 — JSON envelope intentionally differs at `schema_version: "2"` + new `multisig: null` field). Generate ~34 new multisig + axis cells per SPEC §10. Total ~50. **Naming convention** (resolves N-2 from r1 review): use `<template>-<network>-<account>-<privacy>-<self_check>.txt` for v0.2 cells (e.g., `wsh-sortedmulti-mainnet-0-false-false.txt`). **v0.1 regression cells:** retain v0.1's filenames (`bip84-mainnet.txt` etc.) for the 16 single-sig regression set; add a CHANGELOG note that the contents now reflect `schema_version: "2"` JSON envelope while encoded strings inside are byte-identical to v0.1's pin (`81828299...`).
- **E.2** — write integration test files; cover happy paths + mode-violation rows.
- **E.3** — Cargo.toml + CHANGELOG + README updates. **SPEC §9.4 placeholder cleanup** (resolves N-3 from r1 review and SPEC r3 N-1 carryover): edit `design/SPEC_mnemonic_toolkit_v0_2.md` §9.4 to back-fill the actual r1/r2/r3 architect findings summary, replacing the "(populated after r2)" placeholders with concrete content drawn from §11's revision history. This is an editorial task, not a review task — owns to E.3 (release polish), not E.6 (final review).
- **E.4** — full pre-tag verification: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --check`.
- **E.5** — Phase E commit + tag `mnemonic-toolkit-v0.2.0`. (Push gated on user.)
- **E.6** — final review (full v0.2 surface) persisted to `design/agent-reports/phase-E-release-prep-review-r1.md`.

**Per-phase review:** mirrors v0.1 Phase 5 review pattern; persists to disk.

---

## v0.1 → v0.2 wire-bit-identical regression checklist

For each implementation phase, BEFORE merging, run:

```bash
# 1. Regenerate v0.1's 16 single-sig fixtures using v0.2 binary with --account 0
TREZOR='abandon abandon ... art'
for t in bip44 bip49 bip84 bip86; do
  for n in mainnet testnet signet regtest; do
    ./target/debug/mnemonic bundle --phrase "$TREZOR" --network $n --template $t --no-engraving-card > /tmp/v0_2_regen.txt
    # Compare encoded card strings (ms1, mk1, md1 lines starting with their HRPs) byte-identical to v0.1's fixture
    diff <(grep -E '^(ms1|mk1|md1)' /tmp/v0_2_regen.txt) <(grep -E '^(ms1|mk1|md1)' tests/vectors/v0_1/$t-$n.txt)
  done
done
```

The encoded strings MUST be byte-identical. JSON envelope differs (schema_version, new fields) but the bare encoded strings are unchanged per SPEC §9.4.1.

## Cross-repo coordination

No expected sibling-repo pushes at v0.2 release time. The Phase 0 audit + Phase 1.5 spike confirmed all sibling APIs are present at the pinned versions; no cross-repo FOLLOWUPS surfaced. If Phase C surfaces any unexpected sibling-API gap, mirror entries into the affected sibling's `design/FOLLOWUPS.md` per cross-repo convention.

## Carryover items from SPEC review (folded into Phase D + E tasks)

- **SPEC r3 L-1** (`§2.2.2 watch-only verify 4-check enumeration`): Phase D.2 enumerates the 4 substantive checks per SPEC §2.2.2 + §5.4 (skipped slots for entropy/path-rederivation per Q9 closure).
- **SPEC r3 N-1** (`§9.4 placeholder cleanup`): Phase E.3 (editorial release polish; moved from E.6 per PLAN r1 N-3 fix) back-fills §9.4 with the actual r1/r2/r3 architect findings summary, replacing the "(populated after r2)" placeholders.

## Revision history

- **r1 (2026-05-05):** initial v0.2 IMPLEMENTATION_PLAN draft. Architect r1: 0C/2I/4L/3N.
- **r2 (2026-05-05):** integrated architect-r1 findings.
  - **I-1**: Phase B.3 + C.1 explicit `BundleJson.mk1: &'a [String]` → `MkField` ownership change with serde-untagged byte-identical verification.
  - **I-2**: Phase C.4 decided home for `self_check_bundle`: stays in `cmd/bundle.rs`; `watch_only_checks` stays `pub(crate)` in `verify_bundle.rs`.
  - **L-1**: Phase A.4 explicit 16-cell regression-clean criterion before advancing.
  - **L-2**: Phase B.1 mini-spike for taproot multisig wrapper composition; cross-repo FOLLOWUP if it fails.
  - **L-3**: Phase C.2 explicit note that self-multisig CSI collision is correct (all N CSIs identical).
  - **L-4**: Phase D.4 `VerifyBundleJson.schema_version` ALSO bumps to "2".
  - **N-1**: Phase B.1 explicit nesting structure for all 6 multisig variants.
  - **N-2**: Phase E.1 fixture-naming convention: v0.2 cells use `<template>-<network>-<account>-<privacy>-<self_check>.txt`; v0.1 regression cells retain original filenames.
  - **N-3**: SPEC §9.4 placeholder cleanup moved from Phase E.6 (review verdict) to Phase E.3 (editorial release polish).
