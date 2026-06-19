# Implementation plan-doc R0 round 1 — restore per-key use-site override (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; read-only; every citation re-verified against descriptor-mnemonic `c85cd49` and mnemonic-toolkit `5e55603`, incl. rust-miniscript `into_single_descriptors` mixed-key behavior on crates.io 13.0.0 + git rev 95fdd1c).
> Plan: `design/IMPLEMENTATION_PLAN_restore_per_key_use_site_override_2026-06-19.md`. **Verdict: GREEN (0C/0I).** 5 Minors (M1-M5) folded this session; re-confirmed in round 2.

---

This gives me the test-coverage gap analysis. Comparing SPEC §5.6 matrix shapes against plan §5 test inventory:

SPEC §5.6 matrix rows:
1. `wsh(multi)`/`wsh(sortedmulti)`/`sh(wsh(...))`, Some/Some OR Some/None divergent → **Faithful**
2. `sh(multi)` (bare P2SH) override → **Faithful** (via `plain_template_from_tree` matches only Wsh/Sh-Wsh → None)
3. ANY hardened → **Loud refuse**
4. `tr(...)` with overrides → **Loud refuse**
5. Adversarial wire (`@0` override / redundant) → **Decode reject**
6. Non-override card → Unchanged

Plan §5 / §3 test coverage:
- Row 1 wsh(multi) divergent: covered (P1.2, P1.3, P2.2, P2.5, §5 items 2,3,5,7)
- Row 1 Some/None mix: covered (P1.3, P2.2, §5 item 3)
- Row 1 wsh(sortedmulti) divergent: covered (P2.2 "a wsh(sortedmulti) divergent", §5 item 3 "sortedmulti divergent")
- Row 1 `sh(wsh(...))` divergent: **NOT explicitly tested** — the plan tests wsh(multi) and wsh(sortedmulti) but no explicit sh(wsh(multi)) divergent test. Minor gap (same faithful arm; covered structurally).
- Row 2 `sh(multi)` bare-P2SH override → faithful: **NOT explicitly tested in the plan** (the SPEC §5.6 lists it as a distinct faithful case via the `plain_template_from_tree` Wsh/Sh-Wsh-only match). The plan's P2.1 TDD only tests the wsh standard-baseline routing. This is a real coverage gap for a distinct routing/mechanism row.
- Row 3 hardened (baseline `/*h`, override hardened-wildcard, override hardened-alt): covered (P1.1, P2.3, P2.4, §5 items 1,7)
- Row 4 tr(multi_a) override refuse: covered (P2.3, P2.4, P2.5, §5 items 5,7)
- Row 5 decode rejects: covered (P1.4, §5 item 4)
- Row 6 non-override: regression corpus (P1.4 "round-trip of ALL existing corpus cards")

So the only material test-coverage gaps are **`sh(multi)` bare-P2SH override (Row 2)** and **`sh(wsh(multi))` divergent (Row 1 sub-case)**. These are Minors (same mechanisms exercised by adjacent shapes; the faithful arm is shape-agnostic), but Row 2 is a distinct routing decision (the `plain_template_from_tree` Wsh/Sh-Wsh-only match) that no test pins.

I have completed thorough verification. Let me compile the final review.

---

# R0 Review — IMPLEMENTATION_PLAN_restore_per_key_use_site_override_2026-06-19.md

**Reviewer:** opus architect (read-only; every citation re-verified against descriptor-mnemonic `c85cd49` and mnemonic-toolkit `5e55603`).
**Verdict: GREEN.** 0 Critical, 0 Important. The plan correctly implements both SPEC closures (C1 plain-arm routing, C2 faithful per-`@N` reconstruction), the chosen C2 API is sound against the actual rust-miniscript invariants, the phasing/ordering is correct, and no silent-mis-render hole remains. A set of non-blocking Minors follows, two of which are worth folding before execution.

## Citation + mechanism verification (all confirmed)

**C2 API decision (§1) — SOUND.**
- (a) `@N` == `expand_per_at_n` Vec position: confirmed at `canonicalize.rs:339` (doc), `:420/435` (`for idx in 0..d.n` push-in-order), and resolution at `:458-460`. The tree binds keys by `@N` via `lookup_key(keys, index)` (`to_miniscript.rs:140`). Correspondence is definitional.
- (b) `node_to_descriptor(&d.tree, &keys)` (`to_miniscript.rs:134`) and its helpers `build_multi_threshold`/`lookup_key` (`:487/:481`) are fully `DescriptorPublicKey`-generic — they never inspect `XPub` vs `MultiXPub`. A `MultiXPub`-keyed descriptor builds and its `.to_string()` is `<…>` notation. Confirmed.
- (c) sortedmulti per-index sort: `sortedmulti.rs:99-128` — `sorted_node()` sorts only after `ToPublicKey` derivation (the doc-comment at `:99-101` states keys "cannot be sorted until they are converted to consensus-encoded public keys"). So per-index splitting happens first (in `into_single_descriptors`), then sort-per-derived-key. Divergent suffixes sort correctly per index. Confirmed.
- **The mixed `MultiXPub` + `XPub` in ONE descriptor question (the crux): LEGAL and derivable.** `into_single_descriptors` (miniscript `mod.rs:870-926` on crates.io 13.0.0; identical at git rev `95fdd1c:946-988`) clones `XPub`/`Single` keys unchanged and splits only `MultiXPub` per index. The upstream test "Even if only one of the keys is multipath" (`mod.rs:2127`/`95fdd1c:2249`) explicitly pins this. The only invariant is that all `MultiXPub` keys in one descriptor share the same `derivation_paths.len()` (else `MultipathDescLenMismatch` at `:912`) — and `validate_multipath_consistency` (`validate.rs:117-138`) already enforces equal alt-count across all `Some`-multipath entries (skipping `None`), so the builder can never emit mismatched-length `MultiXPub` keys. The builder approach breaks no md-codec / rust-miniscript invariant.

**P2.2 translator reduction — SOUND.**
- Keys arriving at `translate_pk` after C2 are `MultiXPub`/`XPub` (the multipath descriptor) — correct. Setting `xkey.network` on a `MultiXPub` is the right correction (`DescriptorMultiXKey.xkey` is the single `Xpub`, network-keyed once for all alts).
- The current translator has a `Single` arm (strict-NUMS, `restore.rs:1040-1047`) and an `XPub` arm (`:1050-1092`); it has **no `MultiXPub` input arm** today (it only produces them). The plan correctly says to ADD a `MultiXPub` arm. Keep the strict-NUMS `Single` refusal (`:1044`) and the "cannot wrap" hint mapping (`:1113`) — both preserved per the plan.
- `to_miniscript_descriptor_multipath` DOES produce Main-network keys: `xpub_from_tlv_bytes` hardcodes `network: NetworkKind::Main` (`derive.rs:57`), so the network pass is still required. Confirmed.

**P2.1 C1 gate (`restore.rs:1289`) — SOLE non-taproot routing decision; taproot pre-refused.**
- `:1289` `(plain_template_from_tree(&d.tree, &d.use_site_path), None)` is the non-taproot branch; taproot is the separate `:1283-1287` branch (via `classify_taproot_restore`). The P2.3 guard (replacing `:1247`) runs at the gate BEFORE classify (`:1262`), so `taproot_override_card(d)` refuses taproot override cards before `:1289`. Confirmed.
- `build_descriptor_string` callers: `restore.rs:387` (single-sig, no md1 TLV), `:1336` (plain arm, gated by `template_opt`), and `wallet_export`/engrave (round-2 confirmed never sees an override card via restore). No other path to `build_descriptor_string` for an override card. Confirmed.
- The faithful arm's address VALUE comes from the reconstructed STRING parsed at `:1355` → `derive_receive_addresses` (`:1375`), which handles multipath via `into_single_descriptors()` (`derive_address.rs:80-81`) — NOT from `d.derive_address`. So C2's string fully determines the faithful-arm address; D1 does not touch it (correct division of labor).

**P1.1 Point B (`derive.rs:99/110`) — correct; range check retained and sufficient.**
- `:99` (`self.use_site_path.wildcard_hardened`) and `:110` (`alts[chain].hardened`) are both baseline-only; replacing them with `has_hardened_use_site(self)` closes the override-hardened-alt gap (today surfaces as generic `AddressDerivationFailed`). The interleaved chain-index range check (`:103-118`) must be kept — and it is sufficient because per-key range is independently re-checked inside `use_site_to_derivation_path` (`to_miniscript.rs:120-124`), and `validate_multipath_consistency` guarantees override alt-counts equal the baseline (or are `None`). `derive_address` is the only md-codec derivation entry (sole `to_miniscript_descriptor` caller is `derive.rs:120`). Confirmed.

**Phasing/ordering — correct and complete.**
- md-codec-first publish → toolkit-pin is right. The main `Cargo.lock` md-codec entry has `source = "registry+..."` (`Cargo.lock:678`), confirming the toolkit consumes the PUBLISHED crate; CI builds against crates.io. The local-`[patch.crates-io]`-during-dev → drop-patch-and-pin-`0.37.0`-before-push mechanic is sound: the toolkit already carries `[patch.crates-io] miniscript = {git rev 95fdd1c}` (`Cargo.toml:28-29`), so adding a sibling `md-codec` path patch is safe. Verified subtlety: because the miniscript patch is workspace-global, the consumed md-codec (local OR published) compiles against `95fdd1c` inside the toolkit while md-codec's own CI compiles against crates.io `13.0.0` — both miniscript versions have identical mixed-key `into_single_descriptors` behavior, so the C2 builder is correct under both. The fuzz workspace is a separate `[workspace]` (`fuzz/Cargo.toml:20`) whose lockfile needs its own `cargo update` — the plan lists it. Confirmed.

**Version sites — essentially complete; two nits.**
- Confirmed at the cited lines: toolkit `Cargo.toml:36` (md-codec pin), README.md:13, crate README:9, install.sh:32, fuzz/Cargo.lock (md-codec:524, toolkit:574), main Cargo.lock (md-codec:676), CHANGELOG. md-codec/md-cli `Cargo.toml` versions + the shared `descriptor-mnemonic/CHANGELOG.md`.
- **md-cli bump IS warranted (not NO-BUMP):** md-cli's `Cargo.toml:28` exact-pin `md-codec = "=0.36.0"` must change to `=0.37.0` — that is a manifest/dependency change requiring a release, plus the regression test. So `0.7.0 → 0.7.1` PATCH is correct. (md-cli `0.7.0` was itself just released 2026-06-15 per the CHANGELOG; HEAD is at 0.7.0, so the next is 0.7.1.)
- SemVer md-codec MINOR is correct: new `pub fn has_hardened_use_site` + new `pub fn to_miniscript_descriptor_multipath` are additive; `has_hardened_use_site` confirmed not-yet-existing; new error variants are additive. `0.36.0 → 0.37.0` correct.

**TDD / funds-safety oracle — actionable.**
- Each phase has a RED-first test (P1.1 truth table, P1.2 per-key address, P1.3 C2 string, P1.4 decode rejects, P2.1 routing, P2.2 faithful, P2.3 guard matrix, P2.4 advisory parity, P2.5 differential).
- I1 independent golden is concretely actionable: the md-codec differential (`bitcoind_differential.rs:681,738`) is self-referential (both sides from the same render), anchored only by the `wpkh` BIP-84 `[I3c]` golden (`:748-756`). The plan models the new divergent-`@1` golden on `[I3c]`, computed offline from the corpus xpub at `<2;3>/0`. Sound. The toolkit `derive_receive` (`bitcoind_differential.rs:203`, rust-miniscript `into_single_descriptors`) is a genuine independent end-to-end oracle.

## Minor (non-blocking; M1-M2 worth folding)

- **M1 — SPEC §5.6 Row 2 (`sh(multi)` bare-P2SH override → faithful) has NO test in plan §5.** This is a *distinct routing mechanism* — `plain_template_from_tree` matches only `Wsh`/`Sh→Wsh` (`restore.rs:1163-1182`), so a bare `sh(multi)` override returns `None` and reaches the faithful arm via a different code path than the wsh shapes the plan tests. Recommend adding one `sh(multi)` divergent-override case to P2.2/P2.5 to pin that row. (Faithful arm is shape-agnostic, so this is coverage hygiene, not a correctness hole — hence Minor.)
- **M2 — `sh(wsh(multi))` divergent is not explicitly tested either** (P2.2 tests `wsh(multi)` + `wsh(sortedmulti)`; SPEC §5.6 Row 1 includes `sh(wsh(...))`). Fold one `sh(wsh(multi))` divergent case alongside M1, or note that the wsh coverage transitively exercises the inner reconstruction.
- **M3 — CHANGELOG wording.** Plan §P1.6/§4 says "CHANGELOGs (md-codec: …; md-cli: …)" implying two files; descriptor-mnemonic has ONE shared `CHANGELOG.md` with crate-prefixed entries (`## md-codec [X] / ## md-cli [Y]`). Reword to "two prefixed entries in the shared CHANGELOG.md."
- **M4 — install.sh md-cli pin (cross-repo lockstep).** Toolkit `scripts/install.sh:35` pins `descriptor-mnemonic-md-cli-v0.6.2` while md-cli is already at `0.7.0` (pre-existing lag). RELEASE_CHECKLIST:67 says an md-cli release should bump toolkit `install.sh:35` in lockstep. The plan only updates `install.sh:32` (the toolkit self-pin). Optionally fold an `install.sh:35` md-cli pin update (or explicitly note it's deferred, consistent with the repo's "install.sh lag is non-blocking" stance).
- **M5 — Display round-trip guard for multipath (informational, no action).** The faithful arm's `:1365` guard (`parsed.to_string() != descriptor`) must survive a multipath descriptor. This already holds today (the existing translator emits `MultiXPub` keys at `:1079-1084` and general-policy faithful tests pass the same guard), so C2 introduces no new round-trip risk. Worth a one-line note in P2.2 that the existing display-fidelity guard is relied upon, unchanged.

## Bottom line

GREEN — 0 Critical, 0 Important. The C2 multipath-builder API is the correct choice and is provably sound against the `into_single_descriptors` mixed-key handling, the `MultipathDescLenMismatch` invariant (closed by `validate_multipath_consistency`), and the post-derivation sortedmulti sort. C1 routing, the translator reduction (with network correction + strict-NUMS preserved), the guard/advisory parity predicates, the md-codec-first publish ordering, the `[patch]`-then-pin mechanic, and the version-site list are all verified. Fold M1 (and ideally M2) for §5.6 matrix completeness; M3-M5 are hygiene. None gate.

---

## Post-review fold note (this session)
Folded all 5 Minors: M1 (`sh(multi)` bare-P2SH divergent test) + M2 (`sh(wsh(multi))` divergent test) added to P2.2 TDD + §5 item 7; M3 (shared CHANGELOG.md wording) in P1.6; M4 (install.sh:35 md-cli pin note) in P2.6; M5 (display round-trip guard note) in P2.2. No design change. Re-confirmed GREEN in round 2.
