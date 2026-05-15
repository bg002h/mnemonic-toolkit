# Phase P1.5 (Track G — manual_anchor_coverage RED) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-help-icons` (mnemonic-gui)
**Scope:** §3.1 P1.5 sub-phase — `mnemonic-gui/tests/manual_anchor_coverage.rs` (NEW, ~215 LOC).

**Verdict:** **LOCK 0C / 0I / 2N / 1n.**

kebab() implementation matches SPEC §2.2 byte-for-byte against the Python reference across all 5 documented unit-test cases. expected_anchors() walks the same three variant-bearing `FlagKind` variants as the Python extractor. The 459 = 28 + 161 + 270 arithmetic is byte-identical to Track M's lint output, confirming the bidirectional invariant holds at the inventory level. `#[ignore]` gating is correctly applied per `[[feedback-default-cargo-test-runs-sibling-dependent-tests]]`. The module-level comment defensibly documents the Option-B-over-A trade-off (compile-clean hand-compute vs compile-fail helper-call) and the P2.1 migration plan.

---

## Critical

None.

## Important

None.

## Nice-to-have

### N-1 — `kebab(v)` where `v` is `&&str` — Rust auto-derefs, but `kebab(*v)` would be marginally clearer

(lines 105-106) Functionally correct; trivial stylistic preference.

### N-2 — `missing.sort(); missing.dedup()` — works because `dedup` only removes consecutive duplicates after sort

(lines 161-167) Correct, but a `BTreeSet` could be slightly clearer. Stylistic only.

## Nit

### n-1 — Mixed positional vs captured args in `format!`

(line 92 positional `"{}-{}", tab, kebab(sub.name)`; line 96 captured `"{sub_anchor}-{flag_name}"`). Cosmetic only.

---

## Verification trace

1. **kebab() vs SPEC §2.2:** all 5 unit-test cases PASS. Traced edge cases: `"--from-"` → `"from"`; `"a/b//c"` → `"a-b-c"`; `"---"` → `""`; `"?test?"` → `"test"`; `"BIP49"` → `"bip49"`. Byte-identical to Python reference.

2. **expected_anchors() vs Track M build_expected():** both apply identical formulas. Match arm at lines 99-103 covers `Dropdown | NodeValueComposite | TaggedOrIndexed` — verified against `src/schema/mod.rs:84-106`.

3. **459 = 28 + 161 + 270 arithmetic:** independently recounted from `expected_gui_schema_inventory.json`. Byte-identical to the panic message.

4. **`#[ignore]` gating:** verified at lines 135-138; `cargo test --test manual_anchor_coverage` shows `1 ignored; 0 failed; 5 passed`; `-- --include-ignored` triggers the runtime panic correctly.

5. **`collect_html_ids` parser:** plain-string scan handles all relevant edge cases. Single-quote IDs unmatched (pandoc HTML5 default is double-quote). Over-collection from comments/style/script is harmless for direction A check; direction B orphan check is delegated to the Python lint.

6. **Design rationale (Option B vs A):** module comment at lines 16-46 captures the trade-off. Defensible per `[[feedback-r2-blocking-vs-cosmetic-gate]]` — compile-fail RED would prevent the test from running at all; runtime-fail RED still exercises the formula.

7. **SPEC §2.2 formula trace:** `kebab("simplifiedchinese")` → identity. `kebab("phrase")` → `"phrase"`. `kebab("xprv")` → `"xprv"`. All correct.

8. **Off-by-N drift:** 459 on Track G side matches Track M side exactly. Bidirectional invariant holds.

---

**Final verdict:** **LOCK 0C / 0I / 2N / 1n.** Track G P1.5 RED in place + failing for the expected reason. Both Track M (P1.1 + P1.2 + P1.3) and Track G (P1.4 + P1.5) tracks have reached the P1 parity gate per §3.1: all 5 RED suites compile, default-skip where appropriate, and fail with the expected counts when activated.
