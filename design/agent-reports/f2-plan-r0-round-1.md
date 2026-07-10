# PLAN R0 review — F2 wc-codec RAID array_id collision — round 1

**Reviewer:** Fable (plan R0, read-only), per user directive. Plan @ `f67d0be9` vs live source.
**Dispatched:** 2026-07-10 (F2, plan-R0 round 1). Persisted verbatim per CLAUDE.md.

## VERDICT: NOT GREEN — 0 Critical / 2 Important / 4 Minor. Fold and re-dispatch.

Phase split coherent, release ritual ~90% complete, RED-proof feasible — but one executability hole guts P1's test story, and one missed version site.

## IMPORTANT

**I-1 — P1's (b) KATs are unconstructible through the public API once P0 lands; the plan pins no legacy-fixture strategy.**
After (a), two same-seed different-payload arrays get DIFFERENT `array_id`s (prob 1−2⁻²²), so any cross-mix built via post-fix `raid_encode` refuses at the EXISTING equality gate (`raid.rs:363-373`) and never reaches (b)'s parity oracle. No public-API construction yields same-`array_id`/different-stripe plates post-(a): same payloads ⇒ identical stripes (no-op); r is digest-excluded but r=1-vs-r=2 same payloads ⇒ identical data+P1; order is digest-included; within-RS-budget corruption is corrected back. (b) exists precisely for LEGACY plates, which new code cannot emit → P1's TDD cells are dead on arrival, and the funds-critical (b) verification would be improvised mid-flight.
**Fold:** at the G-E pre-fix checkpoint, ALSO generate and PIN as hardcoded word-list fixtures in `crates/wc-codec/tests/raid.rs` the full plate sets of two same-seed different-payload r=2 arrays (+ an r=1 pair if desired) — byte-faithful "plates engraved under the old derivation"; `decode` never re-derives `array_id`, so they pass the equality gate and exercise the oracle. State the hard ordering: fixture generation happens at the same pre-(a) checkpoint as the RED-proof run. (Alt: `#[cfg(test)]` unit tests in `src/raid.rs` using `pub(crate)` encoders with a forged id — weaker fidelity; prefer fixtures.)

**I-2 — Release ritual misses `crates/wc-codec/fuzz/Cargo.lock` (nested fuzz workspace pins `wc-codec 0.1.0` at `:289-290`).**
The plan lists only "workspace `Cargo.lock`". Sites needing the 0.1.1 bump: root `Cargo.lock:1397-1398`, `fuzz/Cargo.lock:1004-1005`, AND `crates/wc-codec/fuzz/Cargo.lock:289-290` (tracked; own workspace, invisible to `--workspace` + the root `--locked` guard `rust.yml:96-97`; `fuzz-smoke.yml:83` builds it WITHOUT `--locked` so CI won't redden — it goes silently stale + dirties the tree on the next local `cargo fuzz`). First-ever wc-codec bump → no prior precedent. **Fold:** enumerate all three lockfiles.

## MINOR
- **M-A — RED-proof home/wording:** the KAT home in `tests/raid.rs` is CORRECT — at the wc-codec layer the pre-fix mix deterministically returns `Ok(wrong payload bytes)` (equal-length 73-B payloads ⇒ length-prefix XOR-cancels; `stripe_to_payload` `raid.rs:159-173` no integrity check ⇒ Ok prob 1). The "exit-0 wrong-XPUB" manifests only at the toolkit mk1 re-parse (~50% secp density = eval's 21/36) and would be flaky without pre-searched seeds. Reword P0's RED cell to "reproduce pre-fix `Ok(wrong payload bytes ≠ either original)`"; toolkit-layer companion cell optional.
- **M-B — Two hand-prose `schema_version: "1"` manual sites go stale on the bump, NOT in P2's list:** `41-mnemonic.md:4585` (word-card `--json` flag-table row) + `:4659` (notes bullet). Transcript regen won't touch prose. Add both to P2.
- **M-C — Resolve "optionally tighten `raid.rs:233`" → DO tighten** (reject non-minimal `bytes.len()`): enforces the injectivity precondition the frozen digest depends on, unreachable from the real caller (`word_card_adapter.rs:85-94`), removes an impl/KAT-7 divergence vector. "Optionally" in a funds-safety plan invites drift.
- **M-D — Single whole-diff review granularity: ACCEPTABLE, conditional on I-1.** (b)'s OVER-rejection (funds-availability) is machine-gated: KATs 1/2/3/9/10 + the 40-case proptest (`tests/raid.rs:566-616`) push genuine arrays (incl. parity-present full sets, which post-(b) newly run the 0-missing verification) through `raid_reconstruct` → any over-reject reddens the required `test (ubuntu-latest)` immediately; G-B adds the r=2 pin. The residual (b) risk is UNDER-rejection, verified by I-1's legacy fixtures. With I-1 folded, no separate (b) per-phase R0 needed; keep the balloon escape-hatch.

## Verified clean (Q3-Q6)
- Version sites @ `f67d0be9`: `wc-codec/Cargo.toml:3`=0.1.0; `mnemonic-toolkit/Cargo.toml:3`=0.83.0; root `Cargo.lock` BOTH entries (`:730-731` toolkit, `:1397-1398` wc-codec); `fuzz/Cargo.lock:578-579`+`:1004-1005`; both READMEs; `scripts/install.sh:32`; `gen.sh` = 6 pins (3/44/109/126/711/724); repo-wide `0.83.0` grep clean elsewhere. `mnemonic-toolkit/Cargo.toml:37` = version-less path (not a version site). No re-vendor; no sibling-pin.
- changelog-check fires on the tag, needs `[0.84.0]`.
- CI gating: `rust.yml:118` `cargo test --workspace` covers both members; live protection `[examples, test (ubuntu-latest), clippy]` → new wc-codec KATs run in a required context. Release = direct-FF admin push (enforce_admins off → bypasses required checks); the green guarantee is the ritual's local full-workspace test + post-push verify (both at plan `:30`).
- Error mapping: `RaidArrayMismatch` reuse, no new `ToolkitError` variant; `WordCard(WcError)`→exit 2 (`error.rs:373,632`); manual exit-table correction correctly in P2.
- Doc-comment sites (M-6): all five live (`pipeline.rs:249-251,:228,:237`, `raid.rs:207-210`, `lib.rs` `RaidMeta::array_id` rustdoc); KAT-7 old-derivation pin at `tests/raid.rs:397-402`; new `canonical` re-derivable in-test from the KAT's own payloads.
- GUI: generic `serde_json::Value` display only — companion FOLLOWUP right.

**Next:** fold I-1/I-2 + minors, re-dispatch round 2. No implementation before round-2 GREEN.
