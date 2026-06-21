# R0 REVIEW — cycle-10 md-codec cluster (M3 + L14/L15/L17 + L6) — PLAN-DOC, Round 1

**Plan-doc:** `design/IMPLEMENTATION_PLAN_cycle10_mdcodec_lib.md`
**Source-of-truth verified against:** `descriptor-mnemonic origin/main = 1a4b322`. Toolkit citations verified against toolkit `origin/master`.

## VERDICT: **RED — 0 Critical / 2 Important**

The funds-critical M3 fix and the publish->pin chain are correct and provably fail-closed. Two Important findings are stale/non-existent-target citations in the bookkeeping sections (§6/§7) that would mis-execute mechanically. Both are mechanical fixes; no design change needed.

## What is VERIFIED-CORRECT (the priority axes)

**M3 funds-safety — fail-closed, fully traced (highest scrutiny):**
- Gate confirmed at `derive.rs:110/117/124`; replacement block `110-122` is the correct target and does not disturb the hardened pre-flight at `105-107`.
- The widening (`max_alts` = MAX over baseline + every override, `None->1`) is correct. Per-key authority `use_site_to_derivation_path` (`to_miniscript.rs:277-292`) re-resolves `chain` against EACH key's own multipath and STILL fail-closes at `:282`. Widening can only let a request reach a per-key path that then derives correctly or rejects — **never a wrong subtree.** Adversarially confirmed: a `None`-multipath key derives a bare (chain-component-free) path, which is structurally correct (a non-multipath key has no receive/change branch) — not a wrong-address. D5(b) legality confirmed at `validate.rs:112-148`.
- Regression `derive_address_chain_out_of_range` (`derive.rs:241`, `chain:5 alt_count:2`, `standard_multipath()` alt-count=2, no overrides -> `max_alts==2`) stays GREEN. Positive control (over-max -> still rejects) is sound.
- Type-check: `UseSitePath { multipath: Option<Vec<Alternative>>, wildcard_hardened: bool }`; `Alternative { hardened: bool, value: u32 }`; `chain: u32` — all confirmed; the plan's `wildcard_hardened: false` RED-test field and `(chain as usize)` cast are correct.

**L14/L15/L17:** `identity.rs:4` import, `:71/:74/:83` (WDT-id), `:172/:176/:189/:193` (policy-id + L14 locus), `:572` vacuous test (both operands explicit `Shared(BIP84)` — genuinely vacuous), `:593` sibling (non-vacuous, keep), fixture `:385`, `canonical_origin.rs:45` (`wpkh->84'/0'/0'` matches the fixture) — ALL confirmed. SemVer in-memory-only determinant holds (`encode.rs` has zero id references). `golden_vector_wpkh_cell_7` `:469`, `compute_wallet_policy_id_canonicalizes_first` `:791`, `pkk` `:627`, `standard_multipath()` `:58` — all confirmed.

**L6:** `canonicalize.rs:168/169/200/206/216` panic site and `:425-432` sibling guard confirmed. `error.rs:66` `DivergentPathCountMismatch { n: u8, got: usize }` exists — no new variant. The `let n_keys = d.n;` borrow-resolution is necessary and correct.

**Publish->pin chain (P4):** Tag conventions VERIFIED against live tags — md-codec uses `md-codec-v0.38.0` (no `descriptor-mnemonic-` prefix); md-cli uses `descriptor-mnemonic-md-cli-v0.9.0`. Neither `md-codec-v0.39.0` nor `descriptor-mnemonic-md-cli-v0.9.1` exists. Order (md-codec publish -> md-cli pin-edit `Cargo.toml:28` `=0.38.0`->`=0.39.0` -> md-cli publish) correct. `fuzz/Cargo.lock:169` stale at `0.35.1` confirmed; `fuzz/Cargo.toml:22-23` path-only; root `Cargo.lock:500` at 0.38.0; `ci.yml:59 cargo fmt --all --check` fmt gate confirmed (contrast toolkit-exempt — correct). `to_miniscript_descriptor` call line is **124** (plan correct; spec had 123 off-by-one).

**Toolkit version-collision handling (§6):** Correctly does NOT hard-code 0.65.1 vs 0.65.2; instructs reconciling against the live tag at pin-bump time. Latest toolkit tag = v0.65.0 confirmed.

## IMPORTANT findings (block GREEN — mechanical fixes)

**[Important-1] Toolkit pin FROM-version citation WRONG: `"0.37"` -> should be `"0.38"`.** Plan §2 + §6 state `Cargo.toml:36 (md-codec = "0.37")` / `Cargo.lock:677 (0.37.0)`. Actual at toolkit origin/master: `Cargo.toml:36 = md-codec = "0.38"`; `Cargo.lock:677 = 0.38.0` (toolkit already consumes 0.38.0 from cycle-4). The transitive bump is `"0.38" -> "0.39"`. (Inherited from spec §1/§4.) Line numbers `:36`/`:677` correct.

**[Important-2] Bughunt-report tick line numbers in §7 (and §2 row) all STALE — every one points at prose, not a checkbox.** Plan claims M3@237/L6@353/L14@572/L15@582/L17@600. Actual `### - [ ]` checkbox headers in `constellation-bughunt-2026-06-20.md`: M3 **@245**, L6 **@368**, L14 **@593**, L15 **@603**, L17 **@621**. The slug text matches — only the line numbers drifted (citation decay). An implementer flipping the literal numbers would tick the wrong/no checkboxes.

## MINOR (fold opportunistically; non-blocking)

- **[Minor-1]** §7 FOLLOWUP-status-flip targets do not exist — none of the five slugs are in `FOLLOWUPS.md` in EITHER repo. The bughunt report is their system-of-record. Either FILE as RESOLVED-in-cycle or rephrase §7 to rely solely on the bughunt-report ticks.
- **[Minor-2]** Clippy invocation differs from CI: plan uses `cargo clippy --all-targets`; CI (`ci.yml:47`) uses `cargo clippy --workspace --all-targets`. Add `--workspace`.
- **[Minor-3]** L14 edit-boundary: the plan's snippet re-declares `let mut path_scratch = BitWriter::new();` but `identity.rs:192` already declares it — the implementer must REPLACE lines 192-193, not insert before them, or get a duplicate binding.
- **[Minor-4]** Plan §2 cites `to_miniscript_descriptor` line 124 (correct); spec §1 had 123 (off-by-one). Plan is right; no action.

## Required to reach GREEN

Fold Important-1 (pin 0.37->0.38) and Important-2 (tick lines -> 245/368/593/603/621), persist this review, re-dispatch. Both are pure citation-corrections — no design/phasing/M3/publish-chain change. The funds-critical M3 fix is provably fail-closed; the in-memory-only SemVer determinant is verified; L6/L17 mechanical and accurate; publish->pin order and tag conventions correct against live tags.
