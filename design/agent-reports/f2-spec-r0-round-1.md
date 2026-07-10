# SPEC R0 review — F2 wc-codec RAID array-id collision — round 1

**Reviewer:** Fable (SPEC R0, read-only), per user directive. SPEC @ master `f67d0be9`.
**Dispatched:** 2026-07-10 (F2, SPEC R0 round 1). Persisted verbatim per CLAUDE.md.

**VERDICT: NOT GREEN — 0 Critical / 2 Important / 4 Minor.** Both Importants are spec-text folds (no design change); the design itself is sound.

## Important

**I-1 — "KATs intact" is false: KAT 7 pins the current array-id derivation and goes RED under (a).**
`crates/wc-codec/tests/raid.rs:397-402` re-computes the derivation in-test and asserts it:
```rust
let expected = (((d[0] as u32) << 16) | ((d[1] as u32) << 8) | (d[2] as u32)) >> 2;
assert_eq!(aids[0], expected, "array-id = top 22 bits of SHA-256(seed)");
```
The recon's "No KAT pins an id value" cited six lines short of this assertion; the SPEC inherits it (§1a "KATs intact"; G1). An implementer hits an unplanned RED on `each_plate_is_a_valid_standalone_word_card` against a guard-rail saying KATs must stay green.
**Fold:** SPEC §2 adds a test-delta: update KAT 7's derivation block to the new formula (it BECOMES the (a) determinism pin); rephrase §1a/G1 to "KATs intact except KAT 7's derivation assertion, updated in lockstep." List stale derivation doc-comments for update: `pipeline.rs:249-251` (`array_id_from_seed` docstring), `raid.rs:208-210` (`raid_encode` doc).

**I-2 — Coverage matrix overstates (b) and (c); one legacy exposure is silently open.**
(b)'s true firing condition is `#parity-plates-present > #data-plates-missing` (≥1 spare equation). Two sub-cases the matrix misses:
1. **Legacy 0-missing pure-data chimera, no parity plate presented** (n data plates, mixed vintages, no P1/P2): (b) has no equation, (c) never fires (no `*recovered` plate — G4 correctly prevents it), (a) is future-only. Info-theoretically undetectable for legacy plates (array_id/n/W all equal) → false coherence report → wrong wallet. §1b "Covers: any-r with 0 missing" overstated; §1c "everything" too.
2. **Legacy r=2 with 1 data + 1 parity missing**: solve on lone surviving parity, zero spares → (b) silent; (c) DOES fire (there is a `*recovered` plate). The "r=2-with-2-missing" exclusion (2 *data* missing) doesn't clearly claim this.
**Fold:** state (b)'s firing condition exactly; enumerate both sub-cases; mark sub-case 1 as the accepted documented legacy residual (in-band undetectable — honest option = documentation + a manual note "include a parity plate when decoding a set: it turns a vintage mix into a loud refusal"); correct (c)'s "everything" → "every decode that produces a `*recovered` plate." No design change (d rightly rejected, wouldn't reach legacy either).

## Minor
- **M-1** §1a digest underspecified: pin `H` = SHA-256 + a canonical injective serialization (fixed-width `n`; per-entry fixed-width `payload_bits` (u16, `≤0xFFFF` at `raid.rs:233`) ‖ payload bytes). Else two implementers derive different ids.
- **M-2** If (b) reuses `RaidArrayMismatch`, its doc (`lib.rs:147-151`) + Display ("mismatched array-id / n / index" `lib.rs:190-193`) don't mention parity-inconsistency; make the doc/message extension explicit in §4.
- **M-3** Manual exit-code table (`41-mnemonic.md:4663-4668`) lists only 0/1, but every `WcError` exits **2** (`error.rs:373,632`; straight-through at `word_card.rs:308,343`). Pre-existing, but this cycle regenerates that section → add/correct the exit-2 row.
- **M-4** §3 "install.sh:32" → actual path `scripts/install.sh:32`.

## Verified sound (adversarially checked)
1. **(a) non-wire-layout TRUE.** `array_id_from_seed` sole caller `raid.rs:238`; reconstruct equality-only (`raid.rs:363-373`); field stays 2 words/22 bits fixed position; `H0_VERSION`=0 untouched. Existing plates decode + self-reconstruct unchanged.
2. **(a) determinism + append-only.** Excluding `r` correct AND load-bearing (KAT 4 `tests/raid.rs:220-239` compares ParityA words across r=1/r=2 — identical iff array_id r-independent). Collision caught (different payloads → different digest → gate `raid.rs:368-371` refuses). No legit over-rejection (combining different-payload sets is never valid; identical re-issues group; r=1-P1 + r=2-P2 of same payloads group). Canonical pre-striping payloads = `raid_encode`'s `payloads` arg (available).
3. **(b) cannot over-reject a genuine array.** Exact integer algebra, no tolerance: a genuine array satisfies both parity eqs by construction; the spare holds identically; 0-missing check same. Foreign single-plate residual has every coefficient nonzero → detection certain where payloads differ; ≈1−2⁻¹¹·W is a fair conservative bound. Edge genuine-data+P1+foreign-P2 → refused = contract-consistent fail-closed, not a regression.
4. **(c) sites verified:** marker `word_card.rs:383`, `RecoveredJson.reconstructed :515-517`, `WORD_CARD_SCHEMA_VERSION :35-39`; `reconstructed=Some(true)` only MDS-solved (`:351`) → G4 achievable. Manual `*recovered` line at `41-mnemonic.md:4633`. Not schema_mirror-gated.
5. **Errors/exit:** all `WcError` → `ToolkitError::WordCard` → exit 2 (`error.rs:632`); reuse consistent; no new `ToolkitError` variant needed.
6. **Version/lockstep:** toolkit 0.83.0→0.84.0 MINOR; wc-codec 0.1.0 path dep unpublished. GUI does NOT typed-parse word-card `--json` (only generic `serde_json::Value` — `mnemonic-gui/src/form/tree_form.rs:180,280`) → companion-FOLLOWUP correct, nothing breaks at pin bump. Examples corpus has no live RAID transcripts → only ~6 gen.sh pins regen.
7. **No 4th exposure beyond I-2:** 3-array mixes covered by the same math; privacy-mode `[0u8;4]`: (a) IMPROVES it (today two all-privacy arrays with equal n collide — seed=0^4n; the payload digest differentiates — worth a §1a note); account-rotation is never a valid combine.

**Fold cost:** all six = spec-text edits (~15 lines). One fold + re-dispatch → GREEN.
