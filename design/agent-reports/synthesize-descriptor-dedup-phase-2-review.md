# Phase 2 (GREEN) Code Review — `synthesize_unified` → `synthesize_descriptor` dedup

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory per-phase review). **Date:** 2026-06-06.
**Branch:** `synthesize-descriptor-dedup`. **Verdict:** **0 Critical / 0 Important / 0 Minor** (static). **GREEN — Phase 2 may proceed to Phase 3** (guards operator-confirmed below).

> Persisted verbatim per CLAUDE.md. Reviewer had no shell; verified the delegation + front-half preservation + non-vacuous frozen characterization statically. The load-bearing guards (the byte-exact cell staying GREEN = the behavior-preservation proof) were operator-run GREEN.

---

## VERDICT: 0 Critical / 0 Important / 0 Minor (static) — guards operator-confirmed GREEN

### What verified clean (static)
**Item 1 — delegation correct, front-half preserved.** `synthesize_unified` (`synthesize.rs:745-827`) retains its full front-half: slot-count (`:753-758`), threshold (`:759-763`), per-slot network/xpub cross-check (`:765-776`), path-decl (`:780-787`), `Descriptor{…}` construction (`:790-817`). Back-half replaced by one delegation (`:826`): `synthesize_descriptor(&descriptor, slots, privacy_preserving, run_language)` — args match the signature (`:229-234`) in order; `slots: &[ResolvedSlot]` typechecks as `&[CosignerKeyInfo]` (`type` alias `:219`); `descriptor.n = slots.len()` so the delegated count-check (`:236`) passes. No orphaned front-half local.

**Item 3 — characterization cell non-vacuous + frozen.** Asserts hardcoded literals (NOT a live `assert_eq!(unified, descriptor)` — satisfies R0 M2); drives n>1 Multi (`.as_multi().expect`); two distinct cosigners (`TREZOR_12_ZERO` vs `BIP39_TEST_2`) → distinct csi → `mk[0]` (`mk1qp40rrp…`) ≠ `mk[1]` (`mk1qpv4y3z…`); asserts ms1[0]+ms1[1]+mk1[0]+mk1[1]+md1 byte-exact.

**Item 4 — no dead code / no masking.** No `#[allow(unused)]` in synthesize.rs (grep 0). Surviving front-half locals all feed the descriptor/checks; the back-half-only `stub`/`stubs`/`policy_id` are gone from `synthesize_unified`; `MsField`/`MkField`/`derive_mk1_chunk_set_id`/`mk1_origin_path` stay live (consumed by `synthesize_descriptor`).

**Item 5 — call sites untouched.** `synthesize_unified` ×4 (`bundle.rs:399`, `verify_bundle.rs:374/464/568`), `synthesize_descriptor` ×5 — all unchanged signatures.

---

### Operator-run guards (the empirical behavior-preservation proof — all GREEN)
- `cargo test -p mnemonic-toolkit --bin mnemonic synthesize_unified` → **7/7** incl. `synthesize_unified_multisig_distinct_cosigners_byte_exact` (the frozen byte-exact cell stays GREEN ⇒ the delegation produced byte-identical Multi-branch output — THE proof).
- `cargo test -p mnemonic-toolkit --no-fail-fast` → **0 failures**.
- `cargo clippy -p mnemonic-toolkit --all-targets` → **0 warnings (exit 0)**.
- `make -C docs/manual verify-examples` (4 pinned bins) → **OK (20 transcripts)** — template-mode bundle transcripts byte-identical.
- `git show 1ea2317 --stat` → **only `synthesize.rs`** (8 ins / 77 del); `synthesize_descriptor` body untouched across the cycle (`git diff master..HEAD` shows no `fn synthesize_descriptor` line change).

---

**0 Critical / 0 Important / 0 Minor + all guards GREEN. Phase 2 may proceed to Phase 3 (release).**
