# R0 REVIEW — cycle-13 Lane A: Coldcard/Jade multisig fidelity pair (H11 + H14) — Round 2

> Reconstructed from the round-2 reviewer's summary (notification tangling under high parallelism). Verdict + confirmations as reported.

## VERDICT: GREEN (0 Critical / 0 Important)

The round-1 folds are accepted with no new drift; every citation verified against `origin/master = 9b2a8ae3` + vendored rust-bitcoin 0.32.8.

### I-1 (canonicalizer collapse) — FOLDED CORRECTLY
`wallet_import/roundtrip.rs` added as the third affected file (§2.5); Decision H11-f (extend `canonicalize_coldcard_multisig` `:361` to emit per-cosigner `Derivation:` on heterogeneous paths; shared line when homogeneous) makes the canonicalizer idempotent on divergent H11 exports. RED tests #15 (divergent idempotence) + #16 (round-trip-verify passes). The existing `canonicalize_coldcard_multisig_idempotent` (`:1397`) baseline confirmed real; consumed at `import_wallet.rs:1447` + `roundtrip.rs:570`.

### I-2 (sorted-slot path↔xpub pairing) — FOLDED CORRECTLY
§2.1 states sorted-only reachability (cycle-2 H10 refusal `export_wallet.rs:124-135`) + the slot-order-vs-sort-order scramble hazard (`coldcard.rs:324-328`/`:339-346`/`:363`). H11-b rewritten to read path+xpub+fp from the SAME sorted slot (never `derivations[i]`). RED test #1b (xpub-sort ≠ slot-order + divergent paths) exercises the scramble.

### Q1 condition + Minors — FOLDED
- Q1: §3.1 mandates consume-pending-only / never-clear-`shared_derivation` + RED test #13 (3-cosigner shared-path regression). Resolution-A ratified.
- M-1: Jade-import blast-radius (`jade.rs:133`/`mod.rs:122`) + RED test #14.
- M-2: synthesize.rs depth-4-xpub/depth-0-fp Row-2 note + H14-h.
- M-3: SHA re-pinned `d55bf4c3` → `9b2a8ae3`.
- M-4: fixture consts `:945-947`; rust-bitcoin `bip32.rs:833-842`/`:111`; stale `identifier()` doc-comment flagged.

Q1/Q2/Q3 decisions intact (resolution A; refuse depth>0/no-XFP; `xpub.depth==0`); H14 matrix unchanged. RED-test list (~16), affected-files (coldcard.rs/jade.rs export, coldcard_multisig.rs import, roundtrip.rs canonicalizer + SPEC + fixtures), scope all consistent.

## Advisory nits (non-blocking, for the plan-doc author)
- §2.2 cites `computed_fp` at `:358-360` (binding is `:359-360`; `:358` is the preceding comment).
- §2.1 cites `cs.fingerprint` at `:367` (read at `:366`, emitted at `:367`).

## Disposition
GREEN. The lane proceeds to the plan-doc stage (own R0 loop). Ships as part of toolkit MINOR v0.66.0 with Lanes B/C.
