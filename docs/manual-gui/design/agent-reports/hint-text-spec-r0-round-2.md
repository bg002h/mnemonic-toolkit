# R0 review — SPEC_gui_hint_text_defaults.md (round 2, scoped convergence)

- **Reviewer:** opus-tier architect, R0 round-2 (gate: 0C/0I). Scope: fold fidelity vs the round-1 prescriptions + fold-introduced drift ONLY. Round-1 verified the design and all recon claims — not re-litigated.
- **Artifact:** `docs/manual-gui/design/SPEC_gui_hint_text_defaults.md` (post-fold).
- **Round-1 findings under verification:** 1 Important (I1) + 4 Minor (m1-m4), all spec-text.
- **Ground truth re-checked against:** `mnemonic-gui` working tree at `master @ 7e9dcca7b740b138e7133c44a9709c4f9010aa66` (exact match to the spec's recon SHA, clean checkout); toolkit working tree incl. the live `mnemonic` binary v0.75.0 (`gui-schema` v5 JSON re-dumped); `docs/manual-gui/pinned-upstream.toml`.

## Verdict

**GREEN — 0 Critical / 0 Important / 0 new Minor. Cleared for build.**

All five folds are faithful to their round-1 prescriptions, every fold-introduced factual claim re-verified at source, and no fold-introduced contradiction was found in the §2.3 ↔ §2.6 ↔ §4 truthfulness chain, the §2.5/§5 zero-consumers claim, or the §3.4 ↔ §7 migration vectors.

---

## Fold-fidelity verification (per prescription)

### I1 → §2.6 rewrite + §4 bullet realignment: **FAITHFUL, facts re-verified live**

Round-1 prescribed (a) 5-of-6 exact v5 mirror, (b) the feerate double-override with the `mnemonic.rs:2231-2236` cite and the v5-emits-`number`/`1` fact, (c) ghost-truthfulness re-anchored on the §2.3 live clap spot-checks + the §9 N3 trust model. The folded §2.6 delivers all three, plus the future-drift-trap paragraph. Spot-verified once more:

- **`src/schema/mnemonic.rs:2231-2236` cite is REAL** (read at the recon SHA): the compare-cost header comment says verbatim that `--feerate` "is a Text widget rather than `FlagKind::Number` because the toolkit accepts decimal values (`f64` clap parser with bounds [0.0, 10000.0]); the GUI's Number kind is i64-only today…". The `FlagSchema` block follows at `:2261-2270` with `kind: FlagKind::Text` (`:2263`) and `default_value: Some("1.0")` (`:2268`) — exactly the double override the spec describes.
- **v5 JSON fact re-confirmed against the live binary** (v0.75.0, newer than round-1's v0.74.0 — the fact is stable across both): `gui-schema` emits `{"name": "--feerate", "required": false, "kind": "number", "choices": null, "default_value": 1}`. Clap side re-read: `compare_cost.rs:32` `#[arg(long, default_value_t = 1.0, value_parser = parse_feerate)]` on `pub feerate: f64` — so `f64::to_string()` → `"1"` as the spec states.
- **Drift-trap claims verified:** `grep -n default_value tests/schema_mirror.rs` → **zero hits** (default strings ungated, as stated). The "NOT kinds" half has an even stronger empirical proof the spec doesn't need to state: the Text-vs-`number` kind divergence exists on master TODAY and `schema_mirror` passes CI — if kinds were gated, master would already be red. The trap warning is correct.
- **§4 bullet realigned consistently:** "anchored on the 6 live clap-attribute spot-checks (§2.3) — NOT on mirror-construction: 5 of 6 mirror the v5 JSON strings exactly (`gui_schema.rs:1184`) …". The `gui_schema.rs:1184` cite re-verified: line 1184 is exactly `let default_value = extract_default_value(arg, &kind);`.
- **No stale affirmative mirror-construction claim survives anywhere in the spec** — both remaining occurrences of the phrase ("NOT by a blanket 'mirror-construction' guarantee" §2.6; "NOT on mirror-construction" §4) are negations. The false verification chain round-1 flagged is fully excised.

### m1 → §2.5 name-collision caveat: **FAITHFUL, cites real, claim NOT weakened**

The new "Flag-NAME collisions (grep caveat)" paragraph matches the prescription: bundle's Number-kind `--account` with its `PinValue` rule (`conditional.rs:229-248` — re-read, real) plus `main.rs:306` hand-seed (re-read: `("--account", FlagValue::Number(0))` at exactly :306) and `main.rs:762` path-hint reader (re-read: `let account_opt … state.number_value("--account")` at exactly :762); `--timestamp` as export-wallet's Timestamp-kind entry. Crucially the caveat **qualifies** the zero-consumers claim ("holds — but only under that scoping") rather than weakening it — conditional fns and FormState ARE subcommand-scoped (round-1 Claim 4), so §2.5's conclusion and §5's restatement ("Per §2.5: zero conditional rules…") remain true as written. The caveat also dovetails with §3.4(c)'s "bundle's `--account` `Number(0)` hand-seed untouched" — internally consistent.

### m2 → §3.4 mechanics + third §7 vector: **FAITHFUL, cross-references consistent both directions**

§3.4 now specifies exactly the four prescribed mechanics: (a) per-subcommand schema lookup keyed by the persisted `tab:sub` key; (b) fail-open on unknown subcommand/flag (never destructive on a lookup miss); (c) kind-scoped to Text/Path entries only, explicitly sparing the `--account` `Number(0)` hand-seed and all Number defaults; (d) post-fix `Text("")`/`Path("")` autosave entries do NOT equal the default and MUST NOT be dropped. §7's persistence-migration test block carries exactly the three vectors — `Path("-")` dropped, `Path("/tmp/x")` survives, `Path("")` survives "(§3.4d)" — with the forward-reference "(third §7 migration test vector)" in §3.4(d) and the back-reference "(§3.4d)" in §7 both present and pointing at each other correctly.

### m3 → cite `:30`: **FAITHFUL**

Spec header line 6 now cites `pinned-upstream.toml:30`; re-verified: line 30 is `tag = "mnemonic-gui-v0.54.0"`.

### m4 → §7 anti-tautology note: **FAITHFUL**

The ghost-presence test now states the prescribed anchor verbatim in substance: egui paints `hint_text` WITHOUT entering the text buffer, so the AccessKit assert is `value == ""` while the snapshot PNG simultaneously shows the ghost, tying the `.gui` `<hint:d>` notation to real widget behavior rather than the renderer's own output.

---

## Fold-introduced-drift hunt

- **§2.3 ↔ §2.6 ↔ §4 truthfulness chain: coherent.** §2.3's feerate row (GUI Text `"1.0"` at `:2261`; clap `default_value_t = 1.0` at `compare_cost.rs:32`) was re-verified live this round on both sides; §2.6 anchors on those spot-checks + N3; §4 restates the identical anchoring with the identical 5-of-6/override split. No sentence anywhere still derives ghost-truthfulness from the mirror.
- **"Typing the literal `1.0` parses identically" (§2.6): true on both paths** — `is_at_default` (Text `s == "1.0"`) suppresses it → omitted → clap default 1.0; and even if emitted, `parse_feerate("1.0")` = 1.0. Consistent with §3.2's "ghost text is the literal `default_value` string" and §3.3's unchanged-assembler design.
- **§3.4 ↔ §7: no contradiction** (verified above). The migration remains load-time-only, semantics-preserving per D33 + zero readers — unchanged from the round-1-verified design.
- **§2.5 caveat vs §5:** no weakening (verified above); the implementer-grep warning is accurate and will prevent exactly the false alarm it predicts.
- **No design-bearing text changed.** All folds are spec-text/citation additions; scope (§3), argv invariant (§4), phases (§6), test plan (§7 additions only additive), and non-goals (§9) are otherwise untouched from the round-1-verified state.

## New findings

### Critical — none.
### Important — none.
### Minor — none new.

(Informational, no action required: the "NOT kinds" claim in §2.6's drift-trap is additionally provable by the live master-green divergence noted above; and the round-1 nit that §2.3 cites the feerate block-open line `:2261` while the name field sits at `:2262` was already accepted in round 1 as a block-open cite — both left as-is.)

---

## Gate

**GREEN (0C / 0I) — R0 converged; cleared for build.** Proceed to plan/implementation per the standing pipeline: Phase G (mnemonic-gui PR, TDD tests-first per §7) → tag `mnemonic-gui-v0.55.0` → Phase M (manual pin-bump + regen) → Phase R (mandatory post-impl whole-diff adversarial review).
