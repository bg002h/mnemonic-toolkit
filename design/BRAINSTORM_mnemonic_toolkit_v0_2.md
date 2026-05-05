# BRAINSTORM — `mnemonic-toolkit` v0.2

**Date:** 2026-05-05
**Status:** Round 1 — proposed locks pending architect review
**Plan-mode meta-plan:** `/home/bcg/.claude/plans/can-we-work-on-polished-steele.md`

## Context

v0.2 of `mnemonic-toolkit` adds 5 features atop the shipped v0.1 single-sig surface:

1. Multisig templates
2. `--account` flag (non-zero account)
3. `--xpub`-input multisig (multiple cosigners + threshold for watch-only)
4. `--privacy-preserving` (mk1 cards with `origin_fingerprint = None`)
5. `--self-check` (toolkit emits + immediately decodes/verifies internally)

K-of-N share encoding is **deferred** (gates on ms-codec v0.2 work, not in this cycle).

**Pre-brainstorm gates cleared:**
- v0.1 audit completed 2026-05-05 (`design/audit-v0_1-for-v0_2-extension.md`); 0C/2I/4L verdict.
- 2 Important fixups landed in commit `9396a58`: `BundleMismatch.card: &'static str` → `String`; doc-comment staleness.
- 4 LOW findings recorded as v0.2 / v0.2-nice-to-have / v0.1-nice-to-have FOLLOWUPS.
- Sibling-crate readiness verified: md-codec v0.16.1 has multisig tags + `Body::Variable` + `PathDeclPaths::Divergent`; mk-codec v0.2.1 supports `Option<Fingerprint> = None`. No cross-repo blockers.

This BRAINSTORM resurrects the `BRAINSTORM_*.md` artifact format that v0.1 skipped, mirroring sibling precedent (`mnemonic-secret/design/BRAINSTORM_ms_v0_1.md`).

**Status emoji legend:**
- ✅ locked — proposed answer, advances to SPEC unless architect flags
- ⏸ deferred — explicit out-of-scope confirmation
- 🟡 open — needs user review before SPEC-locking

---

## Question chain

### Q1: Multisig template scope

Which BIP-388 multisig descriptor shapes does v0.2 emit?

| | Option | Implication |
|---|---|---|
| A | `wsh(sortedmulti(...))` only | Minimal surface; covers ~80% of modern multisig wallets |
| B | All segwit + nested-segwit: `wsh-multi`, `wsh-sortedmulti`, `sh(wsh-sortedmulti)`, `sh(wsh-multi)` | Covers Coldcard / Ledger / BitBox legacy + modern |
| C | All BIP-388 multisig: B above + `tr-multi_a` + `tr-sortedmulti_a` (taproot) | Full coverage; matches md-codec's tag set |
| D | User-supplied descriptor (passthrough) | Out of scope per Q11 (v0.3) |

**Lock: Option C** — full coverage of pure BIP-388 multisig wrappers.

Rationale: md-codec already implements all four multisig tags (`Tag::{Multi, SortedMulti, MultiA, SortedMultiA}`) at the pinned v0.16.1. Excluding taproot-multisig would force a v0.3 expansion within months as taproot multisig adoption ripens. Including it now keeps the SPEC closure-set complete and the test matrix can omit taproot cells if cosigner-side tooling is not yet available; the descriptor format will work regardless.

Six new `CliTemplate` enum variants:
- `WshMulti`, `WshSortedMulti`, `ShWshMulti`, `ShWshSortedMulti` (segwit family)
- `TrMultiA`, `TrSortedMultiA` (taproot family)

**Scope clarification (resolves L2 from r1 review):** "BIP-388 multisig templates" means the *pure* multi/sortedmulti wrapper family — `wsh(multi(k, @0, ..., @n-1))` and equivalents. Hybrid scripts mixing multisig with miniscript hash-locks or other branches (e.g., `wsh(or_d(multi(...), and_v(v:pkh(...), sha256(...))))`) are out of scope and require user-supplied descriptor passthrough (deferred to v0.3 per Q11).

**Hardware-wallet caveat (resolves L1 from r1 review):** taproot multisig (`tr(multi_a/sortedmulti_a)`) signing-side support is nascent as of v0.2. The §7 engraving guidance MUST warn users to verify their signing device supports `multi_a`/`sortedmulti_a` before engraving — emitting a v0.2 taproot-multisig bundle for a wallet whose hardware can't sign it produces an unusable backup.

Status: **✅ locked**

---

### Q2: Multisig path family

BIP-48 vs BIP-87 vs free-form for the cosigner derivation paths?

| | Option | Implication |
|---|---|---|
| A | BIP-48 only — `m/48'/<coin>'/<account>'/<script_type>'` (script_type=1' for sh-wsh, 2' for wsh, 3' for tr) | Common in Ledger / BitBox |
| B | BIP-87 only — `m/87'/<coin>'/<account>'` (script-type-agnostic; wallet decides based on descriptor wrapper) | Modern simplification (Coldcard / Sparrow / Specter) |
| C | Both with `--multisig-path-family <bip48\|bip87>` flag (default `bip87`) | User picks per-bundle; both decoders interop |
| D | Free-form per-cosigner via `--cosigner=<xpub>:<fp>:<path>` only | Defers BIP-48/BIP-87 standardization to user input |

**Lock: Option C with default `bip87`**.

Rationale: forcing one path family would alienate half of the multisig hardware ecosystem. Both families derive the same xpubs at the same indices for the same seed, so the choice only affects the path component encoded into mk1's `origin_path` and md1's path declaration. Default `bip87` because it's the modern simplification with growing adoption; explicit `--multisig-path-family bip48` supports legacy hardware. Free-form per-cosigner paths are still possible via Option D's flag shape (Q5 below); this answer locks the *default* family used when paths aren't explicit.

Status: **✅ locked**

---

### Q3: Threshold range cap

What `k`-of-`n` ranges are valid?

| | Option | Implication |
|---|---|---|
| A | `1 <= k <= 15`, `n <= 15` | BIP-388's 16-bit threshold field; conservative |
| B | `1 <= k <= 32`, `n <= 32` | md-codec's `KeyCountOutOfRange` allows up to 32 |
| C | `2 <= k`, exclude `1-of-N` | Rejects degenerate single-sig-equivalent |
| D | `1 <= k <= n`, `1 <= n <= 16` | Match BIP-388's nominal 16-cosigner cap |

**Lock: Option D** — `1 <= k <= n <= 16`.

Rationale: BIP-388's tooling ecosystem (Coldcard, Sparrow, Ledger, BitBox) caps at 16 cosigners. md-codec's higher cap (32) is technically encodable but no signing wallets accept it. Allowing `k=1` (1-of-N "any cosigner can spend") is occasionally useful for recovery wallets and shouldn't be rejected. Allowing `n=1` is degenerate but harmless; reject `n=0` and `k>n` as `BadInput`.

Status: **✅ locked**

---

### Q4: `--account` semantics for multisig

How does `--account` interact with multiple cosigners?

| | Option | Implication |
|---|---|---|
| A | Single account-int for all cosigners (BIP-87 style) | Simple flag; all derivations share `account=N` |
| B | Per-cosigner accounts only via cosigner-spec (BIP-48 style) | No top-level `--account` flag; cosigner shape carries it |
| C | Hybrid: `--account <N>` sets default for all; cosigner-spec can override per-cosigner | Most flexible |

**Lock: Option C**.

Rationale: in full mode (single seed phrase, toolkit derives all cosigners) all derivations share the user's single account number, so `--account` works naturally as a flag. In watch-only multisig mode (cosigners supply their own xpubs), each cosigner already has their own derivation path baked into the supplied xpub; the `--account` flag becomes informational rather than determinative (it's encoded into the md1 path declaration but doesn't affect xpub derivation since the xpubs are pre-derived). The cosigner spec (Q5) carries the per-cosigner path explicitly. `--account` defaults to 0 for backwards-compat with v0.1.

Status: **✅ locked**

---

### Q5: `--xpub`-input multisig flag shape

How does the user supply multiple cosigners on the CLI?

| | Option | Implication |
|---|---|---|
| A | Repeatable `--xpub <X>` + repeatable `--master-fingerprint <FP>` (positional pairing) | Fragile — clap doesn't enforce ordering correlation |
| B | Repeatable `--cosigner <X>:<FP>:<path>` (compound flag, colon-delimited) | Self-pairing; `<path>` optional with sensible default |
| C | `--cosigners-file <path>` (JSON file with array of `{xpub, fp, path}`) | Scripted/bulk mode; ergonomic for >3 cosigners |
| D | All three above | Maximum flexibility but lots of CLI surface |

**Lock: Option B as canonical, Option C as bulk/scripted form. Reject Option A.**

Rationale: positional pairing of repeatable flags is fragile (clap silently allows `--xpub X1 --xpub X2 --master-fingerprint F1 --master-fingerprint F2` and you have to assume index-correlation; user errors are hard to detect). The compound `--cosigner <xpub>:<fingerprint>:<path>` flag self-documents per-cosigner relationships. `<path>` is optional with default per-cosigner = `m/<purpose>'/<coin>'/<account>'/[<script_type>']` based on the multisig template + path family from Q2.

**Path precedence rule (resolves I3 from r1 review):** when a cosigner spec carries an explicit `<path>`, that path wins over the family default; `--multisig-path-family` is ignored for that cosigner and honored for any cosigner whose `<path>` is omitted. This rule is enforced per-cosigner, not bundle-wide — mixed defaults and explicit paths in one bundle are valid.

Colon delimiter is unambiguous: base58check (xpubs) uses an alphabet without `:`, fingerprints are 8 hex chars (no `:`), and BIP-32 paths use `/` and `'` (no `:`). Splitting on the first two `:` parses cleanly.

For >3 cosigners, the JSON file form (`--cosigners-file <path>`) avoids long command lines. Format:
```json
[
  {"xpub": "xpub6...", "master_fingerprint": "deadbeef", "path": "m/87'/0'/0'"},
  {"xpub": "xpub6...", "master_fingerprint": "cafef00d", "path": "m/87'/0'/0'"},
  {"xpub": "xpub6...", "master_fingerprint": "1234abcd", "path": "m/87'/0'/0'"}
]
```

`--cosigner` and `--cosigners-file` are mutually exclusive (mode-violation if both supplied).

Status: **✅ locked**

---

### Q6: `--privacy-preserving` granularity

Per-card, per-cosigner, or whole-bundle?

| | Option | Implication |
|---|---|---|
| A | Whole-bundle (all-or-none): single flag, all mk1 cards omit fingerprint | Simple; matches the typical use case |
| B | Per-cosigner via cosigner spec: `--cosigner=<xpub>:<fp_or_NONE>:<path>` | Awkward; mixes orthogonal concerns |
| C | Per-card list: `--privacy-cosigners=0,2` (list of cosigner indices to suppress) | Flexible; rare use case |

**Lock: Option A** (whole-bundle).

Rationale: privacy-preserving mode is typically used for an entire wallet (a coordinator shipping a redacted backup-set to a remote signer; or producing an "anonymous" mk1 set that doesn't link to the master fingerprint). Per-cosigner granularity has no known concrete use case and adds CLI surface. If a v0.3+ use case emerges, the flag can be extended (e.g., `--privacy-preserving cosigner-2` or `--privacy-cosigners=2`).

`--privacy-preserving` is a boolean flag (no argument); when set, all emitted mk1 cards have `origin_fingerprint = None`. mk-codec already supports this end-to-end.

Status: **✅ locked**

---

### Q7: `--self-check` failure semantics

What happens when the toolkit's internal verify-pass detects a bundle mismatch?

| | Option | Implication |
|---|---|---|
| A | New exit code 5 (non-breaking — SPEC §6.1 ends at 4/64) | Distinguishes self-check failure from external verify-bundle failure |
| B | Reuse exit 4 (`BundleMismatch`) | Semantically identical; self-check IS a bundle-mismatch detector |
| C | Warn-only (always exits 0 if synthesis succeeded) | Diagnostic but doesn't fail the build |

**Lock: Option B** (reuse exit 4 `BundleMismatch`).

Rationale: a self-check failure means "I emitted X but cannot decode/verify X back" — that's exactly what `BundleMismatch` means. Adding a new exit code 5 just to distinguish "internal vs external" mismatch source is exit-code inflation. The error's `card` field can carry diagnostic context like `"self-check[mk1]"` (now a `String` per Phase 0 fixup) so users can distinguish self-check failures from external `verify-bundle` failures by inspecting the message. Option C (warn-only) defeats the safety purpose of `--self-check`.

When `--self-check` is set, after `synthesize_*` returns successfully, the toolkit invokes the same verify-bundle 9-check logic; any check returning `result: fail` triggers `Err(BundleMismatch{card, message})` from `bundle::run`, exiting 4. (Implementation: factor verify-bundle's check logic into a reusable lib helper — function name and signature deferred to the SPEC / implementation plan; resolves N2 from r1 review.)

Status: **✅ locked**

---

### Q8: Sortedmulti vs unsorted as default

How does the user specify whether keys are lexicographically sorted at the script level?

| | Option | Implication |
|---|---|---|
| A | `--sorted` flag default-on (sortedmulti for safety; users opt-out via `--unsorted`) | Less foot-gun; matches BIP-87 / modern hardware default |
| B | `--sorted` flag default-off (multi by default; users opt-in) | Matches some legacy tooling defaults |
| C | Two separate templates per multisig variant (`wsh-multi` vs `wsh-sortedmulti`) | Explicit; user picks template; orthogonal to flag-shape |

**Lock: Option C** — two separate `CliTemplate` variants per multisig family.

Rationale: consistent with the v0.1 template-selection mental model (one `--template` flag chooses the descriptor shape). Avoids an additional axis flag (`--sorted`). The 6 multisig templates from Q1 (`WshMulti`, `WshSortedMulti`, etc.) make sortedness explicit at the template level. Users who don't know the difference can pick `WshSortedMulti` for safety; users who need ordering preservation pick `WshMulti`.

Status: **✅ locked**

---

### Q9: v0.1 backwards compatibility

Are v0.1 single-sig invocations wire-bit-identical under a v0.2 binary?

| | Option | Implication |
|---|---|---|
| A | Wire-bit-identical: same string output AND same JSON structure | Strict — bumps `schema_version` only on multisig usage |
| B | Schema-version bump unconditionally: `BundleJson.schema_version` = `"2"` always | Old consumers must update parser |
| C | Hybrid: wire bits identical, JSON shape evolves with new optional fields | Single-sig invocations produce a JSON superset of v0.1; consumers ignoring unknown fields unaffected |

**Lock: Option C** — hybrid, with explicit scoping.

**Scope of "wire-bit-identical":** the encoded card strings (ms1, mk1, md1) emitted by v0.2 for any v0.1-compatible single-sig invocation are byte-identical to v0.1's output. v0.1 decoders can consume v0.2-emitted strings unchanged.

**JSON envelope evolves forward-compatibly** (NOT wire-bit-identical with v0.1's JSON):
- `schema_version` bumps from `"1"` to `"2"` unconditionally (every v0.2 invocation, single-sig or multisig).
- New optional fields added: `multisig: { template, threshold, cosigners } | null`, `privacy_preserving: bool` (default false).
- v0.1 fields stay at the same positions in field order.
- Single-sig invocations produce `multisig: null`, `privacy_preserving: false`.

**Fixture corpus disposition:** the v0.1 SHA pin (`81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6`) is retired at v0.2 release. v0.2 ships a new fixture corpus with a new SHA pin reflecting the new JSON envelope shape. The CHANGELOG v0.2.0 entry documents the v0.1 pin's retirement explicitly. Consumers ignoring unknown fields work unchanged; strict consumers should guard on `schema_version == "2"` to detect new fields.

**`mk1` JSON field shape (resolves C2 from r1 review):**
- Single-sig invocations: `"mk1": ["mk1q..."]` (flat `Vec<String>`, same shape as v0.1; multi-chunk single-sig still emits multiple strings in this flat array).
- Multisig invocations: `"mk1": [["mk1q...", ...], ["mk1q...", ...], ...]` (nested `Vec<Vec<String>>`, outer = per-cosigner, inner = chunks per that cosigner).
- The shape change is keyed on `multisig != null` in the same envelope. Consumers handling both shapes inspect `multisig` first.

Status: **✅ locked**

---

### Q10: Fixture matrix expansion

v0.1 has 16 cells (4 templates × 4 networks). What's the v0.2 fixture cap?

Naive Cartesian: 10 templates (4 single-sig + 6 multisig) × 4 networks × {2-of-3 threshold} × {account=0, account=5} × {privacy off, privacy on} = 160. Too many.

| | Option | Implication |
|---|---|---|
| A | Mirror v0.1 plus 1 multisig template × 4 networks (16 + 4 = 20) | Minimal new coverage; risks under-testing multisig |
| B | Sparse matrix: ~30-40 representative cells covering each axis but not Cartesian | Balanced — every axis tested but not exhaustively combined |
| C | Strict Cartesian (~160) | Maximum rigor but slow CI |

**Lock: Option B** — sparse matrix, ~50 cells; strictly less than Cartesian (~160).

Cell breakdown (resolves I1 + N1 from r1 review):
- **16 cells** unchanged from v0.1 (single-sig × 4 networks). Regression detection. Note per Q9: encoded card strings unchanged, JSON envelope regenerated with `schema_version: "2"`.
- **24 cells** multisig baseline: 6 multisig templates × 4 networks, all 2-of-3, account=0, privacy off.
- **4 cells** non-zero account: 1 representative multisig template × 4 networks, account=5, privacy off, 2-of-3.
- **4 cells** privacy on: 1 representative multisig template × 4 networks, account=0, privacy on, 2-of-3.
- **2 cells** self-check positive control: 1 single-sig + 1 multisig, mainnet only, --self-check passes.
- **(Phase E discretion) ~2 optional cells** threshold variation: 1-of-2 and 3-of-3 on mainnet bip87-wsh-sortedmulti.

Total: ~50 cells minimum (52 with optional). The v0.1 SHA pin (`81828299...`) retires per Q9; v0.2 release ships a new fixture-corpus SHA pin in CHANGELOG.

Status: **✅ locked**

---

### Q11: Out-of-scope confirmations

These features are **explicitly deferred past v0.2**:

- ⏸ **K-of-N share encoding** (gating on ms-codec v0.2 work). Confirmed via plan-mode AskUserQuestion this session.
- ⏸ **User-supplied descriptor passthrough** (arbitrary descriptors via `--descriptor "wsh(...)"`). Deferred to v0.3+; allows hash-locked descriptors and other miniscript shapes.
- ⏸ **`--output <dir>` flag** (write 3 files per card instead of stdout sections). Deferred to v0.3.
- ⏸ **Recovery flow** (3 strings → wallet artifact / xpriv reconstruction). Deferred to v0.3+.
- ⏸ **Hash-locked descriptors** (sha256, hash160, etc. as multisig leaves). Subset of user-supplied descriptor mode; same deferral.
- ⏸ **Color / interactive prompts**. Never (SPEC §3.3 forbids; engraving workflow is non-interactive).

Status: **⏸ deferred (locked as deferred)**

---

### Q12: `--xpub` (v0.1) vs `--cosigner` / `--cosigners-file` (v0.2) coexistence

How do single-sig and multisig watch-only flag shapes coexist?

| | Option | Implication |
|---|---|---|
| A | `--xpub <X> --master-fingerprint <FP>` → single-sig watch-only; `--cosigner` / `--cosigners-file` → multisig watch-only. Mutually exclusive at clap level. | Backwards-compat preserved; clear mental model |
| B | `--xpub` becomes repeatable in v0.2; `--master-fingerprint` becomes repeatable; positional pairing | Breaking — single-sig invocations would still work but the flag's semantics shift |
| C | Drop `--xpub` in v0.2; require `--cosigner` for all watch-only (1-of-1 = degenerate single-sig) | Breaking; requires migration |

**Lock: Option A** — preserve `--xpub` for single-sig; new `--cosigner` / `--cosigners-file` for multisig; **runtime mode-violation pre-check** (not clap-level conflicts_with).

Rationale: aligns with Q9's wire-bit-identical lock. A v0.1 user invoking `mnemonic bundle --xpub xpub6... --master-fingerprint deadbeef --network mainnet --template bip84` continues to work identically under a v0.2 binary. New multisig users use the new flags.

**Mode-violation enforcement (resolves I2 from r1 review):** clap's `conflicts_with` exits 64 with default usage text, which would violate SPEC §6.6's byte-exact contract. Instead, `cmd::bundle::run` and `cmd::verify_bundle::run` add runtime pre-checks (mirroring v0.1's existing `--passphrase` + `--xpub` pattern in `cmd/bundle.rs:93-99`) that emit `ToolkitError::ModeViolation` with byte-exact §6.6 text and exit 2. The same pattern applies to the `--cosigner` ↔ `--cosigners-file` conflict from Q5 (resolves L3 from r1).

Mode-violation rows added to SPEC §6.6:

> 1. `--xpub` cannot be combined with `--cosigner` or `--cosigners-file`; pick single-sig (`--xpub`) or multisig (`--cosigner`/`--cosigners-file`) but not both.
> 2. `--cosigner` cannot be combined with `--cosigners-file`; supply cosigners via flag-repetition or file, not both.

Both: exit 2, byte-exact text, runtime check (not clap).

Single-sig invocations under v0.2 produce `BundleJson.multisig: null` (per Q9); multisig invocations produce `BundleJson.multisig: { template, threshold, cosigners: [...] }`.

Status: **✅ locked**

---

## Locks summary

All 12 questions ✅ or ⏸ locked. No 🟡 open items pending user review.

Summary table:

| Q | Topic | Lock |
|---|---|---|
| Q1 | Multisig template scope | All BIP-388 multisig (6 templates incl. taproot) |
| Q2 | Multisig path family | BIP-48 + BIP-87 with `--multisig-path-family` flag, default `bip87` |
| Q3 | Threshold cap | `1 <= k <= n <= 16` |
| Q4 | `--account` semantics | Hybrid: `--account <N>` global default; cosigner-spec per-cosigner override |
| Q5 | Multisig flag shape | `--cosigner <xpub>:<fp>:<path>` (canonical) + `--cosigners-file <path>` (bulk) |
| Q6 | Privacy granularity | Whole-bundle (`--privacy-preserving` boolean) |
| Q7 | Self-check failure | Reuse exit 4 (`BundleMismatch`); card identifier like `"self-check[mk1]"` |
| Q8 | Sortedmulti default | Separate templates (`WshMulti` vs `WshSortedMulti`) |
| Q9 | v0.1 backwards compat | Wire-bit-identical; JSON schema_version bumps to `"2"` with new optional fields |
| Q10 | Fixture matrix | Sparse ~50 cells; v0.1 16 cells regenerated with schema_version=2 (encoded strings unchanged) + multisig baseline + axis additions |
| Q11 | Out-of-scope | K-of-N + user-supplied descriptor + --output + recovery + hash-locks all deferred |
| Q12 | Flag coexistence | `--xpub` for single-sig; `--cosigner`/`--cosigners-file` for multisig; mutually exclusive |

## Implications for SPEC drafting

The locks above translate to specific SPEC §1–§11 mutations:

- **§1 (Scope):** add multisig + non-zero-account + privacy + self-check; explicit deferral list per Q11.
- **§2.1 / §2.2:** new flags (`--cosigner`, `--cosigners-file`, `--multisig-path-family`, `--account`, `--privacy-preserving`, `--self-check`).
- **§4.1 / §4.5 / §4.6:** multi-cosigner derivation; per-cosigner mk1; `Body::Variable{k, children}` for multisig; `PathDeclPaths::Divergent` for per-cosigner paths.
- **§5.2:** engraving card stderr adds threshold + cosigner count rows for multisig mode.
- **§5.3:** `BundleJson` adds `multisig: { template, threshold, cosigners } | null` and `privacy_preserving: bool` fields; `schema_version` bumps to `"2"` unconditionally. `mk1` field shape per Q9: single-sig keeps flat `Vec<String>` (v0.1-shape preserved); multisig uses nested `Vec<Vec<String>>` (outer = per-cosigner, inner = chunks). Consumers branch on `multisig != null`.
- **§5.4:** `VerifyBundleJson` 9-check array adapts to multisig (per-cosigner xpub/fp/path matches; stub_linkage becomes per-cosigner).
- **§6:** new mode-violation rows for the new flag combinations; new `ToolkitError` variants if needed (multisig threshold violation, cosigner-count mismatch).
- **§6.6:** byte-exact text for the new mode-violation rows (per Q12 etc.).
- **§7.1 / §7.3:** multi-cosigner engraving workflow; coordinator-vs-cosigner persona; privacy-preserving mode interaction with passphrase.
- **§8:** shrinks (5 features ship); deferral list per Q11 stays.
- **§9.4 (NEW):** architect review closure for v0.2.
- **§11:** revision history with v0.2.0 entry.

## Implications for IMPLEMENTATION_PLAN drafting

5 phases per the meta-plan:

- **Phase A** — `--account` thread-through (LOW complexity, isolated, lands first for verifiability).
- **Phase B** — Foundation expansion for multisig + privacy + self-check scaffolding.
- **Phase C** — Synthesis expansion (HIGH complexity; the multi-cosigner reshape).
- **Phase D** — Command modules (BundleArgs / VerifyBundleArgs new flags; mode dispatch).
- **Phase E** — Integration tests + release prep.

Pre-SPEC sibling-API spike (Phase 1.5 of meta-plan) validates the assumption that md-codec multisig + `PathDeclPaths::Divergent` + mk-codec privacy mode round-trip correctly before SPEC drafting locks the wire shapes.

## Revision history

- **r1 (2026-05-05):** initial draft. All 12 questions ✅/⏸ locked; pending architect review.
- **r2 (2026-05-05):** integrated architect-r1 findings (2C / 3I / 4L / 3N).
  - **C1**: Q9 reframed — "wire-bit-identical" scoped to encoded card strings only; JSON envelope evolves with new schema_version; v0.1 fixture corpus SHA pin retires.
  - **C2**: Q9 resolves the `mk1` JSON field shape: flat `Vec<String>` for single-sig, nested `Vec<Vec<String>>` for multisig (consumers branch on `multisig != null`).
  - **I1**: Q10 cell counts corrected to ~50 throughout; summary table updated.
  - **I2**: Q12 mode-violation enforcement is runtime pre-check (exit 2, byte-exact §6.6 text), not clap `conflicts_with`. L3 (`--cosigner` ↔ `--cosigners-file`) folded into the same pattern.
  - **I3**: Q5 path-precedence rule added (explicit cosigner path overrides family default).
  - **L1**: Q1 hardware-wallet caveat for taproot multisig added; §7 must warn users to verify signing-side support.
  - **L2**: Q1 scope clarification: BIP-388 multisig = pure multi/sortedmulti wrappers, not hybrid miniscripts.
  - **L4**: §5.3 implication updated with resolved JSON shape.
  - **N2**: Q7's `synthesize::self_check_bundle` function-name reference moved to "deferred to SPEC/plan."
  - **N3**: Q8 rationale "matches" → "consistent with."
