# IMPLEMENTATION PLAN — bitcoin-core-receive-change-pair-merge

**Executes `design/SPEC_bitcoin_core_receive_change_pair_merge.md` (R0-GREEN rev-4, SHA `d9063523`).**

- **Model of execution (CLAUDE.md phase-3):** a SINGLE implementer subagent in a git worktree, TDD, phase by
  phase; per-phase opus R0 (FULL `cargo test -p mnemonic-toolkit` suite) to 0C/0I before advancing; mandatory
  post-impl whole-diff review over the entire diff.
- **Status:** ✅ **R0-GREEN (0C/0I) at round 2** (rev-2 + cosmetic tidy). Reviews: `design/agent-reports/cycleB-plan-r0-round-{1,2}.md`. SPEC also R0-GREEN (rev-5). **Cleared for single-implementer TDD execution.**
- **Target:** `mnemonic-toolkit-v0.77.0` (MINOR); codecs NO-BUMP; paired `mnemonic-gui` MINOR.
- **Branch:** `feature/core-receive-change-pair-merge` off `master@0a865f9d` (worktree).

---

## Guard-rails (apply to every phase)

- **TDD:** write the phase's tests FIRST (they must fail for the right reason), then implement to green.
- **Funds-safety first:** the linchpin oracles (P1 tests §8.1 distinct-key-no-merge, §8.4/§8.10/§8.15
  original-anchored address equality) are written and RED before any merge code exists.
- **No `git add -A`** — stage explicit paths. **NEVER `cargo fmt --all`** (mlock.rs fmt-exempt); `cargo fmt -p
  mnemonic-toolkit` only. Clippy `-D warnings` clean each phase.
- **Alphabetical `ToolkitError` variant ordering** if any new variant is added (none anticipated — the merge
  reuses `ToolkitError::ImportWalletParse`).
- **Per-phase gate:** full `cargo test -p mnemonic-toolkit` green + clippy clean + persist the per-phase R0
  review verbatim to `design/agent-reports/cycleB-phase-N-*.md` BEFORE the fold-and-advance step.
- **Plan-phase carry-forwards from SPEC-R0 round 3 (must be honored in code):**
  1. The pre-pass must classify **key-path vs script-path `tr`** from the parsed key/tap-tree structure so a
     script-path `tr` deterministically fails mergeable grouping → floor-reject (§8.15 gate).
  2. The §8.4/§8.10/§8.15 oracle obtains the **merged descriptor string** from `import-wallet --json` (the
     `descriptor` field, sourced from `original_descriptor` at `import_wallet.rs:1491`) — confirmed present;
     build the oracle against it, never against a re-authored `<0;1>`.
  3. The merged JSON entry writes `active = A.active || B.active` (parse_entry reads `active` from JSON;
     `internal` is threaded by param).

---

## Phase P0 — Type migration & provenance plumbing (mechanical, behavior-preserving)

**Goal:** migrate `CoreSourceMetadata.internal: bool → Option<bool>` and thread an explicit merge-provenance
param into `parse_entry`, WITHOUT producing any `None` yet (no merge logic). Existing behavior byte-identical.

**Files:**
- `src/wallet_import/mod.rs` — `CoreSourceMetadata.internal: Option<bool>` (`:355`); `apply_select_descriptor`
  new predicates (`ActiveReceive` = `active && internal != Some(true)`; `ActiveChange` = `active && internal
  != Some(false)`, arms `:411`/`:427`); `sample_core_metadata` (`:534`) → `internal: Some(false)`.
- `src/wallet_import/bitcoin_core.rs` — `parse_entry` gains param `internal: Option<bool>` (replaces the
  in-fn `parse_bool_field(eobj,"internal")` read at `:327`); the single-entry call site in `parse`'s loop
  (`:210`) passes `Some(parse_bool_field(eobj,"internal")?)`.
- `src/cmd/import_wallet.rs` — `--json` (`:1859`) serializes `Option<bool>` (serde → `true`/`false`/`null`,
  no code change beyond the type); text-summary (`:2265`) prints `both` when `None`, else `true`/`false`.

**TDD (write first):**
- **§8.13 STRENGTHEN `core_fixture_file_multipath_receive_change_pair_parses` (`:952`)** — keep `bundles=2`
  AND assert `--select-descriptor active-receive` returns exactly the ONE `internal:false` entry, `active-
  change` exactly the ONE `internal:true` entry. **NOTE (m3-plan): this is a REGRESSION-LOCK, not a red-first
  driver** — the current `bool` impl already passes it, and P0 produces no `None`. Its teeth land in P1
  (where the `None`-tagging code exists): the implementer MUST confirm this test FAILS against a deliberately
  shape-based-`None` stub before wiring the explicit-param mechanism, proving the guard has real signal.
- Existing bitcoin-core suite unchanged-green (the type migration is transparent).

**Gate:** full `cargo test -p` green; clippy clean; per-phase R0.

---

## Phase P1 — Merge pre-pass: guard matrix + construction + all in-scope shapes (FUNDS-CRITICAL CORE)

**Goal:** implement `merge_receive_change_pairs` and wire it into `parse` after the NOTICE emit (`:205`),
before the ParsedImport loop (`:207`). Covers `wpkh`, `wsh(multi/sortedmulti)`, `sh(...)`, nested
`sh(wsh(...))`, single-key bip86 `tr`; script-path `tr` floor-rejects.

**New/changed code (`src/wallet_import/bitcoin_core.rs`):**
1. `recompute_descriptor_checksum(body) -> Result<String, ToolkitError>` — 6th local copy (mirror
   `descriptor.rs:246` via `miniscript::descriptor::checksum::Engine`).
2. `merge_receive_change_pairs(descriptors: Vec<Value>, stderr) -> Result<Vec<(Value, Option<bool>)>, …>`
   (or a `PreparedEntry { value, internal }` struct) — returns entries paired with their explicit `internal`
   provenance (`None` = merged, `Some(bool)` = passthrough). Steps:
   - **Validate each candidate's own BIP-380 checksum** (`verify_checksum`) BEFORE stripping/consuming
     (§4.4 / M9); a bad input checksum → not merge-eligible (surfaces at ordinary parse).
   - Extract the **grouping key** (§4.1) by **parsing the checksum-validated body with rust-miniscript**
     `Descriptor::<DescriptorPublicKey>::from_str(body)` (PLAN-R0 I1-plan — `lex_placeholders` CANNOT be used;
     it is the floor that REJECTS the fixed step, and `extract_origin_components` never captures the use-site
     step). Read per-key `origin` (fp+path), bare xpub, `derivation_path()` (the final use-site step),
     `wildcard` (`/*` hardness), template kind + threshold, and `Tr::tap_tree()` presence. Grouping key =
     `(template kind, threshold, ORDERED per-key (fp, full origin path, xpub), wildcard-hardness)` EXCLUDING
     the final step. Classify key-path vs script-path `tr` via `tap_tree()` (`None` = mergeable single-key;
     `Some` = script-path → grouping key that never matches a mergeable shape → floor-reject; carry-forward 1,
     gated by §8.15). The API is proven in-repo (`tests/prop_backup_restore_roundtrip.rs:357,385`).
   - Bucket by grouping key (first-seen order). For each bucket, apply the §4.2 guard matrix (conditions
     1-7): exactly-two, fixed unhardened final steps, steps differ, internal flags disagree, per-key
     uniform. If ALL hold → merge; else if the bucket is receive/change-**shaped** but keys/scripts differ →
     emit the §7 differentiated distinct-key error directly (exit 2); else leave entries as-is.
   - **Construct** the merged body (§4.3, sanctioned mechanism): after miniscript-verifying **condition-7
     uniformity** (EVERY key in the receive-side body carries the identical `/<recvStep>/*`), do a **global**
     substring replace `/<recvStep>/*` → `/<recvStep;chgStep>/*` on the checksum-stripped receive body
     (ordered receive-first by `internal`; ACTUAL step values). The trailing `/*` makes this unambiguous
     (origin paths end in `']`, never `/*`). FORBIDDEN: first-only/partial `replacen(…,1)` (would leave a
     key at the wrong chain → C1 false-pass). Recompute checksum (§6).
   - **Vec invariant** (§4.4 / M8): remove BOTH members, insert ONE merged entry at the first member's index;
     unpaired keep order; N-pair safe. Merged JSON entry: `desc` = merged+checksum; `active` = `A||B`
     (carry-forward 3); `range` = union/widening (never blocks); `dropped_fields` = union; `internal`
     provenance = `None`.
3. Wire `parse` (`:207` loop) to consume the prepared `(Value, Option<bool>)` list, passing the explicit
   `internal` into `parse_entry` (P0's param).

**TDD (write first — all RED before merge code):**
- **§8.1 `core_receive_change_distinct_keys_must_not_merge`** (LINCHPIN) — distinct-key `/0/*`+`/1/*` → exit
  2, §7 differentiated message.
- **§8.2 FLIP `:926`** / **§8.3 FLIP `:1108`** — same-key pair → merge-accept (`bundles=1`, `<0;1>/*`, valid
  checksum).
- **§8.4 `core_merged_pair_addresses_match_original_split`** (ANTI-C1 ORACLE) — derive external addrs from
  original `.../0/*`, internal from `.../1/*` using the **test-crate miniscript-direct pattern**
  (`Descriptor::<DescriptorPublicKey>::from_str` + `derive_at_index().address()`, mirroring `derive_receive`
  at `tests/prop_backup_restore_roundtrip.rs:383`; NOT the `pub(crate)` `derive_receive_addresses`, which the
  test crate cannot call — m2-plan). Read the merged descriptor from `import-wallet --json` (`descriptor`
  field, emitted `import_wallet.rs:1589`), split via `into_single_descriptors()` → `[chain-0, chain-1]`, and
  assert chain-0 == external-from-original AND chain-1 == internal-from-original. The oracle MUST FAIL (not
  silently fall back to a re-authored `<0;1>`) if the `--json` read yields no descriptor (m4-plan).
  verify-bundle PASSES.
- **§8.5** select-both; **§8.6** lone-receive-rejects; **§8.7** same-step-no-merge; **§8.8**
  both-internal-false-no-merge; **§8.9** three-sharing-key-ambiguous; **§8.10** multisig-merge + original-
  anchored oracle; **§8.11** multisig-partial-split-no-merge; **§8.12** json `internal:null` + text `both`;
  **§8.14** nonstandard-steps `<5;6>`; **§8.15** tr-bip86-merge (+oracle) & tr-scriptpath-does-not-merge;
  **§8.16** hardened/wildcard-hardness-no-merge; **§8.17** corrupt-input-checksum-not-merged.

**Gate:** full `cargo test -p` green (incl. **§8.18 `lint_zeroize_discipline` regression-guard** — no new
secret handling, public xpubs only, so this is intentional coverage, not new test work; m1-plan); clippy
clean; per-phase R0 (weighted to the guard matrix + the original-anchored oracles' non-tautology).

---

## Phase P2 — Docs/examples lockstep (toolkit repo)

**Goal:** correct the now-false manual prose and regenerate any drifted transcripts with the REAL new binary.

**Files:**
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — rewrite the worked-example block (~`:1401-1410`; re-grep
  at impl): Core emits split `/0/*`+`/1/*`, the toolkit auto-recombines into ONE `<0;1>/*` bundle; note the
  distinct-key refusal. (Replaces the false "Bitcoin Core 25+ emits … `<0;1>/*` multipath shape" +
  one-bundle-per-entry sentences.)
- `docs/manual/src/45-foreign-formats.md` — retire the hand-combine workaround; document auto-merge (re-grep).
- `verify-examples` goldens — regenerate any Core-import transcript with the built new-behavior binary; run
  `make -C docs/manual lint MNEMONIC_BIN=<new> …` and confirm green.
- `.examples-build/` — if any example exercises Core split import, refresh
  `EXAMPLES_BIN_DIR="$PWD/target/debug" bash .examples-build/gen.sh > .examples-build/Examples.md`.

**Gate:** `make -C docs/manual lint` green with the new binary; per-phase R0 (docs-fidelity).

---

## Phase P3 — GUI paired PR (cross-repo `mnemonic-gui`)

**Goal:** the un-gated `import-wallet --json` wire-shape changed (`source_metadata.internal` now nullable;
merged entries collapse 2→1). Update the GUI consumer to tolerate `internal: null` and the reduced entry
count. NOT a `schema_mirror` change (no clap flag/subcommand/dropdown delta).

**Steps:** identify the GUI consumer of `import-wallet --json source_metadata.internal`; make it
`Option<bool>`-tolerant; bump GUI MINOR; PR + CI-before-tag (GUI convention); companion `FOLLOWUPS.md`
entries in both repos. (Executed after the toolkit whole-diff review; may land as a paired PR.)

**Gate:** GUI CI green; paired-PR rule satisfied.

---

## Post-implementation whole-diff review (MANDATORY — the autonomous-run ENDPOINT)

After P0-P2 (toolkit) land green, dispatch an independent opus adversarial execution review over the WHOLE
toolkit diff (R0 = plan correctness; this catches implementation-introduced regressions TDD misses). Persist
verbatim to `design/agent-reports/cycleB-postimpl-whole-diff-review.md`. Weighted to: the merge-guard
matrix, the all-keys uniform (global) replacement — no partial rewrite, the original-anchored oracles' non-
tautology, the `Option<bool>` ripple completeness, and no collateral change to other importers.

**CHECKPOINT HERE** — report to the user before the §10 release ritual (Cargo/READMEs/install.sh self-pin/
fuzz lock/CHANGELOG + direct-FF + tag) and before merging the GUI PR to a tag.

---

## Sequencing & dependencies

P0 → P1 (hard: P1 needs the `Option<bool>` + param). P2 depends on P1 (needs the real new binary for
transcript regen). P3 depends on P1 (wire-shape). Whole-diff review after P0-P2. Release ritual (out of this
run's scope) after the checkpoint. No parallel re-implementations — one implementer, sequential phases.

## Risk register (for the plan R0 to pressure-test)

1. **Grouping-key extraction fidelity** — via rust-miniscript parse (PLAN-R0 I1-plan; `lex_placeholders`
   cannot read the fixed step). Must be positional/ordered and correctly exclude the final step while
   capturing wildcard-hardness + tr key-path/script-path classification. A bug here either fails to merge
   (safe-but-broken) or mis-groups (funds — but cond. 2 key identity backstops, so distinct keys still never
   merge).
2. **script-path `tr` classification** — must be robust (tap tree presence), else a script-path `tr` could
   slip into single-key handling. §8.15 refusal test is the gate.
3. **Oracle non-tautology** — §8.4/§8.10/§8.15 MUST derive from the ORIGINAL split descriptors, not the
   merged `<0;1>`; the plan R0 must confirm the test construction can't regress to comparing against a
   re-authored multipath.
4. **`--json`/text `Option<bool>` completeness** — every reader migrated (SPEC notes grep-confirmed no other
   reader of `CoreSourceMetadata.internal`).
5. **NOTICE ordering** — dropped-fields NOTICE runs on the ORIGINAL array (before merge); merged entry
   carries the union.
