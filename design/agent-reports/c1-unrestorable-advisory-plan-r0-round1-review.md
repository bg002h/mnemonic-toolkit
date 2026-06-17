# C1 (unrestorable-shape advisory) plan-R0 round 1 — architect review (verbatim)

> Reviewer: opus architect (general-purpose, full tools — built + ran the binary). Plan-doc:
> `design/PLAN_C1_unrestorable_shape_advisory_2026-06-16.md` @ toolkit `342b5c1`. Verdict RED
> (0C/2I); both Importants + 3 Minors folded post-review (see footer).

---

## R0 Design Review — PLAN_C1_unrestorable_shape_advisory_2026-06-16.md

**Verdict: RED (0C / 2I)**

Citations all check out, the open design risk (shapes 2/3 constructibility) resolved cleanly in the plan's favor, but two Important findings block GREEN: a predicate completeness bug (false-positive on `sh(sortedmulti)`) and a factually-wrong, possibly-incomplete scope claim (import-wallet is a second md1-engraving surface).

---

### Item 2 — THE OPEN DESIGN RISK: shapes 2/3 bundle-constructibility — RESOLVED (both reachable)

I built the binary (`target/debug/mnemonic`, v0.57.0 @ `342b5c1`) and drove real `bundle --descriptor` invocations with the abandon/legal test phrases, then `restore --md1` of each emitted card:

- **Shape 3 (hardened wildcard):** `bundle --descriptor "wsh(multi(2,@0/*h,@1/*h))" --network mainnet --slot @0.phrase=… --slot @1.phrase=… --json --no-engraving-card` → exit 0, emits 4 md1 cards. `restore --md1 <cards>` → exit 2, hits exactly `restore.rs:1254`: *"this md1 uses a hardened wildcard (`/*h`)… Tracked: restore-md1-per-key-use-site-and-hardened-wildcard"*. **`use_site_path.wildcard_hardened` IS set on the engraved card. Bundle-reachable.**
- **Shape 2 (use-site overrides):** `bundle --descriptor "wsh(multi(2,@0/<0;1>/*,@1/*))" …` → exit 0, 4 cards. `restore --md1` → exit 2, hits exactly `restore.rs:1247`: *"this md1 carries per-cosigner use-site path overrides…"*. **`tlv.use_site_path_overrides.is_some()` IS set. Bundle-reachable.**

Root cause confirmed in source: the three bundle hook sites all call `parse_descriptor::parse_descriptor` (`bundle.rs:1601`, `:1701` via `descriptor_concrete_to_resolved_slots`, `:1946`), and that function populates `wildcard_hardened` from a placeholder's `/*h`/`/*'` suffix (`parse_descriptor.rs:122-125`, `:234`) and `use_site_path_overrides` when any `@i≥1` use-site differs from `@0` (`:195-201`, `:798-805`). **Both shapes are genuinely bundle-constructible — the make-or-break risk passes.** No need to drop any shape or build md1 directly.

Shape 1 also re-verified: GAP-3 descriptor bundles (exit 0) and restore refuses with both substrings `"sole child"` + `"faithful backup"` present.

---

### Item 3 — IMPORTANT #1: shape-1 predicate is INCOMPLETE → false-positive on `sh(sortedmulti(...))`

`plan §"Shape-1 predicate"` enumerates only **two** restorable SortedMulti positions: (a) sole `wsh` child, (b) sole `sh→wsh` grandchild. But md-codec 0.36.0 `to_miniscript.rs` accepts SortedMulti in **three** positions:

- `:205` `new_wsh_sortedmulti` — `wsh(sortedmulti)`
- `:231` `new_sh_wsh_sortedmulti` — `sh(wsh(sortedmulti))`
- **`:248` `new_sh_sortedmulti` — `sh(sortedmulti)` (bare legacy P2SH; the plan OMITS this)**

The toolkit emits this shape as `Tag::Sh` with a **direct** sole child `Tag::SortedMulti` (no intervening `Wsh`) — proven by its own test `parse_descriptor.rs:1511 walk_sh_sortedmulti_root`. The plan's algorithm does **not** strip the bare `sh(sortedmulti)` wrapper, so the walk finds `Tag::SortedMulti` and **fires — a false positive, violating "fire IFF restore refuses."**

Empirically confirmed: `bundle --descriptor "sh(sortedmulti(2,@0,@1))"` → exit 0, and `restore --md1` of the emitted card → **exit 0**, reconstructs faithfully.

**Fix:** the predicate's strip set must mirror all three acceptance arms — add the `Tag::Sh` → sole-child `Tag::SortedMulti` case to the "non-firing" set. Add a clean-negative test for `sh(sortedmulti(2,@0,@1))` (must NOT fire; restore exit 0). The plan's negative list only had the `wsh` sole-child case; it should add both `sh(wsh(…))` and `sh(…)`.

Other shape-1 claims correct (verified): `multi`-in-combinator restores fine; `wsh(sortedmulti)` and `sh(wsh(sortedmulti))` restore fine; bare `sortedmulti(...)` rejected by bundle pre-engraving; excluding `SortedMultiA`/taproot correct (`to_miniscript.rs:423`).

---

### Item 4 — IMPORTANT #2: scope claim is factually WRONG — `import-wallet` is a second md1-engraving surface that emits shapes 2/3

`plan §"Scope — bundle ONLY"` asserts import-wallet emits no md1 to warn about. **False.**

- `import_wallet.rs:1439` calls `synthesize_descriptor(&p.descriptor, …)` and `:1532` emits `md1: bundle.md1`. `p.descriptor: md_codec::Descriptor` (`:1289`).
- Foreign-format parsers route through the same `parse_descriptor::parse_descriptor` that sets the triggering fields (`wallet_import/bitcoin_core.rs:278`, `electrum.rs:377`, …).
- The older() advisory **already fires at import-wallet** (`import_wallet.rs:1291` `older_advisories_tree(&p.descriptor)`).

Empirically confirmed: `import-wallet --format bitcoin-core --blob -` with a `listdescriptors` envelope carrying `wsh(sortedmulti(2,[…]xpub…/*h,[…]xpub…/*h))#ry7qflrd` → import exit 0, emits md1, `restore --md1` → **exit 2, the same `restore.rs:1254` hardened-wildcard refusal**. A user importing a foreign wallet hits the gap with **zero warning**.

`FOLLOWUPS.md:105` literally says the advisory "needs its own R0 (**where to detect**)." The plan answers bundle-only with an incorrect rationale.

**Fix (pick one; rationale MUST be corrected either way):**
- (a) **Recommended:** hook import-wallet too (at `import_wallet.rs:1291`, after the existing older() emit on `&p.descriptor`), matching older() breadth + the "emits a NEW restore-refusable md1" criterion. Add an import-wallet cross-surface parity test (the bitcoin-core hardened-wildcard fixture is ready).
- (b) If keeping bundle-only, replace the false rationale with a correct deliberate one AND track the deferral. (a) is stronger.

`inspect`/`repair` correctly out of scope (consume existing md1, don't synthesize). Export-wallet correctly excluded (only `miniscript::Descriptor`, no md1). No `convert` emits md1.

---

### Items 1, 5, 6 — verified clean

**Item 1 (citations) — all accurate at `342b5c1`:** timelock_advisory.rs :102/:187/:193 (note :193 is `pub(crate)`); bundle hooks :1665/:1707/:1969 + bindings :1601/:1701/:1946; restore guards :1247/:1254; GAP-3 :684; dual-home lib.rs:170 (under `#[cfg(fuzzing)]`)+main.rs:34; manual :50 + restore refs :954-958/:968-969; md-codec field defs tlv.rs:26 / use_site_path.rs:53; precedent tests cli_older_advisory.rs, cli_bundle_full.rs:191/:228, readme_version_current.rs; FOLLOWUPS :68/:81/:101.

**Item 5 (non-blocking) — satisfied.** All three sites have `stderr: &mut E`; new emit mirrors `let _ = writeln!(stderr, …)` (best-effort, can't flip exit code). Writes before stdout JSON. GAP-3 `.success()` unaffected.

**Item 6 (SemVer/lockstep) — complete.** PATCH right (no clap delta → no schema_mirror, no new ToolkitError variant — message is a `String`). Version sites all at 0.57.0 and all listed. **Caveat:** if Item 4 → (a), the manual + FOLLOWUP flip + CHANGELOG must reflect BOTH surfaces (currently bundle-only).

---

### Minor

- **M1 —** `older_advisories_node` is `pub(crate)` (`timelock_advisory.rs:193`). Make the new shape-1 walk helper `pub(crate)` so module unit tests reach it (cf. `timelock_advisory.rs:307`).
- **M2 —** shapes 2/3 are field reads (`desc.tlv.…`/`desc.use_site_path.…`), not Node-tree walks — can't be unit-tested on a bare `Node`. Cover shapes 2/3 at the CLI cross-surface layer (real bundle/import invocations); keep module unit tests to the shape-1 walk + `message()` forms. State this so the implementer doesn't fight `Descriptor` field fragility.
- **M3 —** dedup is moot (at most one entry per shape — a boolean each). State that `unrestorable_advisories` returns at most one per shape (two `/*h` keys = one `wildcard_hardened` fire) so a test doesn't expect N lines.

---

### Summary

Fix the **2 Important** findings (predicate must also exempt `sh(sortedmulti)`; correct the import-wallet scope claim and hook it), fold the 3 Minors, persist, re-dispatch R0. The core feature is sound and all three shapes are empirically bundle-reachable — not a redesign, two corrections to the acceptance-set mirror and the scope.

---

## FOLD (post-review, by implementer)

- **I1 fixed:** plan shape-1 predicate now mirrors ALL THREE md-codec acceptance arms — non-firing positions are (a) sole `wsh` child, (b) sole `sh`→`wsh` grandchild, (c) sole `sh` child (bare P2SH `sh(sortedmulti)`, `to_miniscript.rs:248`). Clean-negative test list expanded to all three sole-child shapes.
- **I2 fixed (option a):** scope changed `bundle ONLY` → **`bundle` + `import-wallet`** (the two md1-engraving surfaces holding `md_codec::Descriptor`). 4th hook at `import_wallet.rs:1291` (after the older() emit). Manual/CHANGELOG/FOLLOWUP wording updated to both surfaces. Added an import-wallet bitcoin-core `/*h` parity test. export-wallet/inspect/repair/convert correctly remain excluded.
- **M1/M2/M3 folded:** shape-1 walk helper `pub(crate)`; shapes 2/3 covered at CLI layer (not module unit), shape-1 at module layer; `unrestorable_advisories` returns ≤1 per shape (noted).
