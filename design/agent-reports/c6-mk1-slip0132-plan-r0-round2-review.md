# C6 (revised: stderr-hint) plan-R0 round 2 — architect review (verbatim)

> Reviewer: opus architect (general-purpose, full tools — built binary, minted real cards). Revised
> plan `design/PLAN_C6_mk1_slip0132_stderr_hint_2026-06-17.md` @ toolkit `1a0d0a9`. Verdict GREEN
> (0C/0I); M-1/M-2/M-3 folded post-review (see footer).

---

**Verdict: GREEN (0C/0I)**

The revised design dissolves round-1's Critical C-1 cleanly. Confirmed both make-or-break items empirically. All findings Minor (none gate).

**1. Dissolves C-1 — YES, empirical.** Stdout unchanged: `Mk1` decode emits `card.xpub.to_string()` at `convert.rs:1593`, untouched; `--xpub-prefix` post-process (`:1086-1098`) only fires on explicit flag. Minted a card via `bundle --slot @0.xpub=<real zpub>`, decoded back → stdout neutral `xpub6CatWdiZ…` byte-identical to original. B1/B2/B3 stay green untouched. No residual stdout leak.

**2. Citations — fixed; one residual (M-1).** slip0132.rs:17-28 ✓, :132-137 ✓; convert.rs:1589 ✓, :1593 ✓, :1086-1098 ✓, :300-301 ✓; B1/B2/B3 ✓. mk-codec 0.4.0 crates.io ✓ but at `crates/mnemonic-toolkit/Cargo.toml:35` (plan drops the crate prefix).

**3. `path_implied_xpub_prefix` — sound, network-free; I-4 MOOT.** Mapping correct; returns abstract XpubPrefix, no network; neither new helper calls apply_xpub_prefix. Confirmed `m/48'/0'/0'/2'` origin lands on a card (use a checked 4th-component accessor, not `[3]`).

**4. Suppression — right condition; N-notes moot.** `convert --from mk1` decodes exactly ONE KeyCard per invocation (multisig mk1 is per-cosigner) → at most one note. No dedup needed.

**5. TDD minting premise — REAL.** `bundle --slot @0.xpub=<real zpub>` → normalizes + `origin_path: m/84'/0'/0'` → derives Zpub → note fires. Mint via `bundle` not `convert --to mk1` (hard refusal) is correct.

**6. SemVer/lockstep — PATCH correct, sites complete.** Advisory-only stderr, zero clap/stdout delta → PATCH. No schema_mirror, no new ToolkitError variant. Version sites all at 0.58.0 (both READMEs, install.sh:32, fuzz/Cargo.lock:575, main Cargo.lock:727). FOLLOWUP:4206 flip recording the product-question outcome is correct.

**7. Useful + accurate.** Fires only on cards whose path conventionally implies a variant AND no form chosen — actionable, not nagging. Wording accurate (intake normalizes zpub→xpub; card carries neutral). Code-not-doc justified (appears at the read-back moment).

## Minor
- **M-1** — citation: `crates/mnemonic-toolkit/Cargo.toml:35` (not root). Prefix-omission class.
- **M-2** — suppression predicate must be `args.xpub_prefix.is_some()`, NOT `is_some_and(|p| !p.is_default())`: a user who passes `--xpub-prefix xpub` (neutral) has chosen a form; `is_default()`-true would wrongly re-fire. Pin to `is_some()`.
- **M-3** — plumbing: `compute_outputs` (`:1185-1192`) takes no stderr writer (returns `(Vec<Output>, Option<&'static str>, Option<SeedVersion>)`; `run()` writes stderr). Slot-2 is occupied by `input_variant` (rendered unconditionally via `render_slip0132_info_line :1061-1066`) — do NOT overload it. Add a dedicated 4th return slot (e.g. `Option<XpubPrefix>` path-implied hint) that `run()` renders after the input_variant block, gated on `args.xpub_prefix.is_none()`. Mechanical; doesn't change design.
- **M-4** — optional: emitting the note on stderr under `--json` is fine (stdout JSON stays clean); just don't assert stderr-empty under --json in tests.

Empirical log (real binary, v0.58.0): zpub→`bundle --slot @0.xpub=`→`origin_path: m/84'/0'/0'`; that card `--to xpub`→neutral `xpub6CatWdiZ…` byte-identical, stdout unchanged; `m/48'/0'/0'/2'` origin lands on card; multisig mk1 per-cosigner (one card per convert → one note).

---

## FOLD (post-round-2, by implementer)
- **M-1:** citation table → `crates/mnemonic-toolkit/Cargo.toml:35`.
- **M-2:** design → suppression predicate is exactly `args.xpub_prefix.is_none()` (note fires only when NO `--xpub-prefix` at all, including the explicit-neutral `--xpub-prefix xpub` case which suppresses).
- **M-3:** design → `compute_outputs` gains a 4th return element `Option<XpubPrefix>` (path-implied hint); `run()` emits the note after the existing `input_variant` stderr block, gated on `args.xpub_prefix.is_none()`. Do NOT overload slot-2 (`input_variant`).
- **M-4:** TDD cells will not assert stderr-empty under `--json`.
