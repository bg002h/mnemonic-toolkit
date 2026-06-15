# Plan-R0 (P0 foundation) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, Opus 4.8). **Verdict: NOT GREEN — 2 Critical /
> 1 Important / 2 Minor.** Plan SHA at review: toolkit `1913027`.

---

**Critical 1.** `mnemonic_toolkit::format` is NOT a public module in normal builds — `pub mod format` is `#[cfg(fuzzing)]`-gated (lib.rs:142-143). The conformance test's `use mnemonic_toolkit::format::{render_grouped, strip_display_separators}` (Task 3) will fail to compile (`unresolved import`/private module) in every normal build. The plan's "verified at plan-write time — `lib.rs:143 pub mod format;`" cites the line but omits the `#[cfg(fuzzing)]` on line 142 that disables it.

**Critical 2.** Task 2 Step 2/4 fail-first + pass commands use `cargo test -p mnemonic-toolkit --lib render_grouped`. Because `format.rs` is a BIN module (not in the lib in normal builds), `--lib` selects ZERO tests and exits 0 — no meaningful red-first, and a false-green at Step 4. MEMORY.md cycle-B records this exact trap ("`cargo test --lib friendly` runs ZERO tests → use `--bin mnemonic`"). Correct target is `--bin mnemonic` OR move the fns to a real lib module.

**Fix for both Criticals:** add an UNCONDITIONAL lib module. Preferred (option a): extract the three pure fns into a new thin module `src/display_grouping.rs` declared `pub mod display_grouping;` (unconditional) in `lib.rs`; this keeps the bin-private heavy API (`BundleJson`/`engraving_card_unified`/…) out of the public lib surface, honoring the crate-shape policy (lib.rs:12 "binary modules stay private to `main.rs`"). Option b (make all of `format.rs` unconditionally `pub`) exposes that heavy API and conflicts with the policy. The plan must pick one explicitly; currently it picks neither and ignores the gate. (Confirmed `format.rs` uses no `use crate::` paths — only `serde`/`mk_codec`/`md_codec` externals — so extraction is clean.)

**Important 1.** Task 2 Step 4's three test commands all use `--lib` (render_grouped / strip_display_separators / render_then_strip) → same zero-tests false-green. Fix to the chosen target (`--lib` once the fns live in a real lib module, else `--bin mnemonic`).

**Minor 1.** Task 3 `sep_char("none")` returns `' '` with comment "// unused by render(gs=0)"; `sep_char` IS called, the returned char is just ignored by `render_grouped`'s early return. Reword to "returned value ignored by render_grouped when group_size==0".

**Minor 2.** Task 3 Step 2's check `grep -n "pub mod format"` → expected `143:pub mod format;` is a FALSE check: it passes while the `#[cfg(fuzzing)]` on line 142 makes the module unreachable, so the worker reads green then hits a compile error at Step 3. Use `grep -n -B1 "pub mod <module>"` and expect to see NO `#[cfg(fuzzing)]` immediately above.

**Verdict: NOT GREEN — 2 Critical / 1 Important / 2 Minor.** Both Criticals stem from the `#[cfg(fuzzing)]`-gated `pub mod format`. Resolution: add an unconditional `pub mod display_grouping;` (new thin module holding the three fns), update the conformance test's `use` path, and fix all `--lib` test commands to target the lib module. Revise before execution.
