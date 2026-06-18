# PLAN — BCH residue conformance hardening (doc + test, NO-BUMP)

**Date:** 2026-06-18
**Author:** session (autonomous)
**Type:** documentation + test-hardening. **SemVer: NO-BUMP in all repos** (no production code, no wire-format, no CLI surface, no public API change).
**Source SHAs (grep-verified at write time):** toolkit `9e64710` (master), descriptor-mnemonic/md-codec `4ec2110` (main), mnemonic-key/mk-codec `c79aa42` (main), mnemonic-secret/ms-codec `6b28918` (master).

## 0. Motivation

User design question: md1/mk1 share `POLYMOD_INIT = 0x23181b3`; is reusing it wise, should ms1 share it, should all three differ? Resolved analysis:

- `0x23181b3` is the **initial polymod residue** (LFSR seed), NOT the domain-separation residue. Domain separation comes from (a) the **per-HRP target constant** (`*_REGULAR_CONST` / `*_LONG_CONST`, the bech32m-`0x2bc830a3` analogue) and (b) the **HRP**, which is folded into the checksummed input (`hrp_expand(hrp) ‖ data ‖ 0¹³`, md-codec `bch.rs:62-82`).
- The four target constants are already all distinct and the separators that matter:
  - ms1 `MS_REGULAR_CONST = 0x10ce0795c2fd1e62a` (BIP-93 codex32 SECRETSHARE32; init `0x1`)
  - md1 `MD_REGULAR_CONST = 0x0815c07747a3392e7` (NUMS: top-65 `SHA-256("shibbolethnums")`; init `0x23181b3`)
  - mk1 `MK_REGULAR_CONST = 0x1062435f91072fa5c`, `MK_LONG_CONST = 0x41890d7e441cbe97273` (NUMS: top-65/75 `SHA-256("shibbolethnumskey")`; init `0x23181b3`)
- Sharing the init across md/mk is harmless: a fixed init is self-consistent for a self-contained code (create + verify use the same init ⇒ `polymod(valid codeword) == TARGET` length-invariantly, for ANY init — the init term cancels because both create and verify carry the identical `c_I(n+13)` contribution). ms1 is the exception **not because `0x23181b3` is intrinsically length-variant** (it is perfectly length-invariant in a self-contained create+verify, R0-confirmed) but because ms1's verify must agree with the **external** rust-codex32 engine (init `1`, target SECRETSHARE32): the reverted ms-codec v0.2.1 path used a non-codex32 init (`0x23181b3`) **paired with** a target (`0x962958058f2c192a`) empirically lifted from a single 12-word vector, so its hand-rolled verify diverged from codex32's across lengths (`mnemonic-secret/design/BUG_decode_with_correction_length_divergence.md:34-41`; current correct values `mnemonic-secret/.../bch.rs:46-52`). **[Minor-1 folded]**

The recommendations to implement (doc + test only): (1) add the missing md1 NUMS drift-guard; (2) document the shared init's deliberate divergence from codex32's `1` in md + fix mk's misleading comment; (3) add a toolkit-level cross-format separation test (the executable answer to "all three differ"). ms1 conformance is already complete.

## 1. Recon — what already exists (so we don't duplicate)

| Item | ms-codec | md-codec | mk-codec |
|---|---|---|---|
| Target-const drift-guard test | ✅ `tests/bch_drift.rs` + `tests/bch_all_lengths.rs::ms_regular_const_is_secretshare32_packed` | ❌ **MISSING** | ✅ `src/consts.rs::nums_constants_reproduce_from_domain` (+ `nums_string_differs_from_md1`) |
| All-length checksum conformance | ✅ `tests/bch_all_lengths.rs` | round-trip suites (`proptest_roundtrip`, `bip341_wallet_vectors`, `wallet_policy`, `chunking`) exercise create+verify across sizes | round-trip + vector suites |
| `POLYMOD_INIT` doc | ✅ excellent (`bch.rs:48-52`, explains `0x1` + the `0x23181b3` bug) | ❌ **bare** (`bch.rs:19`, no comment) | ⚠️ **misleading** (`string_layer/bch.rs:~182` says "(BIP 93)" — but `0x23181b3` is NOT codex32/BIP-93's init `1`) |

**Empirically verified** (python, this session): `MD_REGULAR_CONST == (u128::from_be_bytes(SHA256("shibbolethnums")[0..16]) >> 63)` → `0x0815c07747a3392e7` ✓. So the prescribed md drift-guard will pass (the comment's derivation is true, not aspirational).

## 2. Scope (all NO-BUMP)

### Phase 1 — md-codec (descriptor-mnemonic) — the real gap
- **T1 (test, TDD red-first):** add a NUMS drift-guard mirroring mk's, in md-codec `src/bch.rs` `#[cfg(test)] mod tests` (or a `tests/bch_drift.rs` matching md-codec's test-layout convention — implementer picks per existing style):
  ```rust
  // assert MD_REGULAR_CONST reproduces from its documented NUMS rule
  let d = sha256::Hash::hash(b"shibbolethnums");
  let hi = u128::from_be_bytes(d.as_byte_array()[0..16].try_into().unwrap());
  assert_eq!(hi >> 63, MD_REGULAR_CONST, "MD_REGULAR_CONST drift from SHA-256(\"shibbolethnums\") top-65-bits");
  ```
  (md-codec already depends on `bitcoin::hashes` via the workspace; confirm import path at impl.)
- **D1 (doc):** add a doc comment to `POLYMOD_INIT` (`bch.rs:19`): constellation-internal initial residue, deliberately **NOT** codex32/BIP-93's `1`; harmless for md because the code is self-contained (create+verify share it ⇒ length-invariant for ANY init); shared byte-for-byte with mk1's init; ms1 uses `0x1` only because its verify must agree with the **external** rust-codex32 engine. **[Minor-1 folded]** Frame the ms1 history precisely: the reverted v0.2.1 bug was a non-codex32 init *paired with* an empirically-miscalibrated target diverging from codex32 across lengths — NOT "`0x23181b3` is length-variant" (it isn't, self-contained). Cross-ref `mnemonic-secret/design/BUG_decode_with_correction_length_divergence.md:34-41`.

### Phase 2 — mk-codec (mnemonic-key) — doc accuracy fix
- **D2 (doc):** fix the `POLYMOD_INIT` comment in `src/string_layer/bch.rs` at **lines 181-185 ONLY** (the `POLYMOD_INIT` doc block — **NOT** line 166, which is `GEN_REGULAR`'s "Source: BIP 93 … `ms32_polymod`" cite and is *correct*: the generator coefficients really are BIP-93's). **[Minor-2 folded]** Replace the misleading "(BIP 93)" / "`ms32_polymod` … start with this residue" framing (BIP-93's `ms32_polymod` actually starts from `1`) with: `0x23181b3` is the constellation-internal init shared with md1, **distinct from** codex32/BIP-93's init `1` (which ms1 uses for external-codex32 interop); harmless because mk1's regular+long codes are self-contained. No code change; the drift-guard test already exists.

### Phase 3 — mnemonic-toolkit — cross-format separation capstone
- **T2 (test):** new `crates/mnemonic-toolkit/tests/bch_residue_separation.rs` (or a `#[cfg(test)]` block in `repair.rs`), asserting the constellation-level invariant the user asked about:
  - **(a) pairwise distinctness** of the four imported target constants: `ms_codec::bch::MS_REGULAR_CONST`, `md_codec::bch::MD_REGULAR_CONST`, `mk_codec::MK_REGULAR_CONST`, `mk_codec::MK_LONG_CONST` (all `pub`; mk/md already imported in `repair.rs:41-46`; ms reachable via `pub mod bch`).
  - **(b) cross-format reject** (stronger): for one canonical valid card per HRP (pulled from existing toolkit fixtures/vectors), run the toolkit's `polymod_run(hrp_expand(other_hrp) ‖ data)` and assert it `!=` the other HRP's target residue(s) — i.e. a valid ms1 codeword is not a valid md1/mk1 codeword, and vice-versa. Uses only already-imported `mk_codec::string_layer::bch::{hrp_expand, polymod_run}` + the four constants.
  - The implementer may scope (b) to whatever is cleanly expressible with on-hand fixtures; (a) is mandatory. (b) is the executable proof that HRP+target jointly separate the formats.

### Out of scope — recon-justified (NOT overlooked)
- **ms-codec:** nothing. Const-pin + drift + all-length conformance + init doc all already exist.
- **Redundant multi-length vectors for md/mk:** existing encode↔decode round-trip suites already exercise create+verify across sizes; an explicit length-sweep would duplicate. (Architect: confirm this is a real non-gap, or flag a specific missing length.)
- **Changing any `POLYMOD_INIT` value:** would be a hard wire-format break (invalidates all deployed md1/mk1 cards). Explicitly NOT doing.

## 3. Per-repo SemVer / gates

- All three repos: **NO-BUMP** (test + comment only). No README/CHANGELOG version-site bumps. (md/mk/ms each: add a CHANGELOG "Unreleased/test-hardening" line only if that repo's convention requires it — implementer checks; default no.)
- No CLI surface change ⇒ no `schema_mirror`, no `docs/manual` lockstep, no GUI pairing.
- md/mk doc-only edits do not touch the g6-synced `mlock.rs` ⇒ no cross-repo tag.
- Toolkit test must compile against the currently-pinned ms_codec/md_codec/mk_codec lib versions (verify `ms_codec::bch::MS_REGULAR_CONST` resolves; if the pinned ms_codec predates `pub const MS_REGULAR_CONST`, fall back to the literal with a comment — but recon shows repair.rs already consumes sibling `decode_with_correction`, so the pin is recent).
- FOLLOWUPS: add a NO-BUMP `bch-residue-conformance-hardening` note in each touched repo with cross-citing Companion lines (per CLAUDE.md cross-repo convention). Toolkit primary.

## 4. Open questions for R0

1. Is the md NUMS bit-extraction (`u128::from_be_bytes(digest[0..16]) >> 63` for top-65) the correct mirror of mk's verified technique? (Believed yes — same code shape; empirically the value matches.)
2. Is the length-invariance argument in the D1/D2 doc comments stated correctly (any fixed init is self-consistent; only external-standard-matching forced ms1's `0x1`)? Any over-claim to soften?
3. Cross-format test (b): is the polymod-based cross-reject sound and worth the complexity, or does (a) distinctness + the per-codec drift-guards already suffice? Pick the proportionate bar.
4. Multi-length vectors: real gap in md/mk, or correctly out-of-scope?
5. Test placement conventions per repo (in-module `#[cfg(test)]` vs `tests/`) — any house-style constraint.
