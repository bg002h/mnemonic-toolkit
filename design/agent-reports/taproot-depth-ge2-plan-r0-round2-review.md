# Plan-doc R0 round 2 — architect review (verbatim) — PLAN_taproot_depth_ge2_experimental.md

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect, Opus 4.8).
> **Verdict: GREEN (0 Critical, 0 Important, 2 Minor).** The plan-doc gate is MET. The 2 Minors
> (K3-derivation template name; the :1360 comment-refresh entry) folded before the build.

---

## Pre-verdict working notes (verbatim, abridged)
All cited line numbers verified: `Cargo.toml:29`, `fuzz/Cargo.toml:28`, `restore.rs` gate fn `:819` + call `:748`, `older()` advisory `:1373-1374`, `is_taproot` `:1281`, `cli_restore_taproot.rs` cells `:263`/`:284`. md-codec `Body::Children(Vec<Node>)` + `Body::Tr{tree:Option<Box<Node>>,..}` confirmed (snippets compile).

The K3 derivation guidance said `--template bip84`, but K0/K1/K2 are BIP-87 (`87'/0'/0'`). `mnemonic convert --to xpub` emits a bare xpub (no `[fp/path]` prefix / `/<0;1>/*` suffix) — the implementer assembles the wrapper. Non-blocking (surrounding text conveys the goal format) → minor.

## CRITICAL — None.
## IMPORTANT — None.

## MINOR
**m-r2-1.** K3 derivation guidance said `--template bip84`; K0/K1/K2 are BIP-87 (`87'/0'/0'`). Should be `bip87`, and `convert --to xpub` gives a bare xpub (assemble the `[fp/path]…/<0;1>/*` wrapper). Non-blocking. **Fix:** correct to bip87 + note the assembly. (Task 2 Step 1)
**m-r2-2.** The comment-refresh list omits `restore.rs:~1360` — the Display-fidelity-guard comment "the known depth-2 taptree bug is structurally pre-gated in §3; this catches any future parseable variant" becomes inaccurate after the lift (depth-≥2 is no longer pre-gated; the fidelity guard is now the primary net). **Fix:** add it to the refresh list. (Task 2 Step 3)

## Fold verification (I-1, m-1, m-3 from round 1)
- **I-1 LANDED + CORRECT.** `tr(NUMS,{{{pk(K0),pk(K1)},pk(K2)},pk(K3)})` is genuinely depth-3 (3 TapTree nesting levels before a leaf), 4 distinct keys → distinct-key gate clear → `bundle --descriptor`-acceptable. Exercises a third level of `ensure_taptree_wellformed` recursion.
- **m-1 LANDED + CORRECT.** Step 2 now says `cargo build` is authoritative; `cargo metadata` may error "needs update" → build is the fallback.
- **m-3 LANDED + CORRECT.** Step 4 now says capture goldens by running the restore CLI directly (assert_cmd captures subprocess stdout).

## Load-bearing re-checks
1. depth-3 shape correct + bundle-acceptable with 4 distinct keys — confirmed.
2. All snippets (`ensure_taptree_wellformed`, `taproot_is_deep` let-else, advisory `writeln!`) compile + correct — confirmed.
3. TDD ordering sound (T1 rev bump leaves OLD cells green; T2 Step1 flip → RED; Step3 lift → Step4 GREEN) — confirmed.
4. No new issue introduced by the folds.
5. Spec coverage complete (gate lift, advisory, EXPERIMENTAL marking, fidelity-guard retention, test cells all owned).

## VERDICT: GREEN
0 Critical / 0 Important. Two low-impact minors (bip84→bip87 slip; the :1360 comment-refresh entry), folded. The plan is implementable as written.
