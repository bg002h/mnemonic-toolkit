# IMPLEMENTATION PLAN — faithful `restore --md1` of per-cosigner use-site overrides

**Date:** 2026-06-19
**SPEC:** `design/SPEC_restore_per_key_use_site_override_2026-06-19.md` (R0 GREEN, `5e55603`).
**Source SHAs (grep-verified):** descriptor-mnemonic `c85cd49` (main), mnemonic-toolkit `5e55603` (master).
**Versions:** md-codec `0.36.0 → 0.37.0` (MINOR, publish), md-cli `0.7.0 → 0.7.1` (PATCH, dep bump + test), mnemonic-toolkit `0.58.1 → 0.58.2` (PATCH).
**Cross-repo ordering:** Phase 1 (md-codec + md-cli, descriptor-mnemonic) ships + **publishes 0.37.0 to crates.io** → Phase 2 (toolkit) pins the published `0.37.0`.

## 0. Gate + funds-safety discipline

Each phase is per-phase TDD (RED tests before impl) + a per-phase opus R0 to **0C/0I before advancing**. This is silent-wrong-address funds-safety code: the address-equivalence oracle (with an INDEPENDENT golden — SPEC I1) is the make-or-break gate, not exit-0. No phase tags/ships/advances with an open Critical/Important.

## 1. The C2 API decision (SPEC left this to the plan)

**Chosen: a new md-codec multipath descriptor builder** `pub fn to_miniscript_descriptor_multipath(d: &Descriptor) -> Result<Descriptor<DescriptorPublicKey>, Error>` that builds each key as `MultiXPub` (or `XPub` for a `None`-multipath key) using `ExpandedKey.use_site_path.multipath` per `@N`, where **`@N` == `expand_per_at_n` Vec position** (`canonicalize.rs:339/420`). The toolkit's faithful arm consumes this directly and the `ReconstructTranslator` is reduced to network-correction only.

**Rationale (vs "toolkit consumes a per-`@N` key set"):** the translator's `pk` callback (`restore.rs:1029`) never receives the key's `@N`, and `translate_pk` visits in tree-traversal order ≠ `@N` order in general policies (R0-r2 confirmed) — so any toolkit-side `@N`↔key matching is fragile. Building the multipath descriptor IN md-codec uses the unambiguous `@N`=position correspondence and keeps reconstruction owned by md-codec. The existing single-path `to_miniscript_descriptor(d, chain)` is retained (it is the address-derivation entry; D1 fixes its per-key VALUE); the new builder is the descriptor-STRING entry (C2). Both reuse the per-`@N` `ExpandedKey.use_site_path` from `expand_per_at_n`.

---

## 2. PHASE 1 — md-codec + md-cli (descriptor-mnemonic), ship + publish `0.37.0`

Dev against the local workspace; publish at P1.6. Each sub-phase: RED test(s) → impl → green.

### P1.1 — `has_hardened_use_site` predicate (Point B)
- **Impl:** `pub fn has_hardened_use_site(d: &Descriptor) -> bool` (in `use_site_path.rs` or `derive.rs`) scanning `d.use_site_path` AND every `(_, UseSitePath)` in `d.tlv.use_site_path_overrides` for `wildcard_hardened == true` OR any `Alternative.hardened == true` in `multipath`. Wire into `derive_address` (`derive.rs:92`): replace the baseline-only checks at `:99` and `:110` with `if has_hardened_use_site(self) { return Err(HardenedPublicDerivation) }` (keep the chain-index range check on the resolved per-key path).
- **TDD (RED-first):** predicate truth table — (baseline `/*h`), (override `/*h`), (override hardened ALT, baseline clean), (all unhardened) → expect `true,true,true,false`; `derive_address` returns `HardenedPublicDerivation` for an override-hardened-alt card (today it slips to a generic `AddressDerivationFailed`).

### P1.2 — D1: per-key reconstruction VALUE
- **Impl:** `to_miniscript.rs:60` pass `&e.use_site_path` (not `&d.use_site_path`); in `build_descriptor_public_key` set `wildcard` from `use_site.wildcard_hardened` (replace hardcoded `Wildcard::Unhardened` `:90`). Do NOT touch the `:125-127` hardened-alt reject (SPEC I2 — hardened routes to refusal via P1.1 anyway).
- **TDD:** `derive_address` on `wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))` at chain 0 idx 0 derives `@1` at `2/0` (its own alt0), not `0/0` (baseline). Independent golden (see §4 I1).

### P1.3 — C2: `to_miniscript_descriptor_multipath` (descriptor STRING)
- **Impl:** new `pub fn to_miniscript_descriptor_multipath(d)`; build per-`@N` keys via `expand_per_at_n` → for each `e`: if `e.use_site_path.multipath.is_some()` → `MultiXPub(DescriptorMultiXKey { derivation_paths: DerivPaths from the alts, wildcard from `e.use_site_path.wildcard_hardened`, origin, xkey })`; else → `XPub` (single, no multipath). Then `node_to_descriptor(&d.tree, &keys)`. Share the key-origin/xpub assembly with `build_descriptor_public_key` (extract a helper). (Hardened cards are pre-guarded by callers via P1.1; a hardened alt here still `Err`s through the shared path — acceptable.)
- **TDD:** `to_miniscript_descriptor_multipath` on the divergent card → `.to_string()` carries `@0/<0;1>/*` AND `@1/<2;3>/*` (per-`@N` groups); the `Some`/`None` mix (`@1/*`) → `@1` renders as a single-path `XPub` (bare `/*`) while `@0` stays `<0;1>` multipath; sortedmulti divergent → keys still sort per-index at derivation.

### P1.4 — D5(a)/(b): decode hardening
- **Impl (D5a):** in decode (`decode.rs:57-58` region, post-`expand`/validate), reject any `use_site_path_overrides` entry with `idx == 0` → `Error::BaselineUseSiteOverride { idx }`; reject any override whose `UseSitePath == ` the resolved baseline → `Error::RedundantUseSiteOverride { idx }`. (Two additive variants; `{ idx }` style per `error.rs:137`.)
- **Impl (D5b):** ensure `validate_multipath_consistency` (`validate.rs:117`) treats a `Some`-baseline + `None`-override (and vice-versa) as a recognized legal divergent STRUCTURE (no reject; the C2 builder handles it). Add a doc note; the test is the gate.
- **TDD:** decode of a hand-crafted card with an `@0` override → `BaselineUseSiteOverride`; with a redundant override → `RedundantUseSiteOverride`; round-trip of ALL existing corpus cards still passes (M4 — encoders never emit either).

### P1.5 — md-cli regression
- **Impl:** none beyond the dep bump (md-cli inherits via `derive_address` → `to_miniscript_descriptor`).
- **TDD:** `md address <divergent-card>` yields the CORRECT per-cosigner addresses (independent golden); a `/*h` or override-hardened-alt card → clean `HardenedPublicDerivation` exit (not silent, not generic).

### P1.6 — version + ship + publish
- Bump `md-codec` `Cargo.toml` `0.37.0`; `md-cli` dep `version = "=0.37.0"` (`md-cli/Cargo.toml:28`) + md-cli own version `0.7.1`; **M3:** add TWO crate-prefixed entries to the SHARED `descriptor-mnemonic/CHANGELOG.md` (`## md-codec [0.37.0]` — faithful per-key reconstruction + `has_hardened_use_site` + D5, **funds-safety** framing; `## md-cli [0.7.1]` — inherits + regression). Run md-codec + md-cli full suites + the differential (env-gated bitcoind). **Per-phase R0 (md-codec changes) to 0C/0I.** Then ship descriptor-mnemonic + **publish md-codec 0.37.0 + md-cli 0.7.1 to crates.io.**

---

## 3. PHASE 2 — mnemonic-toolkit, ship `0.58.2`

Dev against a local `[patch]`/path to the Phase-1 md-codec; FINAL-pin to the published `0.37.0` before ship.

### P2.1 — C1: route override cards to the faithful arm
- **Impl:** at the routing site (`restore.rs:1289`, non-taproot branch), gate: `if d.tlv.use_site_path_overrides.is_some() { (None, None) } else { (plain_template_from_tree(&d.tree, &d.use_site_path), None) }`. (Override cards → `template_opt = None` → faithful arm. Taproot override cards never reach here — pre-refused by the P2.3 guard.)
- **TDD:** an override card with a STANDARD `@0` baseline (`wsh(multi(2,@0/<0;1>/*,@1/*))`) routes to the faithful arm (assert it does NOT hit `build_descriptor_string`/the plain template).

### P2.2 — C2: faithful arm consumes the multipath builder
- **Impl:** `faithful_multisig_descriptor` (`restore.rs:1105`) calls `md_codec::to_miniscript::to_miniscript_descriptor_multipath(d)` instead of `to_miniscript_descriptor(d, 0)`. Reduce `ReconstructTranslator` to network-correction only: drop the `multipath` field; `pk` → NUMS `Single` pass-through (+ strict-NUMS refusal, unchanged); `MultiXPub` → set `xkey.network`, return; `XPub` → set `xkey.network`, return; else refuse. Keep the "cannot wrap" error-hint mapping.
- **TDD:** `bundle → restore` of the divergent card → reconstructed descriptor STRING carries `@1`'s divergent suffix (string assertion) AND derives the same addresses as the original (address-equivalence). Cover ALL §5.6 Row-1/Row-2 faithful shapes: `wsh(multi)`, `wsh(sortedmulti)`, **`sh(wsh(multi))`** (M2), **`sh(multi)` bare-P2SH** (M1 — a DISTINCT routing path: `plain_template_from_tree` matches only `Wsh`/`Sh→Wsh` `restore.rs:1163-1182`, so bare `sh(multi)` returns `None` → faithful arm), and the `Some`/`None` mix. **M5:** the existing display round-trip guard (`restore.rs:1365` `parsed.to_string() != descriptor`) is relied upon UNCHANGED — confirm it survives a multipath descriptor (it does today; C2 adds no new round-trip risk).

### P2.3 — guard narrowing + shared predicates
- **Impl:** replace the blanket `:1247` override refusal with: refuse iff `md_codec::has_hardened_use_site(d)` OR `taproot_override_card(d)` (new `fn taproot_override_card(d) -> bool = matches!(d.tree.tag, Tag::Tr) && d.tlv.use_site_path_overrides.is_some()`). Remove the now-subsumed baseline-only `:1254` (superseded by `has_hardened_use_site`). Non-taproot non-hardened override cards fall through to §3 routing (C1→faithful→C2).
- **TDD:** non-taproot non-hardened override → SUCCEEDS faithfully (flip the pinned `per_key_use_site_override_refused` at `cli_restore_multisig_general.rs:414`); `tr(multi_a)` override → REFUSES (taproot guard); override-hardened-alt (non-taproot) → REFUSES (hardened); baseline `/*h` → REFUSES.

### P2.4 — advisory parity
- **Impl (`unrestorable_advisory.rs`):** drop the `PerKeyUseSiteOverrides` arm (`:81-85`) + its enum variant; make `HardenedWildcard` (`:86`) fire on `md_codec::has_hardened_use_site(d)`; add `TaprootUseSiteOverride` firing on the SAME `taproot_override_card(d)` expression the guard uses (M3 — single source). Keep `SortedMultiInCombinator`. Update the messages.
- **TDD:** advisory fires IFF restore refuses, per shape (parity tests at the bundle/import-wallet engrave surfaces): non-taproot non-hardened override → NO advisory + restore succeeds; taproot override → advisory + restore refuses; hardened → advisory + restore refuses.

### P2.5 — differential + corpus
- **Impl/TDD:** add a `wsh(multi)` divergent shape to `tests/bitcoind_differential.rs` corpus (the split logic handles per-key multipath natively — just a corpus entry); assert reconstructed addresses == original via the independent `derive_receive` rust-miniscript oracle, AND assert the reconstructed STRING carries the divergent suffix. Property test in `prop_backup_restore_roundtrip.rs`: divergent-suffix card round-trips faithfully.

### P2.6 — pin + manual + ship
- Pin `md-codec = "0.37"` (toolkit `Cargo.toml:36`); `cargo update -p md-codec --precise 0.37.0`; update `fuzz/Cargo.lock` (separate workspace). Version `0.58.2` + BOTH READMEs (`README.md:13` + `crates/mnemonic-toolkit/README.md:9`) + `scripts/install.sh:32` (toolkit self-pin) + CHANGELOG. **M4:** the md-cli sibling pin at `install.sh:35` is a pre-existing lag (`…md-cli-v0.6.2` while md-cli is `0.7.0`); `RELEASE_CHECKLIST:67` wants it bumped on an md-cli release — fold an update to the new `md-cli-v0.7.1` tag here, OR explicitly note deferred per the repo's "install.sh sibling-lag is non-blocking" stance (decide at execution; not a gate). Update manual `### Unrestorable descriptor shapes` (non-taproot overrides now restorable; taproot + hardened still listed) → run `make -C docs/manual audit` (captured-output discipline). fmt: `cargo +1.95.0 fmt -p mnemonic-toolkit` then `git checkout -- …/mlock.rs`. **Per-phase R0 (toolkit changes) to 0C/0I.** Ship toolkit `0.58.2`.

---

## 4. Cross-repo mechanics, version sites, oracle

- **Dev/publish:** Phase 2 develops against a local md-codec (`[patch.crates-io] md-codec = { path = … }` in the toolkit, mirroring the existing miniscript-fork patch) and FINAL-pins to the published `0.37.0` (remove the patch) before P2.6 ship + the toolkit master push (CI builds against crates.io).
- **Version sites** (per `project_toolkit_release_ritual_version_sites`): toolkit = Cargo.toml + BOTH READMEs + install.sh + fuzz/Cargo.lock + main Cargo.lock + CHANGELOG. md-codec/md-cli = Cargo.toml(s) + CHANGELOGs + crates.io publish.
- **I1 independent golden:** the md-codec differential is self-referential (both sides from the same render) — the divergent shape MUST pin a golden `@1`-address computed OUTSIDE the codec (offline rust-bitcoin/python BIP-32 derive of the test xpub at `<2;3>/0`, documented inline). The toolkit `derive_receive` (rust-miniscript `into_single_descriptors`) is an independent end-to-end oracle.
- **No GUI:** no clap flag/dropdown change ⇒ no `schema_mirror`, no manual flag-coverage gate (only prose). No `ToolkitError` variant (toolkit uses `bad(...)`/`ModeViolation`).
- **FOLLOWUPS:** descriptor-mnemonic companion entry for the md-codec faithful-reconstruction change; the taproot deferral FOLLOWUP is already filed (`4783f02`).

## 5. Consolidated funds-safety test inventory (all RED-first)
1. md-codec: `has_hardened_use_site` truth table; `derive_address` hardened-override clean reject.
2. md-codec: D1 per-key address (divergent) — independent golden.
3. md-codec: C2 multipath STRING per-`@N` groups + `Some`/`None` mix + sortedmulti divergent.
4. md-codec: D5(a) `@0`/redundant decode reject; corpus round-trip unbroken.
5. md-codec differential: divergent `wsh(multi)` + `tr(multi_a)` + `Some/None` shapes with INDEPENDENT goldens.
6. md-cli: `md address` divergent = correct; hardened = clean reject.
7. toolkit: C1 routing; C2 faithful STRING + address-equivalence across the §5.6 faithful shapes — `wsh(multi)`, `wsh(sortedmulti)`, `sh(wsh(multi))` (M2), `sh(multi)` bare-P2SH (M1), `Some`/`None` mix; guard (succeed/refuse matrix); advisory parity; differential corpus entry; flip the refused-pin; the `restore.rs:1365` display round-trip guard survives multipath (M5).

## 6. Risks / per-phase R0 focus
- **The whole point is no silent-wrong-address.** R0/impl-review must verify: the §5.6 shape matrix holds end-to-end; the independent golden actually anchors divergence (not codec self-agreement); the translator reduction doesn't drop network-correction or the strict-NUMS refusal; guard+advisory predicates are the SAME expressions (parity).
- **C2 builder + sortedmulti:** confirm a multipath `sortedmulti` descriptor sorts per-index correctly at `into_single_descriptors` (rust-miniscript owns this).
- **Publish ordering:** toolkit CI must build against the PUBLISHED 0.37.0 (drop the local patch before the master push) — else CI red.
- Per-phase R0 is mandatory before P1.6 publish and before P2.6 ship.

## 7. Citation-decay
Line numbers are snapshots at the header SHAs; re-grep against current `origin/master`/`origin/main` at execution time (CLAUDE.md discipline).
