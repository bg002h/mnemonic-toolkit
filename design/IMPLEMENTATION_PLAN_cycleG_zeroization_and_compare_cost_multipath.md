# IMPLEMENTATION PLAN — Cycle G: repair-engine zeroization + compare-cost multipath

**SPEC:** `design/SPEC_cycleG_zeroization_and_compare_cost_multipath.md` (✅ R0-GREEN @ round 3, Fable). The SPEC
already carries the precise migration surface + fix + test plan per item; this plan is the phase split +
guard-rails + release. Subject to its own **Fable plan-R0** to 0C/0I BEFORE implementation. Per-phase: TDD +
Fable per-phase R0 (FULL `cargo test -p`) + fold-on-Opus. Post-impl: Fable whole-diff.

**Status:** ✅ plan-R0-GREEN (0C/0I, Fable round 1) — 4 non-blocking Minors folded (M1 cite the value/key-order assert sites not "goldens"; M2 the `verify_bundle.rs:2014-2015` stale `Zeroizing` comment; M3 gen.sh ~6 version occurrences → bump globally; M4 SHA). Review `cycleG-plan-r0-round-1.md`. CLEARED for implementation.

**Source SHA:** toolkit `46b2ec4d` (v0.81.0 line + Cycle G design). **Target:** MINOR `v0.82.0`; md/mk/ms
NO-BUMP; no GUI/`schema_mirror`; no crates.io publish. **Worktree:** one `mnemonic-toolkit` worktree (branch
`feature/cycleG-zeroization-compare-cost`). Single implementer, sequential (P0 then P1 — zero file overlap, so
order is free; do P0 first). The two items are INDEPENDENT.

## Phase P0 (item 1) — repair-engine secret zeroization
**Files (SPEC §1):** `src/repair.rs` (`RepairOutcome.corrected_chunks`→`Vec<SecretString>`;
`RepairDetail.original_chunk`/`corrected_chunk`→`SecretString`; construction in `repair_card`/`repair_via_*`/
`apply_ms_corrections`/indel + the producer locals @`:1098/1126/1660`→`SecretString::new`; `verify_mk1_set`
`:978` param + `&*` @`:1051`; the auto-fire `AutoFireRepairJson`/`AutoFireRepairJsonDetail` `:1884-1900`);
`src/cmd/repair.rs` (`RepairJson.corrected_chunks: &'a [String]`→`&'a [SecretString]`); `src/secret_string.rs`
(add `PartialEq<str>` + `PartialEq<&str>`; a slice-serialize unit test); `src/cmd/verify_bundle.rs:2026-2032`
(drop `Zeroizing`+`Option`/`unwrap_or_default`; `.first().is_some_and(|c| &**c==expected_ms1)`) + update the
now-false `:2014-2015` doc-comment ("held in `Zeroizing` … §8 risk 6 / G5") — M2.

**TDD — tests first (SPEC §4.1/4.2/4.3/4.7):**
- Redaction unit: `format!("{:?}", outcome)` / `RepairDetail` debug contains NO seed substring.
- No-wire-change (M1 — these are value + raw-key-ORDER asserts, NOT raw golden files): the existing JSON
  value/key-order pins `cli_repair.rs:99-103` + the auto-fire `cli_auto_repair.rs:307`/`:424-425` stay green
  (byte-identical output; a silent redaction fails them immediately). Optionally add one raw-string envelope
  comparison. The auto-fire `AutoFireRepairJson` path is covered by the latter.
- `PartialEq<str>`/`<&str>` on `SecretString` — the 8 string-element `assert_eq!` sites compile + pass.
- `secret_string.rs` slice-serialize unit: `Vec<SecretString>` serializes byte-identical to `Vec<String>`.

**Guard-rails:** G0-1 the emitters MUST serialize (serde-transparent) / `Display`, NEVER `{:?}` (which now
redacts — would silently drop the corrected chunk the user needs). G0-2 no new leak surface (redacting Debug is
the ONLY behavior change; wire byte-identical). G0-3 do NOT add `SecretString: Default`.

**Per-phase Fable R0** (FULL suite) → 0C/0I. Persist `cycleG-phase-P0-r0-round-N.md`.

## Phase P1 (item 2) — compare-cost multipath
**Files (SPEC §2):** `src/cost/strip.rs` (`translate_descriptor` — split-FIRST when `is_multipath()` via
`into_single_descriptors()?` + `is_empty()` guard + `remove(0)`, then feed the single descriptor into the
EXISTING `has_wildcard`/`TryFrom`/wrapper path; mirror `derive_address.rs:34-60`; update the stale Cycle-C
comment `:21-28`); `tests/cli_bip388_double_star_shorthand.rs` (UPDATE the wpkh test `:377-414` + its comment
`:379-384`; ADD wsh acceptance).

**TDD — tests first (SPEC §4.4/4.5/4.6):**
- (ACCEPT) `compare-cost --descriptor "wsh(multi(2,…/<0;1>/*,…))"` (or `wsh(pk(…/<0;1>/*))`) → succeeds, cost
  byte-identical to the single-path `…/0/*` equivalent; `/**` cost == `/<0;1>/*` == `/0/*` (equivalence cell).
- (UPDATE wpkh) the existing `compare_cost_double_star_rejects_identically_to_explicit_multipath` — both `/**`
  and `/<0;1>/*` now fail IDENTICALLY with the NEW `UnsupportedWrapper` error; assert stderr NO LONGER contains
  "multipath key cannot be a DerivedDescriptorKey".
- (malformed) inconsistent branch counts across keys in one `wsh(multi(...))` → errors cleanly (no panic).

**Guard-rails:** G1-1 split-first (mirror prior-art) so the non-wildcard `…/<0;1>` edge is handled; G1-2
empty-branch guard (no `remove(0)` panic); G1-3 single-path descriptors unchanged; G1-4 the receive-branch cost
== the change-branch cost (index-independent — pin via the equivalence cell).

**Per-phase Fable R0** (FULL suite) → 0C/0I. Persist `cycleG-phase-P1-r0-round-N.md`.

## Post-implementation (mandatory) — Fable whole-diff
Fresh Fable over the whole diff: the secret-hygiene migration (redacting Debug added, wire byte-identical, no
`{:?}` leak), the compare-cost fix (wsh accept + wpkh still-rejects + malformed errors), no regression, SemVer.
Persist `cycleG-postimpl-whole-diff-review.md`.

## Release ritual (only after whole-diff GREEN) — toolkit v0.82.0
Standard toolkit (no sibling/publish): version sites (Cargo.toml + workspace/fuzz Cargo.lock + both READMEs +
install.sh:32 self-pin `v0.81.0`→`v0.82.0`) + `.examples-build` corpus (gen.sh — bump ALL ~6 `0.81.0`
occurrences `:3/:44/:109/:126/:711/:724`, M3; only version strings move — no repair/cost content change expected,
verify the non-version diff is empty) +
CHANGELOG `[0.82.0]` (leave prior intact) + flip BOTH FOLLOWUPs (`repair-engine-outcome-zeroization` +
`compare-cost-multipath-descriptor-unsupported`) → RESOLVED in the shipping commit + regen Examples.md + NO
re-vendor (no dep change) + NO sibling-pin change (md/mk/ms FROZEN — do NOT touch). Build; full suite; FF master
→ tag `mnemonic-toolkit-v0.82.0` → push (admin-bypass `examples`) → verify CI (`examples`, `changelog-check`,
`install-pin-check`, `sibling-pin-check` unchanged). **USE `git commit -F <file>` (backtick gotcha).**

## Guard-rails (cycle)
- **G-A** no-wire-change (item 1) — the load-bearing secret-hygiene property; emitters serialize/Display, never
  `{:?}`; goldens byte-identical.
- **G-B** compare-cost fix is additive-accept only (wsh) — wpkh still `UnsupportedWrapper`; single-path
  unchanged; malformed errors.
- **G-C** batch independence — zero file overlap; if a per-phase R0 finds a shared touch, flag it.
- **G-D** codecs/GUI untouched — NO-BUMP; no clap surface; no schema_mirror.
