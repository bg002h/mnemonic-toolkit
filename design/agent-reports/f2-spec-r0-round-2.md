# SPEC R0 review — F2 wc-codec RAID array-id collision — round 2 (convergence)

**Reviewer:** Fable (SPEC R0 convergence, read-only). SPEC @ master `f67d0be9`. Round-1: `f2-spec-r0-round-1.md`.
**Dispatched:** 2026-07-10 (F2, SPEC R0 round 2). Persisted verbatim per CLAUDE.md.

## VERDICT: GREEN — 0 Critical / 0 Important / 2 Minor (non-blocking). Implementation may begin; fold the two Minors as one-line spec edits at implementation time.

## Round-1 findings — all resolved
**I-1 RESOLVED.** KAT 7's in-test derivation is exactly at `tests/raid.rs:397-402` (SHA-256 recompute :398-400, `>> 2` :401, assert :402). Re-grep confirms it is the ONLY test re-deriving the id (KAT 6 :316-348 is behavior-only equality-mixing). §2 test-delta + §1a/G1 rephrase accurate. No toolkit-side/wc-codec-unit test pins the derivation. (Doc-comment residue → M-6.)

**I-2 RESOLVED.** (b)'s firing condition `#parity-present > #data-missing` is exactly the spare-equation condition. Every legacy mixed-vintage cell enumerated: 0-missing+parity → (b); r=2 1-data-missing+2-parity → (b); r=1 1-missing + r=2 1-data+1-parity-missing → (c); r=2 2-data-missing+2-parity (no spare) → (c)'s general clause "every decode producing a `*recovered` plate"; 0-missing pure-data no-parity → the documented accepted residual. **Residual genuinely in-band undetectable:** all-present data plates carry NO cross-plate redundancy (each individually integrity-tag-valid; only binding is array_id/n/index/W, all equal for legacy same-quorum) — detection info-theoretically impossible without a parity equation; no wire change reaches already-engraved plates. Documentation + manual note is the only honest option; (a) confines it to legacy. G4 fire-only-on-`*recovered` is right (always-on would fire on 100% of genuine decodes). *Optional non-blocking:* since a RAID array always has r≥1, a 0-missing pure-data set-decode implies a parity plate exists but wasn't presented → a one-line stderr hint ("no parity plate presented — cross-plate coherence unverifiable for pre-v0.84.0 arrays") is a true, shape-specific runtime note. Manual-note-only is acceptable; take or leave.

**M-1 RESOLVED** (residual nit → M-5). H=SHA-256, u32-BE n, per-entry u16-BE `payload_bits` (≤0xFFFF at `raid.rs:233`) ‖ payload bytes. KAT-7 delta + §1a agree (re-derive `canonical` identically in-test; `orig`+`seed` in scope). **M-2 RESOLVED** (`lib.rs:147-151` + `:190-193` verified). **M-3 RESOLVED** (exit table `41-mnemonic.md:4663-4668` lists 0/1 only, parse-failure mis-filed under 1 at :4668; all `WcError`→exit 2 via `error.rs:373,632`). **M-4 RESOLVED** (`scripts/install.sh:32`).

## Convergence confirmed
Design unchanged (all six folds text-additive; (a) non-wire-layout, r-exclusion KAT 4 `tests/raid.rs:220-239`, (b) no-over-reject, (c) sites all re-verified). No new Important/Critical.

## Remaining Minor (new, non-blocking)
- **M-5 — §1a canonical-form residue:** (i) "`n` as a fixed-width integer (e.g. u32-BE)" — drop the "e.g." (pin the width). (ii) "‖ the payload bytes" has no byte-count rule → over `raid_encode`'s accepted domain (`raid.rs:233` permits `bytes.len() > ceil(bits/8)`) the serialization is NOT injective — concrete collision: n=2, `{(1,[80 00 02 FF]),(8,[AA])}` vs `{(1,[80]),(2,[FF 00 08 AA])}` serialize identically. Unreachable from real callers (mk1 byte-aligned minimal `payload_bits=8·len`, `word_card_adapter.rs:85-94`; md1 minimal+zero-slack, not RAID-able) → no practical exposure, BUT the derivation freezes into KAT 7 + engraved steel → pin now: "payload bytes = exactly `ceil(payload_bits/8)` bytes, trailing slack bits zero" (or tighten `raid.rs:233` to enforce minimality). Protects the §1a "identical re-issue shares the id" claim.
- **M-6 — §2 stale-doc list incomplete:** besides `pipeline.rs:249-251` + `raid.rs:208-210`, three more state the old `SHA-256(seed)` derivation: **`lib.rs:103` (public rustdoc on the exposed `array_id` field — ships a false API doc if missed)**, `pipeline.rs:228`, `pipeline.rs:237`. Add to the update list.

**Cleared:** SPEC GREEN (0C/0I) — all round-1 findings resolved against source, no design drift; proceed to plan/implementation, folding M-5 (pin canonical byte-count + drop "e.g.") + M-6 (3 extra stale doc sites) as trivial text edits.
