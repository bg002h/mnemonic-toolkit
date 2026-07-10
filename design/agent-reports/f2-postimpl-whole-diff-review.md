# F2 post-implementation whole-diff review — wc-codec RAID array_id collision

**Reviewer:** fresh Fable (post-impl whole-diff, read-only, adversarial). `feature/f2-raid-array-id` @ `35597e99` vs `f67d0be9`.
**Dispatched:** 2026-07-10 (F2, post-impl round 1). Persisted verbatim per CLAUDE.md.

## VERDICT: GREEN (0 Critical / 0 Important / 1 Minor)

7 files, exactly as scoped.

## #1 Frozen layout — EXACT SPEC match, KAT-7 agreement machine-proven
- `array_id_canonical` (`raid.rs:220-229`): `u32-BE payloads.len()` ‖ per index-order: `u16-BE bits` ‖ `bytes[..bits.div_ceil(8)]` — byte-for-byte SPEC §1a. `derive_array_id` (`:238-244`) = `top22(SHA-256(seed ‖ SHA-256(canonical)))`, seed-first. Top-22 extraction `((d[0]<<16)|(d[1]<<8)|d[2])>>2` = the unchanged pre-F2 wire-frozen extraction — only the hash INPUT changed.
- **`r` excluded — proven twice:** `f2a_array_id_is_deterministic_and_payload_sensitive` pins r=1 id == r=2 id; KAT 4 `p1_append_only_parity_a_byte_identical` green.
- **KAT-7 in-test re-derivation** (`tests/raid.rs:398-419`): line-by-line identical (same BE orders, same `ceil(bits/8)`, seed-then-digest streaming ≡ concatenation, same `n`) → impl/pin agreement machine-gated. No silent divergence.
- **Minimality tightening encode-domain only** (`raid_encode:286-296`); decode never re-derives → legacy plates decode + self-reconstruct. Casts guarded (n≤32, bits≤0xFFFF).

## #2 Collision catch + no over-reject — RED-proven against pre-fix code
Copied the new `tests/raid.rs` into a scratch worktree at `f67d0be9`: **4 F2 tests FAIL pre-fix** — `f2a_fresh…` fails ids EQUAL (`2350119==2350119`); `f2b_legacy_r2_cross_mix…` reproduces `Ok(…[116,80,84,222,213,18,226,233]…, reconstructed:[2])` — the EXACT documented wrong-payload mint at exit 0. Non-tautological. Post-fix all green; identical-payload re-issue still groups.

## #3 (b) spare-parity oracle — cannot over-reject a genuine array
Gate `r_available > missing.len()` (`raid.rs:516`) fires only with a spare equation. Walked every genuine case (0-missing re-derived P1/P2 equal engraved by construction; 1-missing+both-parities: solve prefers P1 → recheck holds by construction, P2 holds ∵ MDS exact). 1-missing r=1 + 2-missing r=2 correctly bypass (no spare → the (c)-only tail). Machine gates green (KATs 1/2/3/9/10 now traverse the oracle path on genuine arrays; `f2b_spare_parity_oracle_does_not_over_reject_genuine`; 40-case proptest). Note: the proptest's `Err` arm accepts any refusal → the assert-recovers KATs are the real over-rejection gate (green).

## #4 Legacy fixtures — byte-faithfulness PROVEN by regeneration
Regenerated on the OLD encoder at `f67d0be9` (`seed_of(3,900)`, `payloads([8,8,8],901/902)`, r=2): **word-for-word identical to all 5 pinned constants**; pinned plates carry the old `top22(SHA-256(seed))` id, pass the equality gate, reach the solve. The 0-missing pure-data no-parity residual pinned as expected behavior with exact-payload assertions.

## #5 (c) advisory — verified in tests AND live
JSON `verify_advisory` present only on `reconstructed:true`, null on present/solo-decode (asserted); schema `"2"`; skip-if-None → the only wire addition. Live-drove the binary: text `! verify:` prints under `*recovered`, stderr WARNING fires, all-present decode → no advisory / empty stderr / exit 0 (G4). Matches the manual transcript wording.

## #6-7 Docs
All 5 stale derivation doc-comments updated (`lib.rs` RaidMeta rustdoc, `raid.rs` raid_encode, `pipeline.rs` ×3); residue grep clean. Manual: advisory prose + "include a parity plate" note (both sites), both hand-prose schema sites `"2"`, exit-2 row verified vs code (`error.rs:577` BadInput→1, `:615` Io→1, `:632` WordCard(_)→2). No word-card transcript golden to break; `.examples-build/Examples.md` has no word-card `--json` → no examples-gate drift.

## #8 Suites (run by reviewer)
`cargo test --workspace`: **3801 passed / 0 failed** (217 suites, exit 0); clippy `-D warnings` exit 0; `fmt -p wc-codec` clean; `fmt -p mnemonic-toolkit` diff only `mlock.rs` (g6).

## #9 Scope
Exactly 7 files. No version bump, no Cargo.toml/lock, no install.sh/gen.sh, no md/mk/ms/GUI/mlock, no new `ToolkitError` variant, no clap flag (no schema_mirror impact), no new wc-codec pub API.

## Findings
- **M-1 (Minor, non-blocking):** the text-mode `! verify:` advisory (`word_card.rs:412-415`) has no automated test (every RAID-decode CLI test uses `--json`); plan P2 promised "text+JSON". Behavior verified live (correct). Fold: one ~15-line text-mode test in `cli_word_card.rs` (drop a plate, no `--json`, assert `! verify:` + stderr; assert absence on all-present). Ride the release commit or a follow-up.
- **N-1 (note):** SPEC §1a "trailing slack bits MUST be zero" documented but unenforced — injectivity holds regardless (bits field → byte count → unambiguous); both real callers byte-aligned; M-C deliberately scoped enforcement to byte count. No action.

## Release readiness
GREEN for **v0.84.0 + wc-codec 0.1.1**. Remaining (release ritual, not diff defects): version sites incl. ALL THREE wc-codec lockfiles (root, `fuzz/`, nested `crates/wc-codec/fuzz/`), READMEs, install.sh self-pin, gen.sh pins + Examples.md regen, CHANGELOG `[0.84.0]`, FOLLOWUP `wc-codec-raid-array-id-same-quorum-collision`→RESOLVED, GUI companion FOLLOWUP mirror.
