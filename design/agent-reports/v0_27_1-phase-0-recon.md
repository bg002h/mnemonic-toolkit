# v0.27.1 Phase 0 Reconnaissance

**Date:** 2026-05-19
**Branch:** `v0_27_1/phase-0-recon` (off `release/v0.27.0` tip pre-merge; post-merge master is `77ebfca`)
**Author:** code-explorer agent (opus)
**Plan-doc:** `design/PLAN_v0_27_1_pr_26_fold_cycle.md` (R2 GREEN at recon time; R3 fold pending per §6)
**Verdict:** YELLOW — one important plan-doc line-range correction (Phase 1 stale cites for C1 + I7); all other preconditions GREEN.

---

## §1. Findings-vs-Current-Source Drift Table

All citations verified against `release/v0.27.0` tip. v0.27.0 added substantial material to `cmd/import_wallet.rs`, shifting many line numbers significantly upward from the v0.26.0 baseline where the original agent-report was captured.

| Finding | Plan-doc cite | Current source cite | Status |
|---|---|---|---|
| C1 (roundtrip suppression — two `Err(_)=>Ok(())` arms) | `cmd/import_wallet.rs:471-478` | `cmd/import_wallet.rs:710-736` (function `emit_roundtrip_stderr_warning`); the two Err arms are at **lines 720 + 724** | **DRIFTED** — function body is ~286 lines later; both Err arms confirmed present |
| C2 (`env_sentinel.rs` row 6 mis-doc) | `env_sentinel.rs:1-13` | `env_sentinel.rs:10` (`--slot @N.ms1=`) | MATCH — `SlotSubkey` has no `Ms1` variant per plan-doc |
| I4 (`active`/`internal` silent bool default) | `wallet_import/bitcoin_core.rs:273-280` | same | MATCH |
| I5 (fingerprint fallback silent substitution) | `wallet_import/json_envelope.rs:258-260` | same; `let _ = slot_idx; // reserved` confirmed at **line 269** | MATCH |
| I6 (`extract_threshold` def — bsms) | `wallet_import/bsms.rs:414` (plan-doc Q2 corrected) | `bsms.rs:414` | MATCH |
| I6 (`extract_threshold` def — bitcoin_core) | `wallet_import/bitcoin_core.rs:456` (plan-doc Q2 corrected) | `bitcoin_core.rs:456` | MATCH |
| I6 (call site — bsms) | `wallet_import/bsms.rs:198` | `bsms.rs:198` | MATCH |
| I6 (call site — bitcoin_core) | `wallet_import/bitcoin_core.rs:271` | `bitcoin_core.rs:271` | MATCH |
| I7 (JSON-mode drops canonicalize error) | `cmd/import_wallet.rs:334-338, 396-402` | **lines 425-428** (`canonicalize_*.ok()` pattern); **lines 552-557** (`None =>` branch missing `error` field) | **DRIFTED** — semantically identical, line numbers shifted |
| I8 (unfiled FOLLOWUP slug) | `cost/strip.rs:5,51`; `cost/mod.rs:75` | confirmed at all 3 sites | MATCH — FOLLOWUP filed in v0.27.0 cycle (`53a1bf6`); Phase 3 leaves cites intact |
| I9 (§7.0.a..d SPEC citations) | `wallet_import/bsms.rs:10`; `bitcoin_core.rs:34` | confirmed present + load-bearing | WONTFIX per plan-doc Q3 (anchor verified §2 below) |
| I10 (`error.rs` "Phase N emits") | `error.rs:181-222` | same; "Phase 5 emits" at 182, "Phase 2/3 emits" at 198, "Phase 5 emits" at 203, "Phase 3 emits" at 220 | MATCH |
| I11 ("in Phase 2" user-visible string) | `cost/mod.rs:75` | same — verbatim string confirmed | MATCH |
| I12-I14 (overlay/phrase/non-entropy ms1 untested) | `cli_import_wallet_seed_overlay.rs` | file exists | MATCH |
| I15 (`--select-descriptor` matrix) | `cli_import_wallet_bitcoin_core.rs:107-131` | file exists | MATCH — new cells appended at file end per Q4 |
| I16 (Sniff Ambiguous arm) | `wallet_import/sniff.rs:49` | same — `SniffOutcome::Ambiguous` in match arm | MATCH — dispatch arm in `cmd/import_wallet.rs` actually at **lines 247-252** (plan-doc §7 risk row narrative says "168-170" — that line ref is stale; non-blocking since not a code-fix target) |
| I17-I18 (BSMS line-count + sniff false-positive) | `cli_import_wallet_bsms.rs` + `bsms.rs:47-57` | files exist | MATCH |
| I19 (multisig roundtrip byte_exact) | `cli_import_wallet_roundtrip.rs:371-452` | file exists | MATCH |
| I20 (`ParsedImport` invalid pair) | `wallet_import/mod.rs:60` | struct def at 60; `bsms_audit: Option<...>` at 74; `source_metadata: Option<...>` at 79 | MATCH |
| I21 (`BsmsAuditFields.signature_verified: bool`) | `wallet_import/mod.rs:188` | struct def at 188; `signature_verified: bool` at 193 | MATCH |

### Drift summary

- **2 drifted line ranges requiring plan-doc correction (Phase 1 only):** C1 and I7. Both must be corrected before Phase 1 begins (S1 below).
- **5 informational line shifts:** I16 dispatch arm (247-252 not 168-170 — plan-doc §7 risk row narrative only; non-blocking), `Round1VerificationStatus` precedent at line 844 not 843 (S5; trivial), Phase 5a Q5b internal-grep results (8 internal hits across 3 structs — S4), and the `cli_import_wallet_envelope_v0_27_0.rs` precedent file for Phase 5 fixture cells.
- **I8 effectively pre-resolved:** Phase 3 should leave cites intact, not edit them.
- **All Q5a/Q5b/Q6 type-design refactor sites verified present.**

---

## §2. Q3a §7.0 Anchor Verification

**Grep command:** `grep -n '§7\.0\.'` against IMPLEMENTATION_PLAN, SPEC, and source.

**Results:**

- `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md`: ≥6 hits at lines ~565-574 (`§7.0.a` through `§7.0.e` each as a separate checkbox item) + line ~586 (`§6.11.a` sub-label derived from §7.0.b). File confirmed present.
- `design/SPEC_wallet_import_v0_26_0.md`: 2 hits — line 102 (`"locked rule per §7.0.d"`) + line 126 (`"Per §7.0.b: ..."`)
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:10`: `"§7.0.a locked"`
- `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:34`: `"§7.0.a locked"`

**Total confirmed hits:** ≥10 (threshold was ≥4).

**Interpretation:** I9 is confirmed **wontfix**. The `§7.0.a` source-comment citations refer to a real cross-doc anchor; SPEC body itself depends on `§7.0.b/d` sub-labels. Phase 3 leaves both source comments untouched.

---

## §3. Fixture Capture Plan

Target: `tests/fixtures/v0_27_0_envelopes/{path,passphrase,account}_of_{xpub,descriptor}.{match,no_match}.json` — 6 files.

**Test constants (from existing integration tests):**
- Master phrase (BIP-39 all-zeros 12-word): `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
- "Other" phrase (no match): `legal winner thank year wave sausage worth useful legal winner thank yellow`
- Passphrase for passphrase-of-xpub: `satoshi`

The xpubs must be pre-computed at runtime in the capture script (deterministic from BIP-32 + phrase + path); the simplest approach is a temporary Rust helper at capture time that derives them, then deleted post-capture.

### Per-fixture invocations

| Fixture | Subcommand pattern | Exit code | Notes |
|---|---|---|---|
| `path_of_xpub.match.json` | `xpub-search path-of-xpub --phrase-stdin --target-xpub <xpub@m/84'/0'/0' from master phrase> --json` | 0 | stdin = master phrase |
| `path_of_xpub.no_match.json` | `... --target-xpub <xpub@m/84'/0'/0' from OTHER phrase> --json` | 4 | stdin = master phrase; stdout still valid JSON despite exit 4 |
| `passphrase_of_xpub.match.json` | `xpub-search passphrase-of-xpub --phrase-stdin --passphrase satoshi --target-xpub <xpub@m/84'/0'/0' with satoshi> --json` | 0 | stdin = master phrase |
| `passphrase_of_xpub.no_match.json` | `... --passphrase nakamoto --target-xpub <xpub@m/84'/0'/0' with satoshi> --json` | 4 | wrong passphrase; stdin = master phrase |
| `account_of_descriptor.match.json` | `xpub-search account-of-descriptor --phrase-stdin --descriptor "wpkh([<fp>/84'/0'/0']<xpub>/<0;1>/*)" --json` | 0 | stdin = master phrase; literal-xpub descriptor |
| `account_of_descriptor.no_match.json` | `... --descriptor "wpkh([<fp-other>/84'/0'/0']<xpub-OTHER>/<0;1>/*)" --json` | 4 | stdin = master phrase |

**Capture script approach:** a Rust integration-cell-style helper at Phase 0/Phase 5a transition that derives the 6 xpub values via `xpub_at(...)` (existing test helper in `tests/cli_xpub_search_path_of_xpub.rs`), runs the binary, and writes stdout to the 6 fixture paths. Delete the helper post-capture.

### Expected JSON shapes

**path_of_xpub MATCH:**
```json
{"schema_version":"1","mode":"path-of-xpub","result":"match","path":"m/84'/0'/0'","template":"bip84","account":0,"target_xpub_canonical":"<xpub>","target_xpub_variant":null,"searched_count":<N>}
```

**path_of_xpub NO_MATCH:**
```json
{"schema_version":"1","mode":"path-of-xpub","result":"no_match","path":null,"template":null,"account":null,"target_xpub_canonical":"<xpub>","target_xpub_variant":null,"searched_count":<N>}
```

Note the `path:null, template:null, account:null` on no-match — this is the wire-shape constraint that drives Q5a's private-constructor approach (per plan-doc R1 smoke test).

---

## §4. mnemonic-gui External-Zero Grep (Q5b)

**GUI repo location:** `/scratch/code/shibboleth/mnemonic-gui` (confirmed present)
**Pin:** `docs/manual-gui/pinned-upstream.toml:19` pins `mnemonic-gui-v0.3.0` (manual-gui pin; CLI schema-mirror tracks separately but GUI is not bumped this cycle)

**External grep:**
```
grep -rn 'PathOfXpubResult {\|PassphraseOfXpubResult {\|AccountOfDescriptorResult {' /scratch/code/shibboleth/mnemonic-gui/src
```
**Result: 0 matches.** Confirmed — these structs are toolkit-binary-internal only.

**Phase 5a R0 internal-grep results:**

| Struct | Direct-literal construction sites |
|---|---|
| `PathOfXpubResult` | `path_of_xpub.rs:243` (match) + `:289` (no-match) + `mod.rs:143` (unit test) + `mod.rs:169` (unit test) — **4 hits** |
| `PassphraseOfXpubResult` | `passphrase_of_xpub.rs:284` (match) + `:328` (no-match) — **2 hits** |
| `AccountOfDescriptorResult` | `account_of_descriptor.rs:285` (match) + `:337` (no-match) — **2 hits** |

**Total internal:** 8 (including 2 unit-test sites). Within Q5b's expected N range. The unit-test sites at `mod.rs:143/169` can either adopt builders or stay as direct-literal (test-only — invariant violation surface is limited).

---

## §5. New Findings and Surprises

### S1 — **ACTION REQUIRED:** Plan-doc Phase 1 file list has stale C1 + I7 line cites

Plan-doc §4 Phase 1 "Files (impl)" lists `:471-478` (C1) and `:334-338, 396-402` (I7). Actual locations on `release/v0.27.0` tip:
- **C1:** `cmd/import_wallet.rs:710-736` (function `emit_roundtrip_stderr_warning`); two `Err(_) => return Ok(())` arms at lines **720 + 724**.
- **I7:** `cmd/import_wallet.rs:425-428` (`canonicalize_*.ok()` discarding error); `cmd/import_wallet.rs:552-557` (`None =>` branch emitting `canonicalize_failed` status without `error` field).

**Fold action:** plan-doc R3 amendment updating Phase 1 file list. The behavioral fix description is functionally correct — only navigation aids need correction.

### S2 — `let _ = slot_idx; // reserved` confirmed at I5 fix site

`wallet_import/json_envelope.rs:269` has the exact self-confessed gap I5 identifies. Phase 2's I5 fix threads `slot_idx` through to the NOTICE emit. The fix lives in one function (`mk1_card_to_resolved_slot`, which already takes `slot_idx: usize` as a parameter).

### S3 — Phase 3 plan-doc file-list contradicts I9 wontfix lock

Plan-doc §4 Phase 3 lists `wallet_import/bsms.rs:10` + `bitcoin_core.rs:34` under "I9 — §7.0.a..d citations rewritten." But Q3 (§3) locks I9 as **wontfix**. The Phase 3 file list reads as if a Phase 3 edit will happen there. Recommendation: amend Phase 3 file list to remove the I9 entries entirely + add explicit "skip (I9 wontfix per Q3)" note.

### S4 — `cli_import_wallet_envelope_v0_27_0.rs` is the precedent for Phase 5 fixture cells

This test file (added in v0.27.0 Phase 4) contains cell 8 "Envelope wire-shape fixture comparison" against `tests/fixtures/wallet_import/envelope_v0_27_0.json`. **This is the structural template for the 5 v0.27.1 drift cells** (3 in Phase 5a + 1 each in 5b/5c). The plan-doc should reference this precedent in Phase 5 step-order narrative.

### S5 — `Round1VerificationStatus` enum at line 844, not 843

Plan-doc §4 Phase 5c says "mirror `Round1VerificationStatus` enum from `cmd/import_wallet.rs:843-850`". Actual: enum starts line 844; struct it belongs to (`Round1Verification`) at line 835. Off-by-one in a precedent reference; non-blocking.

### S6 — `compare-cost-single-leaf-tr-input` slug status confirmed open

`grep -A1 '^### \`compare-cost-single-leaf-tr-input\`' design/FOLLOWUPS.md` returns `**Status:** open`. Plan-doc Phase 6 task 4 (M3 fold) gate condition satisfied at recon time; re-verify at cycle close.

---

## §6. Phase 1 Readiness Gate

**Verdict: YELLOW — pending plan-doc R3 amendment for S1 + S3.**

Required pre-Phase-1 actions:
1. Amend plan-doc Phase 1 file list: C1 cite `:471-478` → `:710-736` (Err arms at 720+724); I7 cite `:334-338, 396-402` → `:425-428` (`.ok()`) + `:552-557` (missing-error branch). R3 bump.
2. Amend plan-doc Phase 3 file list: remove I9 entries OR add explicit "skip per Q3 wontfix lock" annotation. Same R3.
3. Optional: bump `Round1VerificationStatus` precedent cite in Phase 5c to `:844-850` (S5).

All other Phase 0 preconditions GREEN:
- ✅ All 5 open FOLLOWUPs exist in `design/FOLLOWUPS.md` with `**Status:** open`
- ✅ Q3a §7.0 anchor verified (≥10 grep hits)
- ✅ I9 wontfix confirmed; Phase 3 skips those files
- ✅ Q5b internal-call-site count: PathOfXpub=4, PassphraseOfXpub=2, AccountOfDescriptor=2 (8 total) — within expected N range
- ✅ GUI external-zero confirmed (0 hits — unblocks v0.28+ option (c))
- ✅ Fixture capture plan fully specified (§3)
- ✅ `compare-cost-single-leaf-tr-input` slug resolves + status open
- ✅ All Q5a/Q5b/Q6 type-design refactor sites verified

---

## §7. Files inspected (absolute paths)

- `/scratch/code/shibboleth/mnemonic-toolkit/design/PLAN_v0_27_1_pr_26_fold_cycle.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/pr-26-post-merge-comprehensive-review.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_wallet_import_v0_26_0.md`
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (C1 @ 710-736; I7 @ 425-428 + 552-557; `Round1Verification`/`Status` @ 835-850)
- `crates/mnemonic-toolkit/src/wallet_import/{bitcoin_core,bsms,json_envelope,mod,sniff}.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/{mod,path_of_xpub,passphrase_of_xpub,account_of_descriptor}.rs`
- `crates/mnemonic-toolkit/src/{env_sentinel,error}.rs`
- `crates/mnemonic-toolkit/src/cost/{mod,strip}.rs`
- `crates/mnemonic-toolkit/tests/cli_import_wallet_envelope_v0_27_0.rs` (Phase 5 cell precedent)
- `/scratch/code/shibboleth/mnemonic-gui/src/**` (external-zero grep)
- `docs/manual-gui/pinned-upstream.toml` (GUI pin reference)
