# Impl review — build-descriptor --allow — round 1
**Verdict: GREEN** (0C / 0I / 4M)

The implementation is a faithful, line-by-line realization of the R0-GREEN SPEC. The one-line `ext_check` swap is exactly where the SPEC pinned it, the fired-detection polarity matches both the SPEC's M3 pin and the pinned miniscript rev's `ext_check` dispatch verbatim, the C1 cost posture is a true deterministic skip (no try-and-catch), the never-silent surface fires in every output mode (probed live, including `--json` and `--emit-spec`), and all byte-stability goldens pass untouched. Full suite: 2878 passed / 0 failed; clippy clean across all targets. All four minors are cosmetic or test-coverage-documentation nits.

## Critical

None.

## Important

None.

## Minor

**M1 — §5 still-refuses clause "PLUS the hint" is asserted only for the sigless kind.** §5 requires "no `--allow` on each allow-success input → exit 2 with the SAME diagnostic as before PLUS the new `; rerun with --allow <kebab> after review` hint." The hint is asserted for sigless twice (`crates/mnemonic-toolkit/src/descriptor_builder/gate.rs:709-716` unit; `crates/mnemonic-toolkit/tests/cli_build_descriptor.rs:885-902` `allow_wrong_rule_still_refuses_with_hint`), but the repeated-keys no-allow refusal (the replay leg of `emit_spec_records_no_allowance`, `cli_build_descriptor.rs:955-963`) asserts kind only, and there is no mixed-timelock no-allow cell. Risk is near-nil — the hint is mechanically derived from `kind.as_str().replace('_',"-")` (`gate.rs:359-363`) and the drift self-test (`src/cmd/build_descriptor.rs:544-551`) pins the token alignment for all 5 — but the SPEC's wording says "each." One `contains("rerun with --allow repeated-keys")` on the replay assertion would close it.

**M2 — the malleable/resource-limit integration-cell gap is real (as the SPEC anticipated) but not explicitly DOCUMENTED in the new test section.** §5: "cover via gate-level unit tests driving `ext_check` + the predicate directly and DOCUMENT the integration-cell gap." The per-variant `AllowSet`→`ExtParams` mapping cells exist (`gate.rs:646-669`, incl. the `raw_pkh`-never-set assertion), which is what the SPEC itself declares sufficient ("the per-variant mapping is what needs pinning, and the unit cells pin it"). The constructibility rationale lives in the pre-existing M1 comment block (`gate.rs:997-1005`), but the new `--allow` test section (`gate.rs:640` ff., `cli_build_descriptor.rs:779` ff.) carries no cross-reference, and fired-detection-via-predicate is unit-exercised only for sigless (`gate.rs:690-705`; repeated-keys and mixed-timelock fired detection are proven end-to-end by the integration banners, `cli_build_descriptor.rs:855-883` — malleable/resource-limit remain mapping-only, consistent with the SPEC's best-effort posture). A one-line comment in the `--allow` section pointing at the `gate.rs:997` rationale satisfies the "DOCUMENT" instruction.

**M3 — human-stderr refusal suffix order: the provenance suffix now trails the hint.** Probed: `[repeated_keys] root.or_d[0]: this subtree reuses a public key (RepeatedPubkeys); rerun with --allow repeated-keys after review (from --key)` — `(from --key)` reads as if attached to the hint rather than the diagnostic. Mechanism: the hint is inside `message` (`gate.rs:359-363`) while provenance is appended at print time (`src/cmd/build_descriptor.rs:380-388`). The SPEC pinned neither order, no test pins the concatenation, and the existing `contains("(from --key)")` assertions survive — cosmetic only; worth a swap at a future touch.

**M4 — ship-time remainders, outstanding by design (reminder, not a defect of this commit):** §6's A1-FOLLOWUP extension in BOTH repos (repeating-Dropdown note, `allowed_rules_fired` + `cost: null` wire notes, stale "(to be filed in the GUI repo)" companion-line fix), the M-r2-1 `--spec … --emit-spec` silent-ignore docs/FOLLOWUP one-liner, the `[0.52.0]` CHANGELOG section, and the version bump (crate still 0.51.0) are all assigned to the release commit. None are present in `4648861` — correct per the SPEC, but they gate the v0.52.0 push.

## SPEC-conformance checklist

**§1 CLI surface — CONFORMS.**
- `CliAllow`: alphabetical declaration, kebab via `rename_all`, exactly 5 variants, `raw_pkh` excluded with the IR rationale in the doc comment (`src/cmd/build_descriptor.rs:115-132`).
- `Vec<CliAllow>` + `value_enum`, no `requires`/`conflicts` edges (`build_descriptor.rs:111-112`); clap derive gives Append semantics for `Vec` (repeatable — probed).
- Duplicate tokens idempotent: `allow_set` collapses to bools (`build_descriptor.rs:156-168`); banner names the rule ONCE (probed; pinned by `allow_duplicate_tokens_idempotent`, `cli_build_descriptor.rs:976-991`).
- Refusal hint fires ONLY for step-3 allowable kinds: `localize_sanity`'s match binds `kind` to one of the 5 step-3 kinds (`gate.rs:326-345`); `ContainsRawPkh` early-returns a `TypeError` root diagnostic WITHOUT the hint (`gate.rs:346-351`) — the hint format at `gate.rs:359-363` is unreachable for any non-allowable kind, so the `_ => "sanity_check failure"` arm in `sanity_message` (`gate.rs:385`) can never produce a wrong-token hint (it is dead from `localize_sanity`; kept for match exhaustiveness only). Step-1/2/4 diagnostics are built by other constructors (`field_diag`/`root_diag`/`check_cap`) — no hint leakage.

**§2 gate integration — CONFORMS.**
- `AllowSet` gate-local, 5 bools, no clap import in gate.rs (`gate.rs:46-58`; gate.rs imports verified at `gate.rs:19-28`).
- `to_ext_params` mapping verified field-by-field against the pinned rev's `ExtParams` (`gate.rs:60-70` vs `~/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/95fdd1c/src/miniscript/analyzable.rs:28-42`); `raw_pkh` never set; `ExtParams::new()` is all-false (`analyzable.rs:46-55`) so the empty set is the exact baseline.
- `validate`/`validate_with_cap` delegate with the empty set (`gate.rs:142-151`); the `#[allow(dead_code)]` is ACCEPTABLE: both are exercised by ~20 pre-existing gate unit cells plus the new 3-entry-point delegation cell (`gate.rs:673-685`), the suppression carries an explicit API-stability justification in the doc comment (`gate.rs:140-142`), and removing them would churn the whole gate test module for zero behavior gain.
- The swap: `inner_ms.ext_check(&allow.to_ext_params())` at `gate.rs:187` — exactly the SPEC's `:141` site, parse stage untouched (`gate.rs:178-181` still `insane()`).
- Fired polarity verified line-by-line: 3 negated (`!requires_sig` `gate.rs:196`, `!is_non_malleable` `:199`, `!within_resource_limits` `:202`), 2 direct (`has_repeated_keys` `:205`, `has_mixed_timelocks` `:208`) — an exact mirror of both `localize_sanity`'s dispatch (`gate.rs:328-340`) and the pinned `ext_check`. `allowed_fired` push order == `ext_check` check order.
- Fired detection runs after `ext_check` passes and before step 4 — a step-4 refusal discards it (correct: no emit happens).

**§3 UX — CONFORMS.**
- Banner copy names every fired kebab token, unconditional on fired: `emit_allow_notes` is called at BOTH `run()` call sites BEFORE any emit — preset mode at `build_descriptor.rs:300` (ahead of both the `emit_spec` early-return at `:301-312` and `emit` at `:313`) and spec mode at `:330`. `--json` banner probed live; `--emit-spec` banner probed live.
- Backslash-continuation strings emit clean single-space text — probed with `cat -A`.
- Unused-note semantics: per-requested, deduped, only when not fired (`build_descriptor.rs:190-206`); probed text matches §3's template.
- Cost skip deterministic: `--json` → `Value::Null` (`build_descriptor.rs:408-412`, probed `"cost": null`); human → the stdout one-liner in the cost block's position (`build_descriptor.rs:470-476`, probed); `--format descriptor|bip388` untouched (`:435-449`, no cost involvement).
- Envelope key only when non-empty (`build_descriptor.rs:419-424`); default sane `--json` envelope probed: keys exactly `[bip388, cost, descriptor, diagnostics]`, stderr exactly 0 bytes.
- `--emit-spec` records no allowance: the serialized `SpecDoc` has no allow surface; replay refusal pinned (`cli_build_descriptor.rs:938-963`).

**§4 manual — CONFORMS.** Flag row + synopsis `[--allow <RULE>]…` (both modes), and the "Reviewed sanity opt-out" subsection covers all eight §4-required items plus the degrading-threshold example (`docs/manual/src/40-cli-reference/41-mnemonic.md:4011-4044`). No stale verbatim step-3 transcripts elsewhere in the manual (grep clean). cspell += `Pubkeys`.

**§5 tests — CONFORMS (with M1/M2 nits).** Cell map: sigless json (`cli_build_descriptor.rs:799-816`), human (`:821-835`), format-descriptor (`:838-849`); repeated-keys flagship (`:854-866`); KEYED mixed-timelock (`:870-880` — the R0-r1 I1 construction verbatim); wrong-rule-still-refuses + hint (`:885-902`); requested-but-unused (`:906-919`); preset composition (`:923-934`); emit-spec no-allowance + replay (`:938-963`); bip388 dup-`keys_info` (`:967-977`); duplicate-token idempotence (`:979-991`). Gate units: mapping incl. `raw_pkh` (`gate.rs:646-669`), 3-entry-point delegation (`:673-685`), fired + sane-no-fire (`:690-705`), hint (`:708-716`). Drift self-test (`build_descriptor.rs:544-551`). Byte-stability: `spec_mode_json_diagnostics_byte_stable_no_flag_key` passes — its golden is a step-1 `schema_field` message, untouched by the step-3-only hint; all archetype/preset descriptor+bip388 goldens pass unmodified. Counts match the commit message (11 + 5 + 1).

**§6/§7** — single phase as specced; release items deferred (M4).

## Empirical probes run

1. **Full suite:** `cargo test -p mnemonic-toolkit` → 2878 passed / 0 failed. `cargo clippy --all-targets` → clean.
2. **Sigless + `--allow sigless-branch --json`:** exit 0; `allowed_rules_fired: ["sigless_branch"]`, `cost: null`, `diagnostics: []`; stderr banner present in `--json` mode.
3. **Banner whitespace (`cat -A`):** single clean line, single spaces; same for the unused note.
4. **Duplicate tokens:** exit 0; the token appears exactly once on stderr.
5. **Human mode:** descriptor + address + `cost preview unavailable for a sanity-overridden descriptor` on stdout; banner on stderr.
6. **Requested-but-unused:** stderr exactly `note: --allow repeated-keys was requested but did not fire (the policy passes that rule without it)`.
7. **`--emit-spec` + `--allow repeated-keys`:** exit 0; banner on stderr; stdout spec JSON with no allow trace.
8. **Refusal hint + provenance:** exit 2; `…(RepeatedPubkeys); rerun with --allow repeated-keys after review (from --key)` (→ M3).
9. **Default-path byte-safety:** sane `--json` run: envelope keys exactly `[bip388, cost, descriptor, diagnostics]`, stderr 0 bytes.
10. **Pinned-rev source:** `ext_check` order, `ExtParams::new()` baseline, field names, and the five predicate polarities all match the committed code.
