# Phase 5 — feature-dev:code-reviewer review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (Phase 5 author commit)
**Verdict:** Not converged. 2 critical / 1 important / 0 nits.

## Critical

### C1 — `KeyCard` struct field names entirely wrong in chapter 44

Chapter 44 used fabricated field names (`fingerprint`, `xpub_compact`, `policy_id_stub`). Actual fields per `mnemonic-key/crates/mk-codec/src/key_card.rs`:

| Wrong (chapter) | Correct (source) |
|---|---|
| `fingerprint: [u8; 4]` | `origin_fingerprint: Option<Fingerprint>` |
| `xpub_compact: [u8; 65]` | `xpub: Xpub` |
| `policy_id_stub: [u8; 4]` | `policy_id_stubs: Vec<[u8; 4]>` |

Only `origin_path` was correct. The example would not compile.

**Fix applied:** Replaced struct literal with `KeyCard::new(...)` using correct types from `bitcoin::bip32`.

### C2 — `KeyCard` is `#[non_exhaustive]`; struct literal won't compile externally

Rust's `#[non_exhaustive]` forbids struct-literal construction from outside the defining crate. The chapter's `let card = KeyCard { ... };` is uncompilable from any consumer.

**Fix applied:** Switched to `KeyCard::new(policy_id_stubs, origin_fingerprint, origin_path, xpub)`. C1 and C2 close together with this single rewrite.

## Important

### I1 — Chapter 44 `bin` module description is factually wrong

`bin` is not a library module; `lib.rs` declares `bytecode`, `consts`, `error`, `key_card`, `string_layer`. `src/bin/gen_mk_vectors.rs` is a binary target, not accessible via `mk_codec::bin`.

**Fix applied:** Replaced the spurious `- **`bin`**` row with prose noting that `gen_mk_vectors` is a binary target, not a library module.

## Minor / nits

(none persisted; transcript-coverage and electrum-language wording deferred to FOLLOWUPS.)

## Convergence assessment

After C1+C2 (single rewrite of the encode example) and I1 (Modules-section fix), Phase 5 is at 0C/0I. No round-2 dispatch needed.
