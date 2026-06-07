# R0 Architect Review — self-check-ms1-iteration — Round 2

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a4215d0dfb043b790`). Confirms the round-1 folds.

---

## VERDICT: 0 Critical / 0 Important / 1 Minor — GREEN — cleared for implementation.

## Folds Verified
- **C1 (oracle → `resolved_slots[i].entropy.is_some()`):** all 4 call sites confirmed in scope — site 1 `bundle.rs:414` (`resolved` from `resolve_slots()` @378); sites 2/3 `:1615`/`:1658` (`resolved_slots` from cosigner/descriptor loops); site 4 `:1910` (`resolved_slots` populated from envelope ms1 decode @1782 + phrase overlay @1846). WIF slots @742-750 set `entropy: None` → parity expects `""` → no false reject. Import-json G-A: @1782 sets entropy from envelope → `entropy_bearing=true` → passes. Complete.
- **I1 (G-A/G-B/G-C guards):** added; G-A + G-B genuinely discriminating (Err under the old `args.slot` oracle, Ok under the corrected one).
- **M1 (synthesized RED):** synthesize-then-mutate specified.
- **M2 (Option 1/2 framing):** deleted; single combined design.

## Mnem-payload entropy round-trip (priority item) — CLEAN
`synthesize.rs:294-314` encodes non-English as `Payload::Mnem { language, entropy }`. `ms_codec::decode` inverts via `dispatch_payload` (`envelope.rs` 0x02 branch → `Mnem { language: data[1], entropy: data[2..] }`); `data[2..]` is byte-identical to the encoded entropy. `Payload::as_bytes()` (`payload.rs:102-107`) extracts entropy uniformly for Entr AND Mnem → `decoded_entropy == resolved[i].entropy` holds for both. `cli_ms1_slot.rs::ms1_mnem_self_check_round_trips` will NOT be false-rejected.

## Length (item 3) → Minor M1-new
`entropy_bearing.len() == bundle.ms1.len()` holds structurally (both from the same n cosigners, same order) at every call site; no test can diverge it. But add an entry guard returning `BundleMismatch{card:"self-check[ms1_length_mismatch]"}` so a future divergence is a clean error not an index panic. Safety belt, non-blocking. *(Folded post-review into §3.)*

## Internal consistency (item 4)
§3/§4/§5 mutually consistent; signature change reflected; SemVer PATCH+tag v0.47.4 correct (`self_check_bundle` BIN-crate pub, not lib API; Cargo.toml at 0.47.3 → v0.47.4 next). No fold-induced drift.
