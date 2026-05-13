# cross-repo SPEC — v0.8.0 published-vector pinning patch cycle

**Version:** 0.8.0 (cross-repo audit cycle; coordinated patch tags
across `mnemonic-toolkit`, `descriptor-mnemonic`, `mnemonic-secret`).
**Date:** 2026-05-13
**Status:** DRAFT — awaiting architect sign-off on this SPEC and the
companion plan before phase work begins.
**Predecessors:** [`SPEC_test_vector_audit_v0_7_1.md`](SPEC_test_vector_audit_v0_7_1.md).
**Audit matrices** (this cycle updates each):
- `mnemonic-toolkit/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`
- `mnemonic-secret/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`
- `descriptor-mnemonic/design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`

**Cross-repo coordination:** per CLAUDE.md mirror-invariant. Each repo's
`design/FOLLOWUPS.md` carries a companion entry `bip-vector-adoption-v0_8`
cross-citing the others; the entry closes when the audit-matrix successor
docs land in lockstep.

## §1 Purpose

v0.8.0 is a vectors-only patch cycle that closes the three highest-ROI
gaps surfaced by the post-v0.7.1 cross-repo BIP-vector adoption audit
(`design/agent-reports/v0_8_0-cross-repo-bip-vector-survey.md`, written
during this cycle's brainstorm). No wire-format change; no new
subcommand or flag. The cycle:

1. Adopts upstream BIP-341 `wallet-test-vectors.json` (taproot key-spend
   + script-path tweak / output-key / bech32m address) in
   `descriptor-mnemonic/crates/md-codec/`. Closes the largest unclaimed
   BIP corpus directly load-bearing on m-format constellation
   correctness — md-codec assembles `tr(K, {…})` trees today but the
   only taproot pin upstream-against is BIP-86 key-spend-only.
2. Adopts the full BIP-93 §Test Vectors corpus in
   `mnemonic-secret/crates/ms-codec/`. Today only §93.4 (`leet`,
   256-bit) is byte-pinned; §93.1–.3, §93.5 (long-codex32 512-bit), and
   the invalid-category corpus rely on `rust-codex32 =0.1.0` not
   silently shifting bit-packing semantics on a bump. Local pins close
   the upstream-drift surface.
3. Completes the BIP-39 Trezor English corpus in `mnemonic-toolkit/`.
   The v0.7.1 cycle pinned 6/24; the JSON loader is in place and each
   additional cell is ~6 lines. v0.7.1 §5 listed this as the canonical
   v0.8 carry-over.
4. Opportunistically folds in BIP-85 vector 85.3 (24-word BIP-39
   application) in `mnemonic-toolkit/` — also v0.7.1 §5 carry-over,
   one-cell change, same patch cycle.
5. Lifts the toolkit-only v0.7.1 audit matrix into a cross-repo v0.8.0
   successor that names every published-vector adoption status across
   all four sibling repos. Today the audit matrix is toolkit-only;
   sibling-repo coverage is footnoted but not first-class.

## §2 Coverage deltas

Per repo, post-v0.8.0 vs the v0.7.1 baseline. Counts are individual
published vectors (not test functions).

| Repo | Spec | v0.7.1 covered | v0.8.0 target | Delta |
|---|---|---|---|---|
| `mnemonic-toolkit` | BIP-39 (Trezor) | 6 / 24 | 24 / 24 | +18 (already shipped pre-cycle in commit `85694b2`; v0.7.1 §5 carry-over already closed by `feat(v0.8-phase-8)`) |
| `mnemonic-toolkit` | BIP-85 | 7 / 9 | 8 / 9 | +1 (v85.3) |
| `ms-codec` | BIP-93 valid | 1 / 5 (§93.4 only) | 5 / 5 | +4 |
| `ms-codec` | BIP-93 invalid | 0 / 64 | 64 / 64 | +64 |
| `md-codec` | BIP-341 `scriptPubKey` | 0 / 7 | 7 / 7 | +7 |

Counts deterministically verified at Phase 0 close via
`gh api repos/bitcoin/bips/contents/<file>` against the live
upstream snapshots (not WebFetch — summarizer counts are
unreliable):

- BIP-341 `wallet-test-vectors.json` `scriptPubKey` array length: **7**.
  The companion `keyPathSpending` array (length 1, signing-flow vector)
  is OUT-OF-SCOPE-PER-LAYER — md-codec exposes no Schnorr signing
  surface; covering it would require a new surface and is filed as
  FOLLOWUP `bip341-keypath-signing-vector-coverage`.
- BIP-93 §Invalid test vectors `<code>`-tagged bullet count: **64**
  (truncated/mixed-case HRPs + bad-checksum + length-violation
  variants). The v0.7.1 ms-codec audit matrix's footnote of "42
  strings" was based on an earlier BIP snapshot or count; the live
  count rules.

Net new active cells across the cycle: ≥ 94 (18 + 1 + 4 + 64 + 7).
Per-phase test counts will be reported at phase close. No cells flip
OUT-OF-SCOPE.

## §3 Out-of-scope (continuing v0.7.1 classifications)

The following remain OUT-OF-SCOPE for this cycle; v0.7.1's rationale
carries forward unchanged:

- **BIP-32 TV5** (invalid extended keys) — `bitcoin v0.32` parse-time.
- **BIP-38 ENCRYPT EC-multiplied** — separate v0.8 feature-FOLLOWUP
  `bip38-ec-multiplied-encrypt-mode-support`, not vector adoption.
- **BIP-39 Japanese vectors** — ms-codec is English-only at v0.8.x;
  Japanese wordlist out-of-scope-per-product (filed as new FOLLOWUP
  `bip39-japanese-wordlist-support`).
- **BIP-340 `test-vectors.csv`** — no signing surface in the
  constellation; filed as FOLLOWUP `bip340-schnorr-signing-surface-evaluation`
  for explicit closure.
- **BIP-379** §Test Vectors — upstream-TBD; blocked.
- **BIP-380** key-expr vectors (45 / 46) — `rust-miniscript` surface
  (continues v0.7.1 classification).
- **BIP-388 byte-exact spec xpub round-trip** — spec publishes xpubs
  without underlying seed; structurally impossible (continues v0.7.1).

## §4 Phase structure (cross-ref to plan)

Detailed phase-by-phase TDD breakdown lives in
`/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. High-level
shape:

- **Phase 0** — SPEC + plan landed; architect-reviewed; 0C/0I gate.
- **Phase 1** — BIP-341 → `md-codec`. RED-first; load
  `wallet-test-vectors.json`; derive output key + bech32m address per
  test; assert against spec-expected; architect-review loop until
  0C/0I; persist phase report.
- **Phase 2** — BIP-93 inline corpus → `ms-codec`. RED-first; pin 5
  valid (§93.1–.5) + 42 invalid; architect-review loop; persist phase
  report.
- **Phase 3** — BIP-39 Trezor English fill (18 cells) + BIP-85 v85.3
  → `mnemonic-toolkit`. RED-first; loader already in place;
  architect-review loop; persist phase report.
- **Phase 4** — Audit matrix v0.8.0 successor doc, cross-repo,
  authored at `<each-repo>/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md`.
  v0.7.1 matrices marked SUPERSEDED with a forward-pointer comment.
  Architect-review loop on the cross-repo doc shape.
- **Phase E** — Release rollup. Patch tags coordinated across repos:
  `mnemonic-toolkit-v0.X.Y+1`, `descriptor-mnemonic-md-codec-v0.X.Y+1`,
  `mnemonic-secret-ms-codec-v0.X.Y+1`. FOLLOWUPS companion entries
  closed in lockstep. (Exact version bumps decided at Phase E gate
  based on each repo's existing semver discipline.)

## §5 Cross-repo coordination

Per CLAUDE.md mirror-invariant. Each repo's `design/FOLLOWUPS.md`
gets a new `bip-vector-adoption-v0_8` entry. The entry body:

> **Companion:** Cycle SPEC at
> `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`;
> cycle plan at `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`.
>
> v0.8.0 vectors-only patch cycle. This repo's phase: [Phase 1 / 2
> / 3]. Closes when the cycle's audit-matrix successor doc lands
> in this repo at `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md`
> and the patch tag is cut.

`mnemonic-key` is OUT-OF-SCOPE for v0.8.0 — no new gap; mk-codec's
v0.7.1 matrix carries the only relevant coverage and continues to
delegate xpub-format / BIP-32 derivation to `bitcoin v0.32`. The
`bip-vector-adoption-v0_8` entry in mnemonic-key reads: *"no scope
for this cycle; included for cross-repo audit symmetry."*

## §6 Acceptance gates

Cycle is shippable when all six hold:

1. All four phase-1-through-4 reports persist at
   `<repo>/design/agent-reports/v0_8_0-phase-{1,2,3,4}-*.md` at
   0C/0I.
2. Pre-existing cell counts in each touched repo: monotonically
   increase. No regression in any test cell that was COVERED at
   v0.7.1.
3. `cargo test --workspace` green in each of the three touched
   repos.
4. `cargo clippy --workspace --all-targets -- -D warnings` clean
   in each of the three touched repos.
5. Each repo's `v0_8_0-bip-test-vector-audit-matrix.md` exists and
   passes self-consistency check (every spec-published vector
   tagged COVERED / MISSING / OUT-OF-SCOPE-PER-{USER,SPEC}; no
   unclassified entries).
6. Cross-repo `FOLLOWUPS.md` mirror entries land in the four
   sibling repos (toolkit, md, ms, mk) in lockstep.

## §7 Cross-refs

- v0.7.1 predecessor SPEC: [`SPEC_test_vector_audit_v0_7_1.md`](SPEC_test_vector_audit_v0_7_1.md).
- v0.7.1 audit matrices (toolkit, ms-codec, md-codec, mk-codec) at each
  repo's `design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`.
- BIP sources cited per-phase in each phase's RED-test fixtures.
- Plan: `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md` (this
  cycle).
- Survey precursor: `design/agent-reports/v0_8_0-cross-repo-bip-vector-survey.md`
  (to be written as Phase 0 deliverable; captures the audit that
  surfaced these gaps).
