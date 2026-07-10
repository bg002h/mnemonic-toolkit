# SPEC — Cycle G: repair-engine secret zeroization + compare-cost multipath support

**Two small, independent toolkit FOLLOWUPs burned down as one cycle: (1) wrap the repair engine's owned
corrected-/original-secret buffers in the redacting `SecretString` (defense-in-depth — the values are
secret-adjacent bearer material that today drop un-zeroized); (2) make `compare-cost --descriptor` accept
multipath `/<0;1>/*` (and the `/**` shorthand) by splitting to the receive branch instead of rejecting.**

- **Author:** Opus 4.8. **R0 review:** Fable (per standing "fable for review, opus for fold", 2026-07-09). **User:** chose the 3-LOW-FOLLOWUP burndown 2026-07-09.
- **Source SHA (recon-verified):** toolkit `f4461c07` (= `mnemonic-toolkit-v0.81.0`). Recon `cycle-prep-recon-zeroization-and-compare-cost-multipath.md`.
- **FOLLOWUPs:** `repair-engine-outcome-zeroization` (filed Cycle F) + `compare-cost-multipath-descriptor-unsupported` (filed Cycle C).
- **Target:** `mnemonic-toolkit` **MINOR (`v0.82.0`)** (the zeroization is a secret-type migration = MINOR per the standing sweep ruling; the batched cycle takes the higher bump). md/mk/ms NO-BUMP; no GUI/`schema_mirror` (no clap surface change); no crates.io publish (toolkit).
- **Status:** DRAFT — pending Fable R0 loop to 0C/0I BEFORE implementation (CLAUDE.md).

## §0 — Scope

**IN:**
1. **Repair-engine zeroization** (`repair-engine-outcome-zeroization`): migrate the plain-`String` secret-bearing
   fields of the repair engine's owned outcome types to the existing redacting `SecretString`:
   - `RepairOutcome.corrected_chunks: Vec<String>` (`repair.rs:437-462`) → `Vec<SecretString>`.
   - `RepairDetail.original_chunk` + `RepairDetail.corrected_chunk` (`repair.rs:424-432`) → `SecretString`.
   `SecretString` already provides a REDACTING `Debug` (so a `RepairOutcome` debug-print can't leak the seed) and
   a **transparent `Serialize`** (so the deliberate D9 UX — the corrected chunk on stdout / in the `--json`
   `RepairJson` envelope — is byte-preserved; NO wire change). Keep caller edits near-zero via `Deref<Target=str>`
   coercion; add a `PartialEq<str>`/`PartialEq<&str>` impl on `SecretString` (it has none today) for the ~11
   test `assert_eq!` sites (recon). This is defense-in-depth (the values already transit argv/stdout by design in
   the repair UX) — [[feedback_secret_hygiene_first_class_bar]].
2. **compare-cost multipath** (`compare-cost-multipath-descriptor-unsupported`): in
   `cost/strip.rs::translate_descriptor` (recon: now `:35-37`, drifted from the filed `:26-27` — Cycle C inserted
   a comment block), when the parsed descriptor `is_multipath()`, split via `.into_single_descriptors()` and cost
   the **receive branch** (index 0) — cost is chain-index-independent — instead of calling `derive_at_index(0)`
   directly (which errors "multipath key cannot be a DerivedDescriptorKey"). Mirror the shipped prior-art
   `derive_address.rs:26-66`. INVERT the existing regression test that asserts multipath rejection → assert
   acceptance + the correct cost. `/**` inherits this for free (it pre-expands to `/<0;1>/*` upstream, Cycle C).

**OUT:**
1. `gui-manual-repair-exit-code-lockstep` — the 3rd burndown item, a SEPARATE GUI-manual docs pass (different
   repo/book) done AFTER this toolkit cycle.
2. Any change to the repair EXIT-code / wire behavior (zeroization is representation-only; compare-cost is
   additive-accept only). Broadening zeroization beyond the repair engine's owned buffers (other subsystems are
   out of scope — this closes only the filed FOLLOWUP).

## §1 — Item 1: repair-engine zeroization
- **Carrier = `SecretString`** (existing type; redacting `Debug`, transparent `Serialize`) — NOT bare
  `Zeroizing<String>` (which lacks the redacting Debug). Confirms secret-hygiene: zeroize-on-drop + redacting
  Debug + (the value is deliberately emitted on stdout in the repair UX, so transparent Serialize is correct, not
  a leak).
- **Migration surface:** the 3 fields above + their construction sites in `repair.rs` (`repair_card`,
  `repair_via_ms_codec`/`_mk_codec`/`_md_codec`, `apply_ms_corrections`, the indel path) + readers. Callers that
  do `&outcome.corrected_chunks[i]` / `.as_str()` keep working via `Deref`; the `--json`/text emitters
  (`cmd/repair.rs`) serialize transparently (verify byte-identical output). The verify-bundle
  `ms1_ground_truth_compare` local `Zeroizing` clone (Cycle F) can consume the `SecretString` directly.
- **Tests:** add `PartialEq<str>` to `SecretString`; the ~11 `assert_eq!(outcome.corrected_chunks[i], "…")`
  sites compile against it. Add a redaction unit test: `format!("{:?}", outcome)` contains NO seed substring.
  Confirm `--json` + text repair output BYTE-IDENTICAL (no wire change) via the existing golden/CLI tests.
- **SemVer:** MINOR (secret-type migration; precedents v0.71.0 T1, v0.67.0 L22 — overrides the older PATCH
  outlier v0.53.6 that predates the ruling).

## §2 — Item 2: compare-cost multipath
- **Fix:** `translate_descriptor` — `if descriptor.is_multipath() { let single =
  descriptor.clone().into_single_descriptors()?.remove(0); … } else { <today's path> }`, then `derive_at_index(0)`
  on the single-path descriptor. Mirror `derive_address.rs:26-66` (its `into_single_descriptors()` +
  first-branch pattern) for error handling (empty-branches guard). Cost is chain-index-independent → the receive
  branch is representative.
- **Test:** the existing regression test asserting `compare-cost --descriptor "…/<0;1>/*"` REJECTS must be
  INVERTED to assert it now succeeds with the same cost as the equivalent single-path `…/0/*` descriptor. Add a
  `/**` cell (equivalence: `/**` costs identically to `/<0;1>/*` and `/0/*`). A genuinely-malformed multipath
  still errors.
- **SemVer (R0 FOCUS — recon flagged debatable):** MINOR recommended (closest precedent v0.78.0 = MINOR for a
  descriptor-acceptance broadening; a previously-erroring input now succeeds = a new capability). PATCH
  counter-argument: compare-cost is a non-funds cost-analysis convenience. **R0 to rule.** Either way the batched
  cycle is ≥MINOR (item 1). 
- **Manual:** an optional non-gating note in the `compare-cost` chapter that multipath/`/**` descriptors are now
  accepted (costed on the receive branch). Add if low-effort; not a lockstep gate (no flag change).

## §3 — Cross-source anchors (recon-verified @ f4461c07)
- `src/repair.rs`: `RepairOutcome` `:437-462` (`corrected_chunks: Vec<String>`), `RepairDetail` `:424-432`
  (`original_chunk`/`corrected_chunk`); construction in `repair_card` + the per-codec `repair_via_*` +
  `apply_ms_corrections` + indel path.
- `SecretString` (existing carrier — redacting Debug + transparent Serialize; NO `PartialEq<str>` yet) — locate
  its module.
- `src/cost/strip.rs::translate_descriptor` `:35-37` (the `derive_at_index(0)` call, no `into_single_descriptors`).
- Prior-art `src/derive_address.rs:26-66` (`is_multipath()` + `into_single_descriptors()` split).
- `cmd/repair.rs` (`RepairJson`/text emitters — transparent-serialize check).

## §4 — Test / risk matrix
1. Zeroization redaction: `{:?}` of `RepairOutcome`/`RepairDetail` leaks NO seed (unit).
2. Zeroization no-wire-change: `mnemonic repair --ms1/--mk1/--md1` text + `--json` output byte-identical to
   v0.81.0 (existing goldens/CLI tests stay green).
3. `PartialEq<str>` on `SecretString` — the ~11 `assert_eq!` sites compile + pass.
4. compare-cost multipath ACCEPT: `--descriptor "wsh(...xpub.../<0;1>/*)"` → succeeds, cost == the single-path
   `…/0/*` equivalent; `/**` == `/<0;1>/*` == `/0/*` (equivalence cell); the INVERTED prior rejection test.
5. compare-cost regression: single-path descriptors unchanged; a malformed multipath still errors.
6. Full `cargo test -p mnemonic-toolkit` green.

## §5 — Cross-repo / release
- **Toolkit only.** md/mk/ms NO-BUMP; no GUI/`schema_mirror` (no clap flag/subcommand/dropdown change — verify).
  No sibling-pin change (no sibling release). No crates.io publish.
- **SemVer:** MINOR `v0.82.0`.
- **Release ritual (standard toolkit):** version sites (Cargo.toml + workspace/fuzz Cargo.lock + both READMEs +
  install.sh:32 self-pin) + `.examples-build` corpus (version pin; only version strings move — no repair/cost
  content change expected, verify) + CHANGELOG `[0.82.0]` + flip BOTH FOLLOWUPs → RESOLVED in the shipping
  commit + regen Examples.md + NO re-vendor (no dep change). Tag `mnemonic-toolkit-v0.82.0`; push; verify CI
  (incl. `examples`, `changelog-check`).

## §6 — R0 focus
1. **compare-cost SemVer** (MINOR vs PATCH) — rule explicitly.
2. **Zeroization no-wire-change** — the transparent `Serialize` on `SecretString` must byte-preserve the repair
   `--json`/text output (the corrected chunk is deliberately emitted; this is NOT a leak, do not redact it on the
   wire). Confirm the emitters don't `format!("{:?}")` (which would now redact).
3. **`PartialEq<str>` scope** — the impl is test-ergonomics; confirm it doesn't accidentally weaken a
   production comparison.
4. **compare-cost `into_single_descriptors` empty-branch / error handling** — mirror `derive_address.rs`'s
   guards; a 0-branch or malformed split must error cleanly, not panic/unwrap.
5. **Batch independence** — zero file overlap (repair.rs/SecretString vs cost/strip.rs); confirm no shared type.

---
*R0 gate: converge to 0C/0I via the Fable-architect loop (persisted to `design/agent-reports/`) BEFORE
implementation; Opus folds. Per CLAUDE.md + user directive.*
