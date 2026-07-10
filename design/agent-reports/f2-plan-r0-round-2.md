# PLAN R0 review — F2 wc-codec RAID array_id collision — round 2 (convergence)

**Reviewer:** Fable (plan R0 convergence, read-only). Plan @ `f67d0be9`. Round-1: `f2-plan-r0-round-1.md`.
**Dispatched:** 2026-07-10 (F2, plan-R0 round 2). Persisted verbatim per CLAUDE.md.

## VERDICT: GREEN — 0 Critical / 0 Important / 0 Minor. Converged; implementation may begin.

**I-1 (legacy-fixture strategy) — RESOLVED, mechanically sound.** `decode` reads `array_id` off the wire (`pipeline.rs:448`); `array_id_from_seed` (`pipeline.rs:252`) has exactly one caller — `raid_encode` (`raid.rs:238`). No decode-side re-derivation, so byte-faithful pre-(a) word-lists pass the equality gate (`raid.rs:363-373`, compares plate-embedded array_id/n/width only) forever after (a). The per-plate RS+tag layer cannot reject the fixtures (RS/tag computed at encode over each plate's own content; the pinned lists are the old encoder's verbatim output — internally consistent; no decode-side payload↔id cross-check). Geometry holds (same seed ⇒ same legacy id; equal-length 73-B ⇒ equal width; same n → chimera reaches the solve + oracle). Ordering stated twice as load-bearing (P0 checkpoint + G-E). RED-proof transience handled (post-(b) the cross-mix refuses; the surviving 0-missing pure-data no-parity `Ok(wrong bytes)` is the pinned documented residual).

**I-2 (lockfiles) — RESOLVED, set complete.** `git grep 'name = "wc-codec"' -- '*Cargo.lock*'` = exactly the plan's three: root `Cargo.lock:1397`, `crates/wc-codec/fuzz/Cargo.lock:289`, `fuzz/Cargo.lock:1004`. `docs/technical-manual/examples/Cargo.lock` + all `vendor/*/Cargo.lock` have no wc-codec entry. `fuzz-smoke.yml:83` no-`--locked` rationale accurate. Bump instruction correct.

**Minors all captured.** M-A: RED cell = `Ok(wrong payload bytes ≠ either original)` at wc-codec layer + XOR-cancel/`stripe_to_payload` rationale (`raid.rs:159-173` no integrity check). M-B: both hand-prose `schema_version:"1"` sites verified live (`41-mnemonic.md:4585` flag-table `--json` row, `:4659` notes bullet; `:4633` transcript + `:4663-4668` exit table also live). M-C: "DO tighten" committed (`word_card_adapter.rs:85-94` unreachability confirmed: `payload_bits=bytes.len()*8`). M-D: single whole-diff retained, I-1 condition satisfied.

**No new gap.** M-C × I-1: the `raid.rs:233` tightening is inside `raid_encode`'s input-validation loop (`:232-236`) — encode-domain only; the decode path (`raid_reconstruct`→`stripe_to_payload`→`symbols_to_bits`) produces minimal bytes by construction, never routes through that check → legacy plates can't be rejected at decode. Fixture generation runs pre-(a) under old code; 73-B byte-aligned payloads are minimal anyway — no interaction either side.

**Executability:** KAT-7 site (`tests/raid.rs:397-402`), the 40-case proptest (`:566-616`, post-(b) 0-missing regression net), and every P2 toolkit site (`word_card.rs:35-39/:351/:383/:515-517`, `lib.rs:103/:147-151/:190-193`) all live. (b) test vectors fully determined by the pinned fixtures — no improvisation. Single-Opus P0→P1→P2 sequential executable start-to-finish.

**Cleared:** dispatch the single Opus implementer on `feature/f2-raid-array-id` with the pre-(a) checkpoint (RED-proof + fixture pinning) as the mandatory FIRST action of P0.
