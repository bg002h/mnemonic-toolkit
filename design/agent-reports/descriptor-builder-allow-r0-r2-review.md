# R0 review — SPEC_descriptor_builder_allow — round 2
**Verdict: GREEN** (0C / 0I / 3M)

The folds are real, correctly applied, and introduce no new contradiction. The C1 posture is fully specified and unambiguous; the I1 cell now matches the verified construction verbatim; the I2 cell's prescription is correct — and round-2 probing found it is correct for a *stronger* reason than round 1 claimed (the r1 review's "clap error today" was empirically wrong; see M-r2-1, which does not invalidate the SPEC text). All remaining findings are precision nits.

## Round-1 fold verification (C1, I1, I2, M1-M6)

**C1 — RESOLVED.** §3 pins a *deterministic skip* ("MUST NOT attempt the cost preview (deterministic skip, not try-and-catch)") keyed on `allowed_fired` non-empty.
- *Null-vs-omit:* unambiguous — §3 says `--json` emits `"cost": null` and §5 pins "`cost` is `null`"; round 1's "(or omits the key)" alternative was resolved to explicit null. The envelope key is literally `"cost"` (`crates/mnemonic-toolkit/src/cmd/build_descriptor.rs:302`), so the SPEC names the right key. Null-not-omit is also the GUI-friendlier choice, and §6 consistently routes a "`cost: null` un-gated wire note" through the A1 FOLLOWUP extension.
- *Coexistence with the address line:* no conflict. The human one-liner replaces only the cost block (`build_descriptor.rs:350-361`, the fatal `run_compare_cost` site); the address line is independent and already best-effort (`:343` `if let Ok`), so exit 0 holds by construction whether or not the address derives. The skip having access to `allowed_fired` is structurally sound: `ValidatedPolicy` (which §2 extends) is exactly what `emit()`/`emit_human()` consume, and it is constructed nowhere outside gate.rs (grep confirmed).
- *Per-mode cells:* present at §5 (`--json` null / human one-liner / `--format descriptor` bare), plus the bip388 cell covering the fourth mode. `--format` paths confirmed cost-free in source (`emit()` match, `:315-329`). `compare-cost` strictness retained (§3) — and remains necessary: probed exit 3 on the NEW keyed mixed-timelock tree too (probe P3).

**I1 — RESOLVED.** §5 carries the keyed tree verbatim: `and_v(v:pk(K1), and_v(v:older(100), older(4194304)))`. Independently re-probed (P1): exit 2, SOLE diagnostic `mixed_timelock` at `root.and_v[1]` — byte-identical to r1 Probe C. The keyless variant independently re-probed (P2): `sigless_branch` at `root.and_v[0].wrap.sub` — the SPEC's "(The keyless variant refuses `sigless_branch` FIRST — wrong cell.)" parenthetical is exact.

**I2 — RESOLVED** (see M-r2-1 for a rationale refinement). §5 rewrites the cell as a preset invocation: kofn + duplicate `--key` + `--allow repeated-keys --emit-spec`, replay via `--spec -` without `--allow` → exit 2; "(No clap-edge changes to `--emit-spec` in this cycle.)" The substrate is live (probe P6): kofn + dup `--key` + `--emit-spec` today → exit 2, `repeated_keys` at `root.or_d[0]`, `(from --key)` provenance — so with `--allow repeated-keys` it passes the gate and reaches the `emit_spec` branch (`build_descriptor.rs:196-207`). The replay direction is pinned by my P4 (the same tree via `--spec` refuses `repeated_keys`).

**M1 — RESOLVED.** §0 now attributes to `design/SPEC_descriptor_builder_engine.md:59`, whose text I verified verbatim: "The reviewed escape hatch for a deliberately 'insane' policy is the existing raw `--descriptor` intake door." The corrected paraphrase (raw door only; "`--spec` runs the same gate") is accurate — both `gate::validate` call sites serve both input modes. Nit: of the claimed echoes, `:95` genuinely echoes the raw-door/deferred-`--allow` route; `:146` echoes only the cut-`--allow`/degrading-threshold story without naming the raw door. Harmless.

**M2 — RESOLVED.** §0 re-grounds vacuousness on the IR render surface ("no raw-pkh node; `render()` can never emit `expr_raw_pkh`; the string parser CAN parse it — an input-space property, not a parser property"). The cited in-repo note exists (it actually begins at `gate.rs:274`, "The IR has no raw-pkh-without-key node…"; the SPEC's `:277-284` anchor is 3 lines low — nit only, content found).

**M3 — RESOLVED.** §2 pins polarity: 3 negated (`!requires_sig`, `!is_non_malleable`, `!within_resource_limits`), 2 direct (`has_repeated_keys`, `has_mixed_timelocks`) — verified an exact match against the shipped `localize_sanity` dispatch (`gate.rs:261-273`). The "pre-computed type/ext data" soundness claim re-verified at the pinned rev (`analyzable.rs:187` `ty.mall.safe`, `:190` `ty.mall.non_malleable`, `:195` `Ctx::check_local_validity`, `:198` `ext.timelock_info`, `:201` pure `iter_pk` walk).

**M4 — RESOLVED.** §3: banner "emitted UNCONDITIONALLY on fired — every output mode including `--json`".

**M5 — RESOLVED.** §5 pins the duplicate-`keys_info` bip388 cell; §4 adds the manual sentence (signer-defined registration behavior). Independently re-probed (P7): 4 `keys_info` entries, the two xpubs each appearing twice, exit 0.

**M6 — RESOLVED.** §6 names the repeating-Dropdown novelty ("a FlagKind combination the GUI schema has not used before (current repeats are Text)") and instructs the stale-companion-line fix. Both anchors verified: the toolkit entry's "(to be filed in the GUI repo)" line exists under `design/FOLLOWUPS.md` (`gui-build-descriptor-presets-pending-pin-bump`, Companion line) and the GUI entry exists at `mnemonic-gui/FOLLOWUPS.md` (first Active entry).

## Critical

None.

## Important

None.

## Minor

**M-r2-1 — I2's stated mechanism is wrong about clap (carried from the r1 review), though the prescription is independently correct; add one sentence.** Round 1 asserted "`--spec` + `--emit-spec` is therefore a clap error today" — probed FALSE: `build-descriptor --spec <file> --emit-spec` is ACCEPTED by clap and runs a normal emit with `--emit-spec` silently ignored (probe P5; same for `--spec … --key foo` — the whole `requires = "archetype"` family). Mechanism: clap drops a conflicting arg (`archetype`, `conflicts_with = "spec"`, `build_descriptor.rs:56`) from the required set when its conflictor is present, so `emit_spec`'s `requires` (`:103`) is never enforced alongside `--spec`; bare `--emit-spec` DOES error (exit 64). This is presumably the "known clap-`requires` nuance from the presets cycle" §1 already alludes to. The SPEC's folded text remains literally true (both attribute claims hold) and the preset-cell prescription is right for a stronger reason: `run()` only honors `emit_spec` inside the archetype branch (`build_descriptor.rs:196`), so a spec-mode `--emit-spec` cell is unimplementable regardless of clap. Recommend: reword §5's parenthetical to that ground truth, and note the pre-existing silent-ignore (`--spec … --allow … --emit-spec` will banner-and-emit a descriptor, not a spec) — it brushes against the SPEC's never-silent ethos; a docs-or-FOLLOWUP one-liner suffices, out of this cycle's scope.

**M-r2-2 — pin the stream for the human cost one-liner.** §3/§5 say the `cost preview unavailable for a sanity-overridden descriptor` line "is present" but not where; the block it replaces prints to stdout (`build_descriptor.rs:350`). Say "stdout, in the cost block's position" so the §5 human cell's assertion is authored against the right stream (the banner, by contrast, is stderr).

**M-r2-3 — two citation nits (cosmetic).** (a) §0's `gate.rs:277-284` — the raw-pkh note begins at `gate.rs:274`. (b) §0's echo anchor `:146` in the engine SPEC echoes the cut-`--allow` supersession but not the raw-door phrase (`:95` does both).

## Empirical probes run

All against `target/debug/mnemonic` 0.51.0 (tracked tree clean at `adab5ac`); miniscript source re-read at the pinned checkout `~/.cargo/git/checkouts/rust-miniscript-*/95fdd1c`.

- **P1** — keyed mixed-timelock spec (`and_v(v:pk(K1), and_v(v:older(100), older(4194304)))`) via `--spec - --json` → exit 2, sole `{kind: "mixed_timelock", node_path: "root.and_v[1]"}` ⇒ I1 fold independently confirmed.
- **P2** — keyless variant via `--spec - --json` → exit 2, `sigless_branch` at `root.and_v[0].wrap.sub` ⇒ the "wrong cell" parenthetical confirmed.
- **P3** — `compare-cost --descriptor` on the keyed mixed-timelock single-path → exit 3, "cannot wrap as Tap: Contains a combination of heightlock and timelock" ⇒ the C1 skip is load-bearing for the NEW I1 cell too.
- **P4** — repeated-keys flagship spec via `--spec - --json` → exit 2, sole `repeated_keys` at `root` ⇒ flagship + replay direction confirmed.
- **P5** — `--emit-spec` alone → clap exit 64 (missing `--archetype`); `--spec <valid> --emit-spec` → exit 0, NORMAL human emit, `--emit-spec` silently ignored; `--spec <valid> --key foo` → likewise accepted/ignored ⇒ M-r2-1 (corrects r1's I2 reasoning; SPEC prescription unaffected).
- **P6** — `--archetype kofn-recovery` + duplicate `--key` + `--emit-spec` → exit 2, `repeated_keys` at `root.or_d[0]`, `(from --key)` ⇒ I2 cell substrate live.
- **P7** — `export-wallet --descriptor <repeated-keys multipath> --format bip388` → exit 0, duplicate `keys_info` (`@0`/`@2`, `@1`/`@3`) ⇒ M5 re-confirmed.
- **Source spot-checks** — envelope key `"cost"` (`build_descriptor.rs:302`); `--format` paths cost-free (`:315-329`); address line best-effort (`:343`); `emit_spec` honored only in the archetype branch (`:196`); `ext_check` order incl. trailing `raw_pkh` arm + the five predicates (`analyzable.rs:242-258`, `:187-209`); `DiagnosticKind::as_str` == §1 table (`gate.rs:96-100`); polarity template (`gate.rs:261-273`); `ValidatedPolicy` constructed only in gate.rs; engine SPEC `:59` verbatim; FOLLOWUP entry wording confirms the supersession claim; stale companion line + GUI entry both present.
