# Plan-R0 (toolkit bitcoind end-to-end oracle) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, opus). **Verdict: NOT GREEN — 0 Critical / 2 Important**
> (+ 4 Minor). Plan-doc at review: `design/PLAN_toolkit_bitcoind_end_to_end_oracle.md`;
> toolkit `master@ffdf3d3`. The architect verified every load-bearing pipeline claim
> against live source (single-sig + multisig restore-from-md1 routing, NUMS detection
> on concrete descriptors, watch-only bundle acceptance, the multipath split, md-codec
> template fidelity, fmt/clippy/CI-impact, the FOLLOWUP) — all TRUE. The two Importants
> are test-correctness precision, not pipeline viability.

---

## Verbatim agent output

# VERDICT: NOT GREEN — 0C / 2I

The plan's architecture is sound and the pipeline is real: every load-bearing claim about the toolkit's behavior (single-sig and multisig restore-from-md1, NUMS detection on concrete descriptors, watch-only bundle acceptance, the multipath split) verifies against live source. The two Important findings are about test-correctness precision (anti-vacuity strength + one unproven corpus shape), not pipeline viability. No Critical findings.

## What I verified TRUE (the plan's load-bearing claims hold)

**Pipeline JSON field paths — CONFIRMED.** `bundle --descriptor … --json --no-engraving-card` emits `v["md1"]` array (`bundle.rs:39`, `:76-77`; STRESS-A `prop_backup_restore_roundtrip.rs:308-314`). `--threshold` is refused in descriptor mode (`bundle.rs:179`). `restore --md1 … --count 5 --json` emits `v["wallets"][0]["descriptor"]`+`["first_addresses"]` (`restore.rs:1661-1667`; STRESS-A `:335-343`). `--count` default 1 (`restore.rs:124`) → `--count 5` mandatory (Risk 8 correct).

**Single-sig restore-from-md1 routing — CONFIRMED, see I-1.** A single-sig `bundle --descriptor wpkh(...)` produces a wallet-policy md1 (`is_wallet_policy()` true when `tlv.pubkeys` populated — `encode.rs:50-52`; `synthesize.rs:154`), so `restore --md1` dispatches to `run_multisig` (`restore.rs:177`), passes the `is_wallet_policy` gate (`:1232`), `plain_template_from_tree` returns None for Wpkh/Pkh (`:1163-1182`) → GeneralFaithful arm (`:1344` → md-codec `to_miniscript_descriptor`). md-codec renders pkh/sh(wpkh)/wpkh/tr-keypath through this exact fn.

**Shapes 8 & 9 (tr NUMS multi_a/sortedmulti_a) — CONFIRMED.** Reconstruction tests exist: `cli_restore_multisig.rs:257` (sortedmulti_a), `:281` (multi_a). Concrete-descriptor intake reaches NUMS detection: `bundle_run_concrete_descriptor` (`bundle.rs:1692`) → `descriptor_concrete_to_resolved_slots` (`pipeline.rs:311`) → `parse_descriptor` (`pipeline.rs:322`) → `walk_tr` sets `is_nums:true` when internal key == `NUMS_H_POINT_X_ONLY_HEX` (`parse_descriptor.rs:468-472`); plan's NUMS_HEX byte-matches (`parse_descriptor.rs:263`).

**Excluded shapes refuse — CONFIRMED.** `classify_taproot_restore` (`restore.rs:696-752`): @-in-both → `refuse_at_in_both` (`:777-795`); depth-≥2 → `ensure_taptree_depth_le_one` (`:819-847`); sortedmulti_a-under-TapTree → `:741-747`; sortedmulti-in-combinator sole-child message (pinned `prop_backup_restore_roundtrip.rs:608`).

**Watch-only bundle — CONFIRMED.** `bundle_run_concrete_descriptor` sets `BundleMode::SingleSigWatchOnly`/`MultisigWatchOnly` (`bundle.rs:1727/1730`); no seed.

**Multipath split — CONFIRMED.** `miniscript = "13"` direct dep (`Cargo.toml:44`); `derive_address.rs:34-35,80-89` uses `is_multipath()`+`into_single_descriptors()`; STRESS-A splits in an integration test (`:386-390`); reconstructed carries `/<0;1>/*` (`restore.rs:1079-1084`; `cli_restore_multisig.rs:266`).

**md-codec template fidelity — CONFIRMED.** All lifted pieces in `bitcoind_differential.rs` (v27.0+sha256 `:28-32`, Wiring/read_wiring `:409-437`, bitcoin_cli `:442-461`, chain=="main" `:484-489`, golden-before-compare `:570-576`, checksum `:518-527`). Dropping the clone-fork step is correct (toolkit pins via `[patch.crates-io]` `Cargo.toml:28-29`).

**fmt/clippy/CI — CONFIRMED.** `rust.yml:111` `cargo test -p mnemonic-toolkit` (no `--include-ignored`) skips the ignored test. fmt gate exempts only `/mlock\.rs$` (`:69`). clippy `--all-targets` (`:198`) compiles the ignored test (plan covers). FOLLOWUP `toolkit-bitcoind-end-to-end-oracle` open at `FOLLOWUPS.md:4178-4186` (`:4184`); companion `bitcoind-differential-corpus-breadth` separate. Toolkit workflows use `@v5`; `cross-tool-differential.yml:48-55` is the closest in-repo template.

## IMPORTANT findings

### I-1 — Shape 6 (`wsh(and_v(v:pk(@0),older(144)))`) is a NOVEL round-trip shape never proven by any existing test.
8 of 9 shapes trace to existing proof (3,4,5,8,9 in `cli_restore_multisig.rs`; 7 via STRESS-A schema 9 `:239-248`; 1,2 via the faithful arm md-codec renders). But shape 6 is a pure single-key (n=1) general wsh policy with NO multi — outside STRESS-A's generated set (always multi at the trunk) and outside `cli_restore_multisig.rs` (multisig-only). md-codec CAN render it (its bitcoind shape 9 is byte-identical) and the `pk`-in-`and_v` fragment is proven, so it WILL reconstruct — but the plan presents it as proven when it's the single unexercised positive shape. **Fix:** (a) extend Phase-1's local run to capture shape 6's bundle→restore evidence in the persisted review; OR (b, stronger) add a permanent `cli_restore_multisig.rs`-style characterization cell for n=1 general-wsh restore so reconstructability is gated by the DEFAULT suite, not only the env-gated cron oracle. State that shape 6 has no current default-suite coverage + which option closes it.

### I-2 — The anti-vacuity golden is a self-captured snapshot of synthetic KEYS, weaker than md-codec's published-vector golden.
The plan captures `WPKH_CHAIN0_IDX0_GOLDEN` from the toolkit's OWN output, then asserts `first_addresses[0]==GOLDEN`. This catches a dead Core connection but NOT a correlated toolkit-derivation drift (a change shifting both the live derivation AND a re-captured golden stays green vacuously). The KEYS xpubs are real BIP-32 xpubs, so an INDEPENDENT golden is computable. **Fix:** capture the golden from an INDEPENDENT derivation — (a) rust-miniscript `derive_receive(shape1_desc,1)[0]` in-test (reuses STRESS-A's helper `:383-401`; makes the golden a genuine second oracle: "toolkit restore == independent miniscript derivation"), or (b) a literal from a one-time Core `deriveaddresses`. Do NOT read it from `restore … first_addresses[0]` (the unit under test).

## MINOR
- **M-1:** STRESS-A KEYS bake `/48h/0h/0h/2h` origins; the plan's table shows `[fp/84h…]`/`[fp/86h…]`. Origin is opaque metadata (no purpose/depth validation; derivation uses xpub+`/0/i`), so either relabel or use verbatim — but STATE which (silently emitting `wpkh([fp/48h…]…)` looks wrong though harmless). Recommend explicit per-shape origin relabel, documented as metadata-only.
- **M-2:** The FOLLOWUP says run this AFTER GAP-4a (`FOLLOWUPS.md:4182-4183`). GAP-4a (cross-tool widening) is substantially in place (`cli_cross_tool_differential.rs` wired in CI) — add one line acknowledging the sequencing.
- **M-3:** Cite the in-repo `cross-tool-differential.yml` as the workflow convention model (@v5, MNEMONIC_BIN, build --bin, --ignored), not just md-codec's @v4.
- **M-4:** Confirm in Phase 1 that the toolkit's reconstructed-descriptor checksum equals Core's `getdescriptorinfo` checksum for shape 1 (miniscript + Core both BIP-380, should agree; md-codec's precedent is for md-codec's own checksum).

## Anti-vacuity assessment
Genuinely non-vacuous once I-1/I-2 folded. Shape 9 is toolkit-unique (md-codec 13.0.0 lacks SortedMultiA; toolkit's 95fdd1c has it). Deferring distinct-trunk is right. Chain-1 descriptor-equivalence secondary (Risk 9) is sound given restore surfaces only receive (`restore.rs:1375`).

**Re-dispatch:** fold I-1 + I-2, persist verbatim, re-dispatch until GREEN. No code until GREEN.
