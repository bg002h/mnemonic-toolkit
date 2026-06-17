# C2 impl-review round 2 — confirmation of C1/I1/M1 folds (verbatim) + convergence

> Reviewer: opus code reviewer (general-purpose). Confirms the round-1 impl-review folds on
> branch `feature/verify-bundle-bip388-intake`. Verdict RED (0C/1I); the lone Important was a
> plan-doc self-contradiction, folded immediately after (see footer) → converged 0C/0I.

---

[Cross-check summary — all six version sites at 0.57.0, mutually consistent:]
- `crates/mnemonic-toolkit/Cargo.toml:3` → `0.57.0` ✓
- root `README.md:13` → `<!-- toolkit-version: 0.57.0 -->` ✓
- `crates/mnemonic-toolkit/README.md:9` → `<!-- toolkit-version: 0.57.0 -->` ✓
- `scripts/install.sh:32` → `mnemonic-toolkit-v0.57.0` ✓
- main `Cargo.lock:727` → `version = "0.57.0"` ✓
- `fuzz/Cargo.lock:575` → `version = "0.57.0"` ✓

**Verdict: RED — 0 Critical, 1 Important**

**I1 — Plan-doc M1 fold is internally self-contradicting; the stale "only one README marker" premise was NOT fully removed.** `design/PLAN_C2_verify_bundle_bip388_intake_2026-06-16.md:150-151` still reads: *"(The 'README marker ×2' recollection is from a prior cycle — today there is only one; do not hunt for a second.)"* — which directly contradicts the corrected text at lines 138-144 ("there are **TWO** guard-enforced markers — `README.md:13` AND `crates/mnemonic-toolkit/README.md:9`"). This is the exact false premise (M1) that caused round-1's C1 suite-RED. The M1 fold patched the top of the section but left the contradicting parenthetical at the bottom intact, so the next reader/cycle's "version-marker lockstep" checklist is still actively mis-instructed to stop at one marker. Fix: delete or invert lines 150-151 to match the corrected "TWO markers" text.

Everything else is GREEN, confirmed on disk:
- **Item 1 ✓** `crates/mnemonic-toolkit/README.md:9` = `<!-- toolkit-version: 0.57.0 -->`; `cargo test -p mnemonic-toolkit --test readme_version_current` → `both_readmes_carry_current_version_marker ... ok` (1 passed).
- **Item 2 ✓** `fuzz/Cargo.lock:575` `mnemonic-toolkit` package = `version = "0.57.0"`.
- **Item 3 (partial)** The section now correctly lists TWO README markers + the fuzz lockfile + install.sh self-pin — but the trailing contradiction is the I1 above.
- **Item 4 ✓** Straggler sweep clean. All six sites mutually consistent at 0.57.0. The remaining `0.56.0` hits are all expected-historical (CHANGELOG `[0.56.0]` entry, "removed v0.56.0" prose, `v0.56.0+` floor in recon files, prior plan-docs/continuity, and the now-corrected plan-doc "before" reference). `docs/manual-gui/pinned-upstream.toml:22` carries `toolkit-tag-implied = mnemonic-toolkit-v0.13.0` — a GUI-manual artifact that lags by design (not a toolkit-release-ritual bump target), so NOT a missed site.
- **Item 5 ✓** Core feature code unchanged from round 1: the verify_bundle.rs insertion is the exact `is_bip388_policy_shape`→`expand_bip388_policy(&…)?` block (mirror of bundle.rs:319); the 3 test cells (2a sortedmulti round-trip, 2b wpkh n=1, 2c `@N`-beyond refused-by-message) are unchanged. The fold touched only the 4 lockstep files — no code drift.

Note: one Important blocks the gate per the 0C/0I rule. Recommend the implementer fix the plan-doc lines 150-151 contradiction, then re-confirm. The on-disk release-ritual state itself is fully correct and the suite is GREEN — this is purely a plan-doc-faithfulness gap in the M1 fold.

---

## FOLD + CONVERGENCE (post-round-2, by implementer)

- **I1 fixed:** `PLAN_C2_…:150-151` rewritten to remove the contradiction — the trailing parenthetical now reads "Impl-review C1 CORRECTED the earlier R0-r1 M7 claim of 'only one README marker': there ARE two, both guard-enforced … The '×2' recollection from prior cycles was RIGHT." Lines 138-144 (TWO markers) and 150-151 are now consistent.
- **Empirical confirmation:** full `cargo test -p mnemonic-toolkit` at v0.57.0 → **exit 0 (GREEN)** (background run `bz1xge2ch`), which includes `readme_version_current` — so C1 is fixed suite-wide. Fuzz build `RUSTFLAGS="--cfg fuzzing" cargo build` → exit 0.
- **The round-2 Important was a documentation-faithfulness gap only** (the reviewer itself stated "the on-disk release-ritual state itself is fully correct and the suite is GREEN"). With the single contradicting sentence resolved, **C2 converges to 0 Critical / 0 Important.** Ship.
