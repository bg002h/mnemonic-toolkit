# C5 (import --format descriptor) plan-R0 round 1 — architect review (verbatim)

> Reviewer: opus architect (general-purpose, full tools — built + ran the binary, traced specter).
> Plan-doc: `design/PLAN_C5_import_format_descriptor_2026-06-17.md` @ toolkit `b15f5e6`. Verdict
> GREEN (0C/0I); 5 Minors folded post-review (see footer).

---

**Verdict: GREEN (0C/0I)**

The plan is architecturally sound and implementable as written. All citations verify against HEAD `b15f5e6`. The three make-or-break questions (checksum, glue path, end-to-end feasibility) are resolved empirically and the plan's chosen directions are correct. No Critical or Important findings. Minors below are documentation/rationale precision fixes the implementer should fold but none gate implementation.

## Critical
None.

## Important
None.

## Minor

**M1 — Decision 5 rationale is factually wrong about specter "requiring" a checksum.** The plan says *"specter does `verify_checksum` which REQUIRES one."* Incorrect. `specter.rs:217-222` calls `miniscript::descriptor::checksum::verify_checksum` (miniscript-13.0.0), whose doc says *"Checks and verifies the checksum **if it is present**"* — when no `#` is present it returns `Ok(&s[..])` with NO error. Confirmed at runtime: `bundle --network mainnet --descriptor <body-without-#csum>` exits 0, while a bad checksum (`#deadbeef`) errors. **The pipeline does NOT require a checksum; it validates-if-present (BIP-380 tolerant mode).** Definitive R0 answer: the new parser should **mirror `bundle --descriptor`: call `verify_checksum` (tolerate-absence, validate-if-present), NOT require a checksum.** Requiring one would be inconsistent with `bundle --descriptor`/`verify-bundle` and would defeat the "subsumes foreign/hand-written commented descriptors" goal. The toolkit's own green/descriptor export always emits a `#csum`, so the round-trip case carries one regardless. Add a TDD cell for the checksum-less foreign-descriptor accept case.

**M2 — Impl step 3 glue-path: pick the explicit specter form definitively.** `descriptor_concrete_to_resolved_slots` (`pipeline.rs:311-350`) returns only `(MdDescriptor, Vec<ResolvedSlot>)` — it does NOT produce `network` or `threshold`, and there is NO shared network/threshold helper in `pipeline.rs` (`network_from_origins` + `extract_threshold_local` are duplicated per-parser). So the parser must compute network via `extract_origin_components(&body, "descriptor")` + a copied `network_from_origins` regardless. **Recommendation: clone the specter explicit sequence** (`verify_checksum` → `concrete_keys_to_placeholders` → `parse_descriptor::parse_descriptor` → `extract_origin_components` → `network_from_origins` → per-slot `finalize_slot_fields` → `validate_watch_only_resolved` → `extract_threshold_local`), giving a `descriptor`-prefixed error namespace and ALL `ParsedImport` fields. Do NOT use `descriptor_concrete_to_resolved_slots` (forces a separate `extract_origin_components`, double-parsing, wrong error prefix). Drop the "OR" and lock the explicit form.

**M3 — Comment-strip precedent citation is narrow.** `descriptor_intake.rs:216-220` (`.lines().map(trim).filter(!empty)`) strips blank lines only — NOT leading `#`-comment lines. The new `strip_comments` must additionally drop full-line `#` comments. Note "blank-strip precedent only; `#`-comment strip is new."

**M4 — GUI is TWO toolkit releases behind, not "one."** The GUI pin is `mnemonic-toolkit-v0.56.0` (`Cargo.toml:42` + `pinned-upstream.toml:22`); HEAD is v0.57.1 — so v0.57.0 (C2) AND v0.57.1 (C1) both intervene. The MEASURE conclusion is still correct: diffing `src/cmd/*.rs` v0.56.0..HEAD shows ZERO clap `long=` flag-name changes, no new subcommands, no new dropdown values → the pin bump v0.56.0→v0.58.0 surfaces only the new `descriptor` VALUE, which `schema_mirror` (flag-NAMES only) ignores → gate PASSES. Fix "one behind" → "two behind."

**M5 — GUI version bump: MINOR is the established convention, not PATCH.** Per the GUI CHANGELOG, the analogous precedent v0.41.0 (schema dropdown-value/flag catch-up + pin bump) was tagged **SemVer-MINOR**, as were v0.38.0/0.39.0/0.40.0. A new user-visible dropdown value + pin bump = feature surface → **MINOR (0.41.0 → 0.42.0)**.

## Confirmations (resolving the asked items)

- **Item 1 (citations):** All spot-checks pass against `b15f5e6`. dispatch `match format_str` at :1157-1173 (plan said :1157-1172, off-by-one tail, immaterial). Manual `41-mnemonic.md:1087` import `--format` row IS stale (`<bsms|bitcoin-core>`, 2/8). advisory hooks :1291/:1295.
- **Item 4 (end-to-end feasibility):** PROVEN. `export-wallet --format green` (singlesig) + `--format descriptor` (singlesig + multisig) emit the expected shapes with `#csum` and `/<0;1>/*` multipath; green refuses multisig, descriptor accepts it. Both round-trip through the target pipeline (`bundle --descriptor <SS>`, `<MS>`, with-and-without-csum all exit 0 + emit md1; bad-csum errors). The specter import suite is green (17 passed) incl. `specter_multisig_2of3_sortedmulti_parses_clean` — "supports both singlesig and multisig" verified.
- **Item 5 (unit variant):** Correct. 8 exhaustive accessors at mod.rs:146-285 each need one `Self::Descriptor => None` arm (alphabetical `ColdcardMultisig` < `Descriptor` < `Electrum`). `--json` `source_format` always set from `format_str`; each `*_source_metadata` block surfaces only on `Some`, so an all-None unit variant emits NO metadata block — identical to `BsmsTwoLine`. `signature_verified()` (mod.rs:482) is on `BsmsVerification`, NOT `ImportProvenance` → no arm needed.
- **Item 6 (explicit-only wiring):** Correct. The `match args.format.as_deref()` block (:512-1071) is exhaustive via `Some(other) => BadInput` — a `Some("descriptor")` arm IS required (else `--format descriptor` errors before dispatch). The arm can be minimal `Some("descriptor") => "descriptor",` with NO sniff-mismatch sub-match. `DescriptorParser::sniff → false` + absence from `sniff.rs` votes → explicit-only. BSMS-encrypted precedent (:214-231) confirms.
- **Item 8 (SemVer/lockstep):** MINOR correct. Version sites all at 0.57.1. No new `ToolkitError` variant — use `ImportWalletParse` (specter's convention) for parser internal errors, `BadInput` for strip/arity refusals.
- **Item 9 (conventions):** Alphabetical insertions verified: provenance variant, `mod descriptor;` decl (between `coldcard_multisig` :29 and `electrum` :30), clap value, dispatch arm, all 8 accessor arms.

Refinement (not a gate): the parser's internal parse-error variant is `ImportWalletParse` (specter's convention); reserve `BadInput` for the strip/arity refusals.

---

## FOLD (post-review, by implementer)

- **M1:** decision 5 corrected — checksum is **tolerant** (`verify_checksum` validate-if-present, mirroring `bundle --descriptor`), NOT required. Added a checksum-less accept TDD cell.
- **M2:** impl step 3 locked to the **explicit specter sequence** (dropped the `descriptor_concrete_to_resolved_slots` "OR").
- **M3:** comment-strip precedent annotated "blank-strip only; `#`-strip is new."
- **M4:** "one behind" → "two behind" (v0.56.0; v0.57.0+v0.57.1 intervene); MEASURE still passes.
- **M5:** GUI bump = **MINOR (v0.41.0 → v0.42.0)**.
- **Error variant:** parser internal errors use `ImportWalletParse`; `BadInput` for strip/arity. (No new variant.)
