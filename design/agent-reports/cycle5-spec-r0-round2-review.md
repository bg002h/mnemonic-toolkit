# cycle-5 "S-NET" network-provenance invariant — SPEC R0 review, ROUND 2

**Artifact under review:** `design/BRAINSTORM_cycle5_snet_network_invariant.md` (folded after round 1)
**Reviewer:** opus software architect (adversarial R0, fold-verification)
**origin/master SHA:** `ac4eead0` (toolkit `0.62.1`)
**Date:** 2026-06-21
**Gate:** mandatory pre-implementation R0 — NO code until 0 Critical / 0 Important.
**Prior round:** R0 round 1 = 0C / 3I (RED); findings I1, I2, I3 + minors M-1…M-6.

All fold claims were re-verified against `git show origin/master:<path>` at review time — the spec's
self-assessment was treated as a hypothesis, not evidence. The round-1 architecture confirmations
(dead `NetworkMismatch` variant, 2-way NetworkKind maximality, `CosignerSpec` precedent, helper home,
oracle-gate soundness) were not re-litigated; only the folds and possible fold-introduced drift were
adversarially re-examined.

---

## Fold verification (against live source)

### I1 (H9 variant/exit) — RESOLVED
**Round-1 finding:** H9's variant/exit was unspecified and §7 implied exit 2 / `NetworkMismatch`,
conflicting with the adjacent live exit-1 `ImportWalletNetworkClassMismatch` sibling.

**Verified against live source:**
- `import_wallet.rs:1191-1209` (the `--network` override block): the guard reads `parsed.first()`
  (`:1192`), computes `parsed_coin_type`, and on mismatch returns
  `ToolkitError::ImportWalletNetworkClassMismatch { requested, parsed_coin_type }` (`:1199`); the
  rebind loops `parsed.iter_mut()` rebinding every entry's `network`. The spec's H9 fix — *extend the
  `first()`-only class-check to ALL entries, reusing `ImportWalletNetworkClassMismatch`* — is the
  correct, minimal, per-entry generalization of exactly this code. CONFIRMED.
- `error.rs:576` `ImportWalletNetworkClassMismatch { .. } => 1` (exit 1) and `error.rs:587`
  `NetworkMismatch { .. } => 2` (exit 2). CONFIRMED verbatim.
- The folded spec now pins H9 → `ImportWalletNetworkClassMismatch` / **exit 1** at every site: §1
  table (line 45), §2.3.1 (the new two-axis section, lines 135-142), §3 row H9 (line 161), §7 row H9
  (line 248), and decision rows 9/9a (lines 291-292). The H9 RED test (§7) asserts
  `exit 1, kind=ImportWalletNetworkClassMismatch`.
- **No remaining `H9 → NetworkMismatch` or `H9 → exit 2` claim survives.** `grep -nE 'H9'` filtered for
  `NetworkMismatch|exit 2` returns ONLY the deliberate axis-contrast prose (§2.3.1 line 142, §3 line
  161, §7 line 243, decision rows 9/9a) where the spec explicitly states H9 does NOT use
  `NetworkMismatch`/exit 2 and explains why. No contradictory positive assertion remains.
- **Two-axis documentation is present and correct (§2.3.1):** axis-1 = `--network`-vs-coin-type-class
  → `ImportWalletNetworkClassMismatch` / exit 1 (H9); axis-2 = xpub-version-vs-coin-type →
  `NetworkMismatch` / exit 2 (H15/M13/M14/L2/L10/L11). The two-condition distinction is real and
  correctly characterized: a blob can pass the per-entry `--network` class check yet still carry an
  xpub whose version bytes contradict its own coin-type path (the H15 hand-edited case). The coexistence
  of two exit codes in the import flow is intentional, documented, and architecturally justified.

**Disposition: RESOLVED.** The variant/exit is now pinned and consistent with the live sibling, the
two-axis split is sound, and the §7 RED test asserts the correct exit.

### I2 (positive control) — RESOLVED
**Round-1 finding:** the H9 positive control `[Bitcoin] + --network signet → exit 0` was factually
wrong (a coin-type-0 blob + signet is a cross-class override the live code REFUSES at exit 1).

**Verified against live source:**
- `cli_import_wallet_network_override.rs:76` `mainnet_blob_override_to_signet_refused` — a mainnet
  (`core-bip84-mainnet.json`, coin-type-0) blob + `--network signet` is REFUSED. The old bogus control
  would therefore have FAILED. CONFIRMED the premise of the round-1 finding.
- `grep '\[Bitcoin\].*signet|signet.*exit 0'` against the folded spec → EMPTY. The bogus control is GONE.
- The folded H9 row (§7 line 248) now uses RED = mixed `[Bitcoin, Testnet] + --network bitcoin`
  (first=Bitcoin passes the old `first()` check; the Testnet entry is caught per-entry → exit 1), and
  positive control = same-class homogeneous `[Bitcoin, Bitcoin] + --network bitcoin → exit 0, both
  network:"mainnet"`.
- **The new control is genuinely same-class.** Bitcoin = coin-type-0 (`override.coin_type()==0` for
  `--network bitcoin`); a homogeneous `[Bitcoin, Bitcoin]` blob is uniformly coin-type-0; the per-entry
  extension finds NO disagreeing entry → not refused → exit 0. This is the exact live analogue of
  `mainnet_blob_override_to_mainnet_noop_ok` (`:82-85`: `core-bip84-mainnet.json` + `--network mainnet`
  → exit 0, `network:"mainnet"`), which passes today. CONFIRMED the control will pass.

**Disposition: RESOLVED.** The wrong control is removed; the replacement is genuinely same-class and
matches a live passing test.

### I3 (zero-false-reject + originless) — RESOLVED
**Round-1 finding:** "zero false-reject" was asserted not proven (no full-suite sweep committed), and
the originless / no-coin-type input case was unhandled (over-reject risk for a legitimate input).

**Verified against live source:**
- `descriptor.rs:199` `coin_type_from_path`: `if comps.len() < 2 { return Err(ImportWalletParse(...)) }`
  — an originless / sub-2-component-origin key genuinely has NO derivable coin-type to assert against.
  CONFIRMED the no-op precondition's factual grounding.
- `cli_descriptor_concrete.rs:174` — `wpkh(tpubD…/0/*)` originless concrete descriptor in the test
  `export_wallet_originless_concrete_still_accepted` ("origin-less concrete must NOT be rejected"). The
  spec's cited fixture/shape is real and is itself a regression guard for the exact accept the no-op
  precondition must preserve. CONFIRMED.

**(a) No-op precondition is present at MULTIPLE sites (not just once):** §2.2 helper-comment (line 75),
the full precondition paragraph (line 98), the insertion-strategy paragraph (line 100, "ONLY when a
coin-type network was actually derivable"), §3 hard-REJECT justification (line 170), §7 originless
positive control (line 263), and decision row 9b (line 293). The precondition is stated WHEREVER the
helper / cross-check is described — it is not a single buried clause. The scoping is correct: it
applies to the import parsers + M13; M14/L11/build-L1 "always have an asserted side" — verified against
`convert.rs:922-924` (the `--xpub-prefix` non-default path requires `--network`, so an asserted side is
always present) and the WIF/build paths which carry `pk.network` / `--network`.

**(b) FULL-package sweep committed as the proof:** §3 (line 170, "proven zero by a committed FULL-suite
sweep — a green `cargo test -p mnemonic-toolkit` (the WHOLE package) … NOT a targeted-test claim"),
§7 (line 261, dedicated "Committed FULL-suite sweep" paragraph), and decision row 9c (line 294). It
cites the project full-package-R0 discipline (MEMORY `feedback_r0_review_run_full_package_suite`)
correctly — the exact lesson (targeted runs miss stale/consistent fixtures elsewhere in the package) is
the one that bit P6.1. CONFIRMED.

**Disposition: RESOLVED.** Both sub-gaps are closed: the no-op precondition is specified and grounded
in real source, a positive control proves the originless accept survives, and the full-package green is
committed as the no-over-rejection proof.

### Minors — all RESOLVED
- **M-1 (slip0132 path):** `git ls-files | grep slip0132` → `crates/mnemonic-toolkit/src/slip0132.rs`
  (crate root); NO `wallet_import/slip0132.rs` exists; `grep wallet_import/slip0132` against the spec →
  EMPTY. The source-SHA table (line 30) and §3 rows 9/10 now cite `src/slip0132.rs`. `apply_xpub_prefix`
  at `:108`, `swap_target_for` at `:197` — line numbers correct. RESOLVED.
- **M-2 (L3 legacy branch):** `coldcard.rs:238` `raw_account = … .map(|n| n as u32)`; the truncation is
  interpolated ONLY in the legacy top-level-xpub fallback (`deriv_path_str_opt == None` arm,
  `format!("m/{purpose}'/{coin_type}'/{raw_account}'")` at `:268`); the per-bipN path uses
  `Some(s)` and never touches `raw_account`. The folded §5.1 + §7 L3 row now require a *legacy
  top-level-xpub* fixture (cf. `coldcard-mk1-legacy-bip84-mainnet.json`) and explicitly flag a per-bipN
  RED test as VACUOUS. RESOLVED. (Exit/kind `ImportWalletParse`/exit 2 + the firewall-from-helper
  decision #10 carried forward.)
- **M-3 (detail_json wire delta):** §6.2 (line 223) now explicitly acknowledges the `detail_json`
  `--json` error-shape delta ("not true that there is no `--json` wire change at all"), correctly states
  it is NOT a `schema_mirror`/manual trigger (those gate clap flag-NAMES + dropdown VALUES, not
  error-envelope shape per CLAUDE.md), and notes the dead variant ⇒ zero observable blast radius.
  Decision row 14 mirrors this honestly. RESOLVED.
- **M-4 (field-type ratification):** §2.3 + decision row 4 record the R0-ratified decision (keep
  `&'static str` + take the rename `xpub_network→decoded_network`, `expected→expected_network`, add
  `context`); the open lean is closed. The three arm edits (Display `:830`, `detail_json` `:913`, unit
  test `:1013`) are the complete set — re-confirmed `git grep` shows only the def + the 5 match arms.
  RESOLVED.
- **M-5 (DescriptorPublicKey arms):** §4 step 1 now enumerates the `XPub`/`MultiXPub` (`.xkey.network`)
  arms and the `Single` skip + all-`Single` "unknown → keep default, no warning" fallback. RESOLVED.
- **M-6 (Display wording no-op):** §2.3 carries the no-op note; the `:1013` unit test asserts only
  `exit_code()==2` (re-confirmed in source: the test body constructs the variant and asserts the exit
  code, not the Display string). RESOLVED.

---

## Fold-introduced-drift check (adversarial)

- **No stray contradiction.** `grep H9 … NetworkMismatch|exit 2` returns only deliberate axis-contrast
  prose that says H9 does NOT use exit 2. `grep wallet_import/slip0132` → empty. `grep [Bitcoin]…signet`
  → empty. No now-dangling cross-reference.
- **Resolved-decisions table is internally consistent with the folded body.** Rows 9 (H9 →
  `ImportWalletNetworkClassMismatch` / exit 1), 9a (two coexisting exit codes), 9b (originless no-op),
  9c (full-suite sweep) all match §2.3.1 / §2.2 / §3 / §7 verbatim. Row 4 (field rename ratified), row
  14 (lockstep NONE + honest `detail_json` caveat), row 15 (oracle add-only) are consistent.
- **No NEW open question introduced.** §9 remains "no open questions — leans recorded"; the two R0
  ratification points (§2.3 field type, §4 L1 WARN-vs-REJECT + §5.1 L3 fold) are decided, not left open.
  The spec is still decision-complete.
- **Core architecture unchanged and still sound.** 2-way NetworkKind granularity (provably maximal —
  one testnet xpub version byte covers testnet/signet/regtest), dead-`NetworkMismatch` wiring (zero
  construction sites confirmed: `git grep` shows def + 5 arms only), the 11 fix sites (line numbers
  ±3, all located in round 1), SemVer MINOR 0.63.0 (first-to-ship vs the paused own-account cycle),
  zero clap/schema_mirror/manual lockstep, and the oracle add-only-rejection argument — none was
  touched by the folds; all remain sound. The I1 fold (H9 → exit 1) actually IMPROVES architectural
  coherence: it keeps both `--network`-axis refusals (the existing single-entry one and the new
  per-entry one) on the same variant/exit, and isolates the dead-variant wiring to the genuinely
  distinct xpub-version axis.

---

## Critical

**(none)**

## Important

**(none)** — all three round-1 Importants (I1, I2, I3) are verified RESOLVED against live source; the
folds introduced no new Important and no contradiction.

## Minor

**(none new)** — all six round-1 Minors are verified RESOLVED. No new Minor surfaced.

---

## Verdict

Criticals = 0; Importants = 0; Minors = 0.

**R0 ROUND 2: 0C / 0I — GREEN.**

I1, I2, I3 are each RESOLVED (verified against `import_wallet.rs:1191-1209`, `error.rs:576/587`,
`cli_import_wallet_network_override.rs:76/82`, `descriptor.rs:199`, `cli_descriptor_concrete.rs:174`,
`coldcard.rs:238/268`, `slip0132.rs:108`). All six Minors RESOLVED. The folds introduced no new
Critical/Important and no internal contradiction; the spec is decision-complete and the core
architecture is unchanged and sound. The brainstorm spec PASSES the R0 gate and may proceed to the
plan-doc (which runs its own independent R0 loop per CLAUDE.md).
