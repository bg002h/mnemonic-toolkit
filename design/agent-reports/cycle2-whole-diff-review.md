# cycle-2 (H8 / H10 / H7) — WHOLE-DIFF adversarial execution review (final ship gate)

**Reviewer:** opus adversarial whole-diff review (mandatory, non-deferrable; final gate before
version-bump / merge-to-master / tag).
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/cycle2-integration`,
branch `cycle2-integration`.
**Baseline:** `origin/master` = `f9467cc5` (release 0.61.0, cycle-1 H12/H1/H13 merged).
**Merged HEAD:** `6bfa83e4` — H8 (`d824d2fe`) + H10 (`f90b252b`) + H7 (`6bfa83e4`), 0 conflicts.
**Diff under review:** `git -C <wt> diff origin/master...HEAD` (9 files, +1399/−41).
**Context:** spec R0 ×3, plan R0 ×2, and the three per-phase impl reviews
(`cycle2-h8/h10/h7-impl-review-round1.md`), all GREEN.

---

## VERDICT: GREEN — 0 Critical / 0 Important. **CLEAR TO SHIP.**

The three HIGH funds-loss fixes compile and pass TOGETHER (3297 passed / 0 failed / 15 ignored
across 185 binaries; clippy clean). The combined change is file-disjoint at the commit level; the
one cross-file coupling (H8 changed `synthesize_template_descriptor`'s signature; H7 edited
`bundle.rs`) is NON-interfering — H8's signature change is entirely internal to `synthesize.rs`
(public `synthesize_descriptor` arity unchanged), and `bundle.rs` calls only that unchanged
public entry. All three funds-safety properties hold empirically in the merged binary, and the
cycle-1 H13 hardened-multipath reject STILL fires in the merged tree for BOTH the bare and the
new prefix-annotated forms. No version-site / schema / manual / fmt / mlock churn; the new error
variant is correctly alphabetical; no clap flag added (Q-WIRE clean). Ship.

---

## Critical

NONE.

## Important

NONE.

---

## Cross-phase interaction

**Commit-level file disjointness (verified `git diff-tree --name-only` per commit):**

| phase | source files | test files |
|---|---|---|
| H8 (`53787cbb`) | `synthesize.rs` | (in-module) |
| H10 (`29b39723`) | `cmd/export_wallet.rs`, `error.rs` | `cli_export_wallet_unsorted_multi_refusal.rs`, `cli_wallet_cross_format_convergence.rs` |
| H7 (`36095b88`) | `cmd/bundle.rs`, `parse_descriptor.rs` | `cli_bundle_h7_prefix_origin.rs`, `cli_mode_violations_v0_5.rs` |

**No source file is touched by two phases.** The merge is a clean union; there is no same-file
three-way text conflict and none was reported (0 conflicts).

**The one cross-file coupling — H8-signature vs H7-`bundle.rs` — verdict: NON-INTERFERING.**
- H8 added a 4th param (`run_language: bip39::Language`) to the PRIVATE
  `fn synthesize_template_descriptor` (`synthesize.rs:1163`). Every caller of that fn is WITHIN
  `synthesize.rs`: the SOLE non-test caller is `synthesize_descriptor`'s `is_template()` dispatch
  at `:487-493` (now forwards `run_language`); the remaining 8 call sites are all `#[cfg(test)]`
  and were updated in lockstep. `grep -rn synthesize_template_descriptor src/` confirms ZERO
  call sites outside `synthesize.rs`.
- The PUBLIC entry `synthesize_descriptor` (`synthesize.rs:467`) signature is UNCHANGED (still
  `(descriptor, cosigners, privacy_preserving, run_language, md1_form)` — 5 params, identical to
  master). `bundle.rs` calls ONLY `synthesize_descriptor` (3 runtime sites: `:1810`, `:1927`,
  `:2171`), never the private template fn. **No arity break; the merged `bundle.rs` compiles
  against `synthesize.rs` — empirically confirmed by a clean `cargo build` + 3297-test pass.**
- H7's `bundle.rs` edit is confined to the xpub-slot fp cross-check (`:1650-1672`): it adds a
  prefix/suffix-annotation-vs-`--slot @N.fingerprint=` mismatch refusal. It does NOT touch any
  `synthesize_*` call site. The data it consumes (`anno_fp` ← `resolved_placeholders.fingerprint_annos`
  ← `lex_placeholders`) flows from H7's own `parse_descriptor.rs` change — a self-contained
  producer/consumer pair, no overlap with H8 or H10.

**No shared symbol touched by two phases with semantic drift.** H10 is the ONLY phase editing
`error.rs` (adds one variant, reuses nothing H7/H8 touch); H7 reuses the pre-existing
`DescriptorParse` (no `error.rs` edit), so there is no error-variant collision. H8 adds no
variant. The three are genuinely disjoint: H7 (`parse_descriptor.rs` lexer / `bundle.rs`
cross-check) vs H10 (`export_wallet.rs` emit chokepoint / `error.rs`) vs H8 (`synthesize.rs` ms1
emit) share no function, type, or control-flow path. No behavioral coupling.

---

## Merged build/test result

- `cargo build -p mnemonic-toolkit --bin mnemonic` → clean (24s, 0 errors).
- `cargo test -p mnemonic-toolkit` → **exit 0; 3297 passed / 0 failed / 15 ignored across 185
  test binaries** (independently confirmed by a second background run, also exit 0).
- `cargo clippy -p mnemonic-toolkit --tests` → clean (0 warnings, 0 errors).
- Key merged-tree tests all green: `template_singlesig_non_english_run_language_emits_mnem`,
  `template_keyed_ms1_parity_across_languages` (H8); `unsorted_multi_refused_typed_exit2_for_fieldless_vendors`
  (H10); `lex_prefix_origin_parity_with_suffix`, `lex_prefix_hardened_multipath_still_rejects_h13`
  (H7); plus the two lockstep-updated pre-existing tests `c4_unsorted_multi_order_preservation`
  and `descriptor_without_template_accepted`.

---

## Holistic funds-safety (empirical, merged binary `target/debug/mnemonic`)

**H8 — non-English template → `Payload::Mnem`.** Tests pin Spanish (wire 3) emits `Mnem`, not
the silent-English `Entr`; English still emits `Entr` (regression guard); the master-fingerprint
DIVERGENCE test computes both fps in-test (oracle doc-only) and proves the card reconstructs the
SPANISH seed; keyed↔template ms1 byte-parity across both languages. All green in the merged tree.

**H10 — empirical refuse/allow matrix (merged binary):**
- unsorted `wsh-multi` → `coldcard-multisig`: **exit 2**, typed message ("UNSORTED multisig …
  BIP-67 sortedmulti-only …"). ✓
- SORTED `wsh-sortedmulti` → `coldcard-multisig`: **exit 0** (still exports). ✓
- single-sig `bip84` → `coldcard`: **exit 0** (still exports). ✓
- unsorted `wsh-multi` → `descriptor` (faithful): **exit 0**, emits literal `multi(`. ✓
- New variant exit_code = 2; message names the offending format AND points to faithful formats
  (anti-dead-end). The guard is a structured check on the typed `CliTemplate` (immune to the
  `sortedmulti(`-as-substring false-match), placed at the shared `emit_payload` chokepoint, so it
  also covers restore for free without editing `restore.rs`.

**H7 — empirical (merged binary):**
- prefix `[deadbeef/84'/0'/0']@0` accepted, origin carried: **exit 0**. ✓
- prefix-form stdout **byte-identical** to the suffix-form (`diff -q` IDENTICAL). ✓
- WRONG `--slot @0.fingerprint=cafef00d` vs prefix `deadbeef`: **exit 2**
  ("--slot @0.fingerprint specifies cafef00d but descriptor @0 annotation specifies deadbeef"). ✓
- both-positions `[…]@0[…]`: **exit 2** (typed ambiguous-double-origin refusal). ✓

No residual gap, no new bug surfaced by the combination.

---

## H13-still-fires-in-merged-tree (empirical confirmation)

Built the merged binary and fed hardened-multipath descriptors through the real
`mnemonic bundle … --slot @0.xpub=<valid> --network mainnet` path:

| input | exit | message |
|---|---|---|
| **H13-A bare** `wpkh(@0/<0';1'>/*)` | **2** | `@0 multipath alternative `0'` is hardened; … un-restorable` |
| **H13-B prefix** `wpkh([deadbeef/84'/0'/0']@0/<0';1'>/*)` | **2** | `@0 multipath alternative `0'` is hardened; … un-restorable` |

**The cycle-1 H13 reject FIRES in the merged tree for BOTH the bare AND the new prefix-annotated
form.** H7's named-group regex conversion did NOT detach the H13 multipath validator from the
`mpath` body — the prefix alternation can sit before `@N` and the by-NAME `caps.name("mpath")`
access still reads the multipath body unshifted. Confirmed both at the binary level (table above)
and by the in-tree test `lex_prefix_hardened_multipath_still_rejects_h13` (green).

---

## Ship-readiness / deferred-minors

**Scope hygiene (all confirmed):**
- Exactly 9 files changed, all under `crates/mnemonic-toolkit/` (5 src + 4 tests).
- NO Cargo.toml / Cargo.lock / mlock / schema / README / install.sh / docs/manual / fuzz churn
  (grep-empty). Version-site edits correctly NOT yet made (next step).
- NO clap flag/arg/subcommand/dropdown-value added → NO GUI schema-mirror leg, NO manual
  flag-table leg required. **Q-WIRE clean.**
- New error variant `ExportWalletUnsortedMultisigUnsupported { format: &'static str }` is
  **alphabetical**: sits between `ExportWalletTaprootMultisigUnsupported` (T<U) and `FutureFormat`
  (Export…<Future) in the enum AND in all three exhaustive `match self` arms (`exit_code`/`kind`/
  `message`). Confirmed.
- NO `cargo fmt --all` / mlock churn: the implementers did NOT run `cargo fmt` — the standing
  project-wide manual-format deviations (lib.rs mod order, mlock.rs — the documented g6/mlock
  fmt exemptions) exist identically on master and were NOT "fixed" in the diff; the diff is
  hunk-targeted to the H8/H10/H7 logic + tests only.
- The 3 pre-existing tests updated are correct lockstep updates, NOT made-to-pass hacks:
  - `c4_unsorted_multi_order_preservation` — was a "by-design sortedmulti-coercion" probe; now
    asserts the H10 refusal (exit≠0, stderr `"UNSORTED multisig"` + `"sortedmulti-only"` — both
    strings VERIFIED present in the H10 message). The faithful-format convergence anchor
    (bitcoin-core/bsms/sparrow/specter → "Multi" tag, declaration-order preserved) is PRESERVED.
  - `descriptor_without_template_accepted` — swaps the annotated `[deadbeef/…]@0` descriptor for
    the bare `@0` form; the annotated form now (correctly) fires H7's per-@N fp cross-check vs
    TREZOR_24's real master fp. The rewrite preserves the test's MODE-acceptance intent.

**Deferred minors (carried from per-phase reviews — NONE block ship):**
- **H8 m-1 (cosmetic):** a comment cites the keyed twin as `:547`; the live assignment is `:552`
  (off-by-a-few on a doc comment; the byte-identity claim itself is TRUE). DEFER (no FOLLOWUP
  needed — purely cosmetic). Does NOT block ship.
- **H8 m-2 (observation):** `synthesize_full` (`#[allow(dead_code)]`, test-only helper) still
  emits hardcoded-English `Entr`; it is NOT a template path and NOT reachable from
  `--md1-form=template`. Out of scope; DEFER. Does NOT block ship.
- **H10 minors (informational):** integration `FIELDLESS` omits the generic `coldcard` single-sig
  alias (covered by the unit test's `Coldcard` arm); doc-comment format-name list is accurate.
  No action. Does NOT block ship.
- **H7 M1 (cosmetic):** path-only prefix vs suffix forms reject with DIFFERENT downstream message
  text (both loud exit-2 `DescriptorParse`, funds-safe). DEFER. Does NOT block ship.
- **H7 M2 (pre-existing):** standalone `rustfmt --check` reports 4+1 deviations in
  `parse_descriptor.rs`/`bundle.rs` that ALSO exist on master (manual-format territory). H7
  introduces ZERO new fmt deviations. Pre-existing; no action. Does NOT block ship.

**No minor rises to Important; no minor blocks ship. All are pure cosmetics or pre-existing
out-of-scope observations.**

---

## Bottom line

GREEN — 0 Critical / 0 Important. The merged cycle-2 change builds and passes as a unit
(3297/0/15, clippy clean), the H8-signature×H7-bundle.rs coupling is non-interfering, all three
funds-safety properties hold empirically in the merged binary, and cycle-1 H13 still fires for
bare AND prefix forms in the merged tree. **CLEAR TO SHIP** (proceed to version-bump → merge →
tag; the standard release version-site checklist — both READMEs + fuzz/Cargo.lock self-pins — is
the next step and is OUT of this diff's scope by design).

_Review only — no source edited; the checkout-baseline comparison was restored to HEAD (worktree
clean). Persisted before any version-bump/merge/tag step._
