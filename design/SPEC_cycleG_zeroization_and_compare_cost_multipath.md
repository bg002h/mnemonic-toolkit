# SPEC — Cycle G: repair-engine secret zeroization + compare-cost multipath support

**Two small, independent toolkit FOLLOWUPs burned down as one cycle: (1) wrap the repair engine's owned
corrected-/original-secret buffers in the redacting `SecretString` (defense-in-depth — the values are
secret-adjacent bearer material that today drop un-zeroized); (2) make `compare-cost --descriptor` accept
multipath `/<0;1>/*` (and the `/**` shorthand) by splitting to the receive branch instead of rejecting.**

- **Author:** Opus 4.8. **R0 review:** Fable (per standing "fable for review, opus for fold", 2026-07-09). **User:** chose the 3-LOW-FOLLOWUP burndown 2026-07-09.
- **Source SHA (recon-verified):** toolkit `f4461c07` (= `mnemonic-toolkit-v0.81.0`). Recon `cycle-prep-recon-zeroization-and-compare-cost-multipath.md`.
- **FOLLOWUPs:** `repair-engine-outcome-zeroization` (filed Cycle F) + `compare-cost-multipath-descriptor-unsupported` (filed Cycle C).
- **Target:** `mnemonic-toolkit` **MINOR (`v0.82.0`)** (the zeroization is a secret-type migration = MINOR per the standing sweep ruling; the batched cycle takes the higher bump). md/mk/ms NO-BUMP; no GUI/`schema_mirror` (no clap surface change); no crates.io publish (toolkit).
- **Status:** DRAFT rev-3 — folded SPEC-R0-round-1 (0C/1I/6M): I1 (compare-cost existing test is `wpkh`=UnsupportedWrapper regardless of multipath → UPDATE it to assert the new wrapper error, ADD `wsh` acceptance tests); M1 full migration surface (2nd wire struct AutoFireRepairJson + verify_mk1_set + `&*`); M2 verify-bundle no-`Default` compare; M3 stale comments; M4 malformed-multipath fixture; M5 split-first-mirror-prior-art; M6 slice-serialize unit. Compare-cost SemVer R0-ruled MINOR. + round-2 (0C/1I/2M: I1(r2) §0-scope-line harmonized to UPDATE-not-invert; M1(r2) count 8-not-~11; M2(r2) producer-local note). Reviews `cycleG-spec-r0-round-{1,2}.md`. Pending Fable R0 round-3 to 0C/0I.

## §0 — Scope

**IN:**
1. **Repair-engine zeroization** (`repair-engine-outcome-zeroization`): migrate the plain-`String` secret-bearing
   fields of the repair engine's owned outcome types to the existing redacting `SecretString`:
   - `RepairOutcome.corrected_chunks: Vec<String>` (`repair.rs:437-462`) → `Vec<SecretString>`.
   - `RepairDetail.original_chunk` + `RepairDetail.corrected_chunk` (`repair.rs:424-432`) → `SecretString`.
   `SecretString` already provides a REDACTING `Debug` (so a `RepairOutcome` debug-print can't leak the seed) and
   a **transparent `Serialize`** (so the deliberate D9 UX — the corrected chunk on stdout / in the `--json`
   `RepairJson` envelope — is byte-preserved; NO wire change). Keep caller edits near-zero via `Deref<Target=str>`
   coercion; add a `PartialEq<str>`/`PartialEq<&str>` impl on `SecretString` (it has none today) for the 8 string-element
   test `assert_eq!` sites (recon). This is defense-in-depth (the values already transit argv/stdout by design in
   the repair UX) — [[feedback_secret_hygiene_first_class_bar]].
2. **compare-cost multipath** (`compare-cost-multipath-descriptor-unsupported`): in
   `cost/strip.rs::translate_descriptor` (recon: now `:35-37`, drifted from the filed `:26-27` — Cycle C inserted
   a comment block), when the parsed descriptor `is_multipath()`, split via `.into_single_descriptors()` and cost
   the **receive branch** (index 0) — cost is chain-index-independent — instead of calling `derive_at_index(0)`
   directly (which errors "multipath key cannot be a DerivedDescriptorKey"). Mirror the shipped prior-art
   `derive_address.rs:26-66`. UPDATE the existing (`wpkh`) rejection test to assert the NEW `UnsupportedWrapper`
   error — multipath now gets PAST derivation, NOT to acceptance, since compare-cost rejects `wpkh` regardless
   (see §2) — and ADD `wsh`-wrapper acceptance tests asserting the correct cost (see §2/§4). `/**` inherits this
   for free (it pre-expands to `/<0;1>/*` upstream, Cycle C).

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
- **Migration surface (M1 — complete list):** the 3 fields above + construction in `repair.rs` (`repair_card`,
  `repair_via_ms_codec`/`_mk_codec`/`_md_codec`, `apply_ms_corrections`, the indel path). BOTH wire structs
  widen `corrected_chunks: &'a [String]` → `&'a [SecretString]`: `RepairJson` (`cmd/repair.rs:288-302`) AND the
  auto-fire `AutoFireRepairJson` (`repair.rs:1884-1900`) + `AutoFireRepairJsonDetail` (:1897-1898); the
  `*Detail.original_chunk`/`corrected_chunk` can stay `&'a str` (field-init deref-coerces). `verify_mk1_set`
  (`repair.rs:978`, `corrected_chunks: &[String]`) + its `.as_str()` @:1051 → use `&*` (NOT `.as_str()`, which
  may not resolve through `Deref<str>` at MSRV). Readers doing `&outcome.corrected_chunks[i]` keep working via
  `Deref`; the `--json`/text emitters serialize/`Display` transparently (verify byte-identical output).
- **M2 — verify-bundle `ms1_ground_truth_compare` call site (`verify_bundle.rs:2026-2032`):** it currently wraps
  a clone in `Zeroizing` + `.unwrap_or_default()` — the latter would need `SecretString: Default` (absent). Do
  NOT add `Default`; instead DROP the redundant `Zeroizing` wrap AND the `Option` fallback (the
  `outcome.repairs.is_empty()` guard @:2020-2025 already guarantees `corrected_chunks` non-empty) → compare via
  `outcome.corrected_chunks.first().is_some_and(|c| &**c == expected_ms1)` (or equivalent). Consume the
  `SecretString` directly.
- **Tests:** add `PartialEq<str>` to `SecretString`; the 8 string-element `assert_eq!(outcome.corrected_chunks[i], "…")`
  sites compile against it (the producer locals `let mut corrected_chunks: Vec<String>` @`repair.rs:1098/1126/1660`
  push `.clone()` → `SecretString::new(...)`, compile-enforced). Add a redaction unit test: `format!("{:?}", outcome)` contains NO seed substring.
  Confirm `--json` + text repair output BYTE-IDENTICAL (no wire change) via the existing golden/CLI tests.
- **SemVer:** MINOR (secret-type migration; precedents v0.71.0 T1, v0.67.0 L22 — overrides the older PATCH
  outlier v0.53.6 that predates the ruling).

## §2 — Item 2: compare-cost multipath
- **Fix (M5 — mirror the prior-art structure EXACTLY):** in `translate_descriptor`, split FIRST when
  `descriptor.is_multipath()` — `let single = descriptor.clone().into_single_descriptors()?;` + an
  `is_empty()` guard, `let d = single.remove(0);` — mirroring `derive_address.rs:34-42`, THEN feed the
  single-path `d` into the EXISTING derivation/wrapper path (`derive_at_index(0)` + the `has_wildcard`/`TryFrom`/
  wrapper-match logic). Split-first-then-existing-path handles the non-wildcard-multipath edge (`…/<0;1>` with no
  trailing `/*`) for free, unlike a bolted-on `if/else`. Cost is chain-index-independent (R0-confirmed: receive
  vs change differ only in one child index; same-size keys → identical templates/vbytes) → the receive branch
  (index 0) is representative.
- **IMPORTANT — compare-cost only supports miniscript-WRAPPING descriptors (`wsh`/`tr`), NOT `wpkh`/`pkh`/bare**
  (`strip.rs:59-63` → `UnsupportedWrapper`). The multipath fix gets a descriptor PAST the derivation error, but a
  `wpkh(...)` multipath STILL fails with `UnsupportedWrapper` (a separate, correct rejection). So acceptance
  tests MUST use a supported wrapper (`wsh`).
- **Tests (R0 I1 — the existing test is `wpkh`, CANNOT invert-to-success):**
  1. **UPDATE (rename, NOT invert)** `compare_cost_double_star_rejects_identically_to_explicit_multipath`
     (`tests/cli_bip388_double_star_shorthand.rs:377-414`, `wpkh` fixture): both spellings (`/**` and
     `/<0;1>/*`) now fail IDENTICALLY with the NEW `UnsupportedWrapper` error — assert the stderr NO LONGER
     contains "multipath key cannot be a DerivedDescriptorKey" (pins that multipath now gets PAST derivation) and
     that `/**`≡`/<0;1>/*` still holds on this surface.
  2. **ADD** acceptance tests on a `wsh` wrapper (e.g. `wsh(multi(2,…/<0;1>/*,…))` or `wsh(pk(…/<0;1>/*))`):
     succeeds + cost byte-identical to the single-path `…/0/*` equivalent; `/**` cost == `/<0;1>/*` == `/0/*`
     (equivalence cell).
  3. Malformed multipath (inconsistent branch counts across keys — M4) still errors cleanly via the
     `into_single_descriptors()` error path (no panic).
- **M3 — update the now-false stale comments same-PR:** the Cycle-C block `strip.rs:21-28` ("rejects ALL
  /<0;1>/*") + the test-file comment `:379-384`.
- **SemVer: MINOR (R0-ruled).** A previously-erroring `--descriptor` now succeeding = an observable capability
  addition on the public CLI surface (precedent v0.78.0); the PATCH counter (v0.65.1) was panic→clean-error, not
  accept-widening. Moot for the release number (item 1 independently forces MINOR).
- **Manual:** optional non-gating note in the `compare-cost` chapter (multipath/`/**` now accepted, costed on
  the receive branch); add if low-effort.

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
2. Zeroization no-wire-change: `mnemonic repair --ms1/--mk1/--md1` text + `--json` byte-identical to v0.81.0
   (existing goldens/CLI tests stay green); the auto-fire `AutoFireRepairJson` path likewise.
3. `PartialEq<str>`/`PartialEq<&str>` on `SecretString` — the 8 string-element `assert_eq!` sites compile + pass.
4. **compare-cost multipath ACCEPT — SUPPORTED `wsh` wrapper (I1):** `--descriptor "wsh(multi(2,…/<0;1>/*,…))"`
   (or `wsh(pk(…/<0;1>/*))`) → succeeds, cost byte-identical to the single-path `…/0/*` equivalent; `/**` cost ==
   `/<0;1>/*` == `/0/*` (equivalence cell).
5. **compare-cost wpkh test UPDATED (I1 — not inverted):** the existing `wpkh` `/**`≡`/<0;1>/*` test now asserts
   BOTH fail IDENTICALLY with the NEW `UnsupportedWrapper` error (stderr NO LONGER "multipath key cannot be a
   DerivedDescriptorKey") — pins multipath got past derivation; wpkh still unsupported.
6. **compare-cost malformed multipath (M4):** inconsistent branch counts across keys (`/<0;1>/*` on one key,
   `/<0;1;2>/*` on another in one `wsh(multi(...))`) → errors cleanly via `into_single_descriptors()` (no panic).
   Single-path descriptors unchanged.
7. **M6 slice-serialize unit** (`secret_string.rs`): `Vec<SecretString>` serializes byte-identical to
   `Vec<String>` (the `RepairJson.corrected_chunks` shape).
8. Full `cargo test -p mnemonic-toolkit` green.

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
