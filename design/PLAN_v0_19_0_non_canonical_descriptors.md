# PLAN v0.19.0 — Non-Canonical Miniscript Descriptor Support in `mnemonic bundle`

**Toolkit target:** `mnemonic-toolkit-v0.19.0`  **GUI target:** `mnemonic-gui-v0.8.0` (lockstep)
**Status (plan-mode draft):** Phase-1 explore complete; opus R0-R3 converged V4 to 0C/0I on "strict explicit-required" Q1 lock. **V5 reverses Q1 per user direction 2026-05-16:** silent inference with `m/48'/<coin>'/<account>'/2'` default for non-canonical wsh/tr/sh-wsh wallets. R4 returned 2C/4I/3N (missed GUI projection consequence). **V6 folds R4 findings.**
**Predecessor:** `a27b8b1` (toolkit) + `1b14da4` (GUI). v0.18.1 + v0.7.2 closed all 5 v0.6.0-cycle FOLLOWUPs.

## §0 Context (why this cycle now)

The user wants the `mnemonic bundle` command to accept non-canonical miniscript descriptors — anything outside md-codec's `canonical_origin()` table — with phrase-bearing slots producing wire-correct bundles. Target invocation (timelock-inheritance example):

```
mnemonic bundle --network mainnet \
  --descriptor 'wsh(andor(pkh([<fp>/48h/0h/0h/2h]@0),after(12000000),
                          or_i(and_v(v:pkh([<fp>/48h/0h/0h/2h]@1),older(4032)),
                               and_v(v:pkh([<fp>/48h/0h/0h/2h]@2),older(32768)))))' \
  --language english \
  --slot '@0.phrase=…' --slot '@1.phrase=…' --slot '@2.phrase=…'
```

Reclassifies FOLLOWUP `miniscript-beyond-bip388` from `v1+` to `v0.19.0-feature` (toolkit `design/FOLLOWUPS.md:1175`). Bucket 5 (v1.0 readiness) updates: Buckets 1-5 fully closed after this cycle.

## §1 Locked design decisions (from plan-mode AskUserQuestion)

**Q1 — Per-placeholder origin source (REVERSED 2026-05-16 per user direction):** **Default inference with stderr info notice.** When a descriptor classifies as non-canonical (`md_codec::canonical_origin::canonical_origin(&tree).is_none()`) and any `@N` lacks an explicit origin path (no inline `[fp/path]@N`, no `--slot @N.path=`), the toolkit **silently assigns** the BIP-48 cosigner path `m/48'/<coin>'/<account>'/2'` per placeholder, where:

- `<coin>` = `0'` for mainnet, `1'` for testnet/signet/regtest (BIP-44 coin-type convention).
- `<account>` = `<--account-value>'` (defaults to `0'`; the CLI default for `--account`).

**Applies to all non-canonical wrappers**: `wsh(<ms>)`, `sh(wsh(<ms>))`, `tr(<key-or-NUMS>, <ms-tap-tree>)`. Multisig and singlesig (n=1) uniformly. The user-supplied origin (inline OR slot-path form) **overrides** the default per-`@N`. **Stderr info notice on emission** (matches `[[feedback-manual-gui-lockstep]]`-precedent advisory pattern): `info: non-canonical descriptor; defaulting origin path for @<N>...@<M> to m/48'/<coin>'/<account>'/2' (BIP-48 cosigner path). Override per-placeholder with [fp/path]@N or --slot @N.path=m/...` — printed once with the comma-separated list of `@N` indices that received the default.

**Existing toolkit refusal `DESCRIPTOR_WITH_NONZERO_ACCOUNT` at `bundle.rs:200-206` is RELAXED for non-canonical descriptors only.** Canonical-descriptor mode preserves the existing refusal (canonical_origin's table already supplies the per-shape default; user-supplied `--account != 0` there is redundant and refused as before). Non-canonical descriptor mode now consumes `--account N` as the account component of the default path.

**Q1.a (V5) — Sensible defaults for ambiguities the user's terse direction left open** (flagged for R4 architect validation):

1. **Network parameterization**: yes — `<coin>` follows BIP-44 convention (`0'`/`1'`); user's `--network mainnet` example yields `m/48'/0'/0'/2'` exactly. Alternative: hardcode `0'` regardless of `--network` (would make testnet bundles unusable; rejected).
2. **Account parameterization**: yes — `<account>` consumes `--account N`. The user's example `--account 0` resolves to `m/48'/0'/0'/2'`. Alternative: hardcode `0'` regardless (would block `--account N` for non-canonical mode; rejected because the user's CLI offers `--account`).
3. **Singlesig non-canonical (n=1)**: same default `m/48'/<coin>'/<account>'/2'`. Alternatives considered: BIP-84 `m/84'/<coin>'/<account>'` (would force users to know wsh-pk-singlesig is conceptually-singlesig); refuse (would break the user's directive). Uniformity wins.
4. **Stderr behavior**: info notice (not silent) — same Option-A pattern as v0.7.1 contiguity warning; user can pipe stderr to /dev/null if they want truly silent. Truly-silent inference is rejected because it makes the per-`@N` path invisible at audit time.
5. **GUI behavior**: when the descriptor classifies non-canonical AND any slot has no path subkey AND no inline `[fp/path]@N`, render the default path as **placeholder text** ("`m/48'/0'/0'/2'` (default)") in the slot editor's path field; plus a one-line info banner adjacent to the slot grid (same render-site as the contiguity warning). Q4 (Option-A inline conditional.rs) is preserved.

**Q2 — tr() wrapping:** **Accept `tr(NUMS, <ms>)` sentinel.** The literal token `NUMS` (case-sensitive, no quotes) is substituted with the BIP-341 unspendable hex `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` *before* the descriptor reaches rust-miniscript. `tr(@N, <ms>)` with a real placeholder also accepted. Bare `tr(<ms>)` refused with friendly error pointing to both forms. Result: bech32m P2TR addresses with script-path-only spending (intentional, surfaced in stderr notice on bundle emission).

**Q3 — Subset scope:** **Trust rust-miniscript.** Toolkit gates are (a) wrapper class ∈ {wsh, sh-wsh, tr}, (b) per-@N origin coverage, (c) §6.6.b slot-grammar legal-set extension. Any fragment rust-miniscript parses in the wrapper context is accepted; refused fragments are documented as "rust-miniscript's standardness gate." Test corpus exercises the user's `andor` + 2-3 representative shapes (`thresh`, `multi_a`, multi-leaf tap-tree).

**Q4 — GUI vocabulary:** **Option-A inline `conditional.rs::bundle()` rule.** Manual extension to the existing helper pattern (`template_slot_count_warning` at `conditional.rs:189`, `detect_slot_index_gaps` at `slot_editor.rs:161`). No new SPEC §6.10 Predicate/Effect vocabulary; no schema-version bump. Drift gate `gui_schema_conditional_drift.rs` covers it via test cells.

## §2 Architectural strategy (file inventory)

**Corrected understanding from R0 (C1 fold).** md-codec `TlvSection` has two distinct fields:

- **`tlv.use_site_path_overrides`** — child-derivation suffix per `@N` (multipath alts `/<0;1>/*` + wildcard hardening). Already populated by `parse_descriptor.rs:734` from `resolve_placeholders`. NOT the origin-path field.
- **`tlv.origin_path_overrides`** — origin path from master per `@N`. NEVER populated by the toolkit today. md-codec's `validate_explicit_origin_required` (`validate.rs:182-207`) consults this OR falls back to `d.path_decl.paths` (Shared/Divergent OriginPaths) — confirmed by md-codec test `validate_explicit_origin_required_passes_with_populated_shared_path_decl` (`validate.rs:588-599`).

**Implication:** inline `[fp/path]@N` in a non-canonical wsh-miniscript already passes md-codec wire validation today via `path_decl` — `resolve_placeholders` populates `path_decl.paths` (Shared or Divergent) at `parse_descriptor.rs:202-211` from inline annotations. The toolkit's `is_legal_set` rejection at `slot_input.rs:266-273` (secret + watch-only mix) does NOT fire for inline-annotation cases (the annotation is internal to the descriptor string, not a separate `--slot path=` flag). The legal-set gate only fires when the user explicitly types `--slot @N.phrase=... --slot @N.path=...`. **So `--slot @N.path=` is the only new plumbing required for the user's command to work; inline form requires zero toolkit code change beyond formal SPEC patch + manual mirror.**

### Toolkit changes

| File | Change | Lines (rough) |
|---|---|---|
| `crates/mnemonic-toolkit/src/slot_input.rs:293-310` | Extend `is_legal_set()` matrix with `[Phrase, Path]` + `[Phrase, Fingerprint, Path]`. Apply unconditionally (no context parameter — see §2.1 below for ordering settlement). The toolkit's downstream binding code already handles phrase + path correctly via `path_decl` propagation; the legal-set check is the only barrier. | ~10 LOC + tests |
| `crates/mnemonic-toolkit/src/slot_input.rs:225-287` | `validate_slot_set` unchanged — the legal-set arms cover the new pairs. The "non-canonical only" gating happens later (§2.1). | 0 LOC |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs` between `validate_slot_set` (line 247) and descriptor parse | New post-parse re-validation step: after `parse_descriptor` returns, if `canonical_origin(&desc.tree).is_some()` (canonical) AND any slot's subkey set contains `Path` with a `Phrase` peer → refuse with SPEC §6.6 row 4 (existing). When non-canonical → no additional check needed. This is the "split structural/grammar validation" fork settled per R0 I1 option (ii). | ~15 LOC |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs` (`bundle_run_unified_descriptor`, around lines 1019-1030 watch-only path branch + 960-997 phrase branch) | **Post-parse path resolution with default fallback (V5 Q1-reversal fold):** after `parse_descriptor` returns, for each `@N`, resolve the effective origin path in priority order: (1) user-supplied `--slot @N.path=` if present; (2) inline `[fp/path]@N` if `parse_descriptor`'s `path_decl` populated it; (3) **default `m/48'/<coin>'/<account>'/2'`** if `canonical_origin(&tree).is_none()` AND neither (1) nor (2) supplied a path. Mutation site: `mddesc.path_decl.paths = Divergent(vec)` where `vec[i]` is the resolved path for `@i`. **Invariant (R2 #4 fold):** `vec.len() == n`. The default may apply to some `@N` and not others (mixed: e.g., `@0` has inline, `@1`/`@2` use default); per-slot, not per-bundle. Existing phrase-branch code reads `bundle.rs:950-954` (`match &resolved_placeholders.path_decl.paths { Shared(op) => …; Divergent(v) => &v[idx] }`) unchanged. **Per-`@N` path-comparison helper (R2 #3 fold):** `compare_slot_path_vs_inline_path(idx, slot_inputs, inline_path) -> Result<(), ToolkitError>` called BEFORE the mutation for each slot that has BOTH `--slot @N.path=` AND inline; refuses row 19 on disagreement. **Stderr info notice (Q1.a #4 fold):** when one or more `@N` received the default, print `info: non-canonical descriptor; defaulting origin path for @{idx-list} to m/48'/<coin>'/<account>'/2' (BIP-48 cosigner path). Override per-placeholder with [fp/path]@N or --slot @N.path=m/...` to stderr before bundle emission. No fingerprint injection (rows 17/18 unchanged). | ~60 LOC |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs:200-206` (`DESCRIPTOR_WITH_NONZERO_ACCOUNT` guard) | **Relax for non-canonical descriptors (V5 Q1-reversal fold):** condition becomes `if canonical_origin(&tree).is_some() && args.account != 0 && descriptor_mode { refuse }`. For non-canonical descriptor mode, `--account N` is now meaningful (consumed by default-path inference) and not refused. Canonical descriptor mode still refuses `--account != 0` (per-shape canonical default already supplies the path; user-supplied account is redundant). This requires the canonicity classification to happen BEFORE this guard fires — restructure: move guard from the pre-`bundle_run_unified` site to inside `bundle_run_unified_descriptor` post-`parse_descriptor`. | ~10 LOC + restructure |
| `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:483-509` (R4 C1 fold — GUI projection rule for `--account` pin) | **The existing `gui-schema` projection emits a `ConditionalRule` pinning `--account` to 0 whenever `--descriptor` is present (Predicate: `FlagPresent{--descriptor}`, Effect: `PinValue(0)`).** Under V5, this rule over-fires for non-canonical descriptors — silently coercing user-typed `--account 5` to `0` before the toolkit consumes it, defeating Q1.a #2. **V6 resolution (Q4 Option-A-consistent):** keep the toolkit-side `gui-schema` rule as-is (still encoded; drift-test at `cli_gui_schema_v3_extensions.rs:46-86` keeps asserting its presence on `flag_present --descriptor`). Add a GUI-side Option-A override in `mnemonic-gui/src/form/conditional.rs::bundle()` (see GUI changes table below) that *removes* the pin when descriptor is non-canonical. The drift-gate cell needs a documented exception (or refactor to assert "schema rule present" XOR "GUI override applies under condition X"). Update §6.10.7 mapping table line for `DESCRIPTOR_WITH_NONZERO_ACCOUNT` to note "Option-A enhanced (GUI canonicity override)." Schema vocabulary unchanged — Q4 lock preserved (no new Predicate). | ~0 toolkit LOC (rule preserved); ~15 GUI LOC override; +1 §6.10.7 mapping note |
| `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (new flag `--classify-descriptor <STR>`) | **R2 #2 fold:** new diagnostic flag on `mnemonic gui-schema` that takes a descriptor string and prints `canonical` or `non-canonical` to stdout. Implementation: call `parse_descriptor(input, &[], &[])?` to get `MdDescriptor.tree`, then `md_codec::canonical_origin::canonical_origin(&tree).is_some() ? "canonical" : "non-canonical"`. Drives the drift-gate kittest cell in mnemonic-gui. Exit 0 on success, exit 2 on descriptor-parse failure (so tests can distinguish "non-canonical" from "unparseable"). | ~20 LOC + 1 manual chapter row |
| `crates/mnemonic-toolkit/src/parse_descriptor.rs` (new file-level fn `substitute_nums_sentinel`) | NUMS sentinel pre-substitution: scan input for literal `NUMS` tokens inside `tr(...)` first-arg position; replace with `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`. Returns a `String` (owned, substituted form). Bare `tr(<non-key>)` detection happens implicitly via rust-miniscript's `MsDescriptor::from_str` error at line 709 — toolkit's `friendly.rs` mapper translates that to row 16's hint. | ~25 LOC + tests |
| `crates/mnemonic-toolkit/src/parse_descriptor.rs:696` (first executable line of `parse_descriptor`) | Explicit rebinding: `let input = substitute_nums_sentinel(input)?;` shadows the parameter for the rest of the function. ALL downstream callers in the function body (`lex_placeholders(input)` at 696, `substitute_synthetic(input, ctx)` at 708) consume the rebound form. R1 C-R1-3 fold: the dataflow MUST cover the `substitute_synthetic` call site too, not just `lex_placeholders`. | ~3 LOC (1-line let-binding + comment) |
| `crates/mnemonic-toolkit/src/parse_descriptor.rs:984-997 + 1080-1100` | Fingerprint cross-check (rows 17 + 18): when phrase slot has `--slot @N.fingerprint=` AND inline `[fp/...]@N`, refuse on mismatch. When phrase-derived master fingerprint disagrees with inline `[fp/...]@N`, refuse. Extends existing line 984-989 cross-check logic. | ~25 LOC |
| `crates/mnemonic-toolkit/src/friendly.rs:220-260` | New error variants + friendly-text mappings: `NonCanonicalMissingOrigin`, `TrInvalidBare`, `PhrasePathFingerprintMismatch` (rows 15-18). | ~40 LOC |

**No changes to:** `synthesize.rs`, `verify_bundle.rs` (round-trip is automatic — synthetic inline annotations are part of the descriptor string, so verify-bundle re-runs the same pipeline), `derive.rs`, `slot_input.rs::validate_slot_set` body.

### §2.1 `validate_slot_set` ordering (R0 I1 lock)

**Option (ii) locked:** split `validate_slot_set` into structural and grammar checks. Structural check (contiguity, duplicate-subkey, conflict-type-mix) runs at current early site `bundle.rs:247`. Grammar check (legal-set arms — which now permit `[Phrase, Path]` and `[Phrase, Fingerprint, Path]`) also runs at early site BUT in canonical/template mode the new arms produce a deferred refusal in `bundle_run_unified` / `bundle_run_unified_template` — implemented as: after `parse_descriptor` returns, if `canonical_origin(&tree).is_some()` AND any slot has the new pairs, refuse with SPEC row 4 stderr text (existing). The slot grammar accepts the pairs structurally; the canonicity verdict drives the actual rejection. This preserves the existing "all structural validation before any binding" discipline (per `bundle.rs:247` comment) and adds one post-parse semantic check.

### md-codec — NO CHANGE

`md_codec::canonical_origin::canonical_origin` is public (verified: `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/canonical_origin.rs:45`); `validate_explicit_origin_required` already passes for `path_decl`-only origin attestation (verified: same file `validate.rs:194-204` + test 588-599). No md-codec wire-format change. No md-codec version bump.

### GUI changes (R1 C-R1-2 fold: classifier is GUI-side string-level heuristic)

| File | Change | Lines (rough) |
|---|---|---|
| `mnemonic-gui/src/form/conditional.rs` (new fn) | `classify_descriptor_canonicity(desc: &str) -> Canonicity` — **wrapper-form regex match** (R2 #5 fold) on 5 patterns mirroring md-codec's `canonical_origin.rs` table: `^pkh\(\[?…\]?@\d+\)$`, `^wpkh\(\[?…\]?@\d+\)$`, `^tr\(\[?…\]?@\d+\)$` (no second arg — bare tr-keypath), `^wsh\((multi\|sortedmulti)\(`, `^sh\(wsh\((multi\|sortedmulti)\(`. Anything else → `Canonicity::NonCanonical`. No general "strip inline `[fp/path]` then match" preprocessing — the wrapper-form regexes are agnostic about what's inside (they only check the wrapper shape via prefix anchors). Implementation note: each regex is compiled once at module load. R1 C-R1-2 alternatives (a) "port full parser" (~2000 LOC, impractical) and (b) "shell out subcommand at render time" (latency unacceptable for real-time banner) were rejected. Drift-gate (next row) keeps the heuristic safe against md-codec table evolution. | ~50 LOC |
| `mnemonic-gui/tests/canonicity_drift.rs` (new) | Drift-gate kittest cell. Corpus of ~20 descriptor strings (covering all 5 canonical shapes + 6-8 non-canonical shapes including the user's `wsh(andor(...))` + `tr(NUMS, ...)`) with expected verdicts. The drift-gate cell runs the GUI classifier AND **shells out to `mnemonic gui-schema --classify-descriptor <str>`** (the new toolkit subcommand) on each fixture; asserts the canonicity verdict matches. R2 #4 fold: pinned subcommand mechanism replaces V3's earlier "(or similar)" hand-wave. **Cross-repo test invariant**: any new canonical shape added to md-codec's `canonical_origin.rs` requires a paired GUI test entry. Documented in CLAUDE.md mirror invariant block. | ~80 LOC |
| `mnemonic-gui/src/form/conditional.rs:105-170` (`bundle`) | New helper `is_descriptor_non_canonical(state: &FormState) -> bool` — returns true when `state.has_value("--descriptor") AND classify_descriptor_canonicity(...) == Canonicity::NonCanonical`. Used to gate the new warning banner. NOT used to alter slot subkey dropdown — that stays unfiltered (CLI remains authoritative). | ~20 LOC |
| `mnemonic-gui/src/form/conditional.rs` (new fn, V5 reframe) | `descriptor_non_canonical_default_path_notice(state) -> Option<String>` mirroring `template_slot_count_warning`. Returns informational text when descriptor is non-canonical AND ≥1 `@N` has no path source (neither inline nor slot). Text: `info: non-canonical descriptor; @{idx-list} will use default path m/48'/<coin>'/<account>'/2' (BIP-48). Override by adding [fp/path]@N to descriptor or filling in slot path.` Banner-style (info color, not warning red). | ~30 LOC |
| `mnemonic-gui/src/form/slot_editor.rs:191` (path field placeholder) | When the row's subkey is `Path` AND the form's descriptor is non-canonical AND the row's `value` is empty, show placeholder text `m/48'/<coin>'/<account>'/2' (default)` in the text input — same egui `hint_text` pattern used elsewhere. Computed from **the user-typed widget value of `--account`**, NOT the pin-coerced emission value (R4 I4 fold). Reads `--network` + `--account` from form state via `state.text_value`/`state.has_value`; canonicity verdict via the new `classify_descriptor_canonicity` helper. | ~15 LOC |
| `mnemonic-gui/src/form/conditional.rs:144-149` (`--account → PinValue(0)` override — R4 C1 fold) | **Wrap the existing `vis.push(("--account", PinValue(0)))` push at lines 144-149 in a canonicity gate.** Before pushing, classify the descriptor via `classify_descriptor_canonicity(state.text_value("--descriptor"))`. Only push the `PinValue(0)` when verdict is `Canonical`. For `NonCanonical`, the existing widget value flows through to argv emission unmodified. The gui-schema-emitted rule still exists (toolkit emits it unconditionally per V6 R4 C1 fold above), but the hand-coded `bundle()` Option-A function deviates — drift-gate cell at `cli_gui_schema_v3_extensions.rs:46-86` (or its mirror in mnemonic-gui tests) needs a documented exception that the gui-schema-vs-conditional drift here is *intentional* for non-canonical descriptors. | ~15 LOC + drift-test annotation |
| `mnemonic-gui/src/main.rs` (or wherever bundle warnings render) | Wire the new warning banner adjacent to slot grid. Same render-site precedent as v0.7.1 contiguity + v0.7.2 template/slot-count warnings. | ~10 LOC |

### Manual changes

| File | Change |
|---|---|
| `mnemonic-toolkit/docs/manual/src/40-cli-reference/41-mnemonic.md` | New subsection "Non-Canonical Descriptor Mode" under `mnemonic bundle`: defines canonicity (cites md-codec classifier), enumerates the two origin-path forms (inline + `--slot @N.path=`), shows the user's `wsh(andor(...))` example + `tr(NUMS, ...)` example, lists refusal cases. |
| Optional: `docs/manual/src/00-overview.md` | One-line cross-reference to the new subsection. |

**Mirror invariant gate:** `docs/manual/tests/lint.sh` already enforces flag-coverage; no new toolkit flag is added (Q1 reuses `--slot @N.path=`), so the lint passes automatically.

## §3 SPEC patches (`design/SPEC_mnemonic_toolkit_v0_5.md`)

### §6.6.b — extend validity matrix (line 225-237)

Add two new rows, gated on a NEW phrase "*when the descriptor is non-canonical per `md_codec::canonical_origin`*":

> - `{phrase, path}` (non-canonical descriptor mode only) → secret-bearing with explicit per-`@N` origin path
> - `{phrase, fingerprint, path}` (non-canonical descriptor mode only) → secret-bearing with explicit origin + fingerprint attestation; toolkit cross-checks supplied fingerprint against phrase-derived master fingerprint

Add the inverse refusal: "any `{phrase, path}` or `{phrase, fingerprint, path}` in canonical-descriptor or template mode → exit 2 row 4 (existing conflict path)."

### §6.6 mode-violation ladder — new rows

Insert two rows after row 14:

> | 15 | **REMOVED in V5 Q1-reversal.** Non-canonical descriptor with bare `@N` no longer refuses — default `m/48'/<coin>'/<account>'/2'` is silently applied with stderr info notice. Row number reserved (not reused) to preserve §6.6 ladder index stability across cycles. | — | (no error; advisory only) |
> | 16 | Bare `tr(<miniscript>)` with no internal key | 2 | `error: tr() requires an internal key. For script-path-only spending use tr(NUMS, <ms>); for full taproot use tr(@<index>, <ms>) with a slot binding for the internal key.` |
> | 17 | Phrase slot with `--slot @N.fingerprint=` AND inline `[<fp>/path]@N` in descriptor AND values disagree | 2 | `error: slot @{N} fingerprint mismatch: --slot says {fp_slot}, descriptor inline [{fp_descriptor}/...] disagrees; supply consistent values.` |
> | 18 | Phrase slot's derived master fingerprint disagrees with inline `[<fp>/path]@N` | 2 | `error: slot @{N} phrase-derived fingerprint {fp_derived} does not match descriptor inline [{fp_inline}/...]; verify the phrase or correct the descriptor.` |
> | 19 | Phrase slot with `--slot @N.path=` AND inline `[<fp>/path]@N` in descriptor AND paths disagree | 2 | `error: slot @{N} path mismatch: --slot says {path_slot}, descriptor inline [.../{path_descriptor}] disagrees; supply consistent values or remove one source.` |

**Parsimony tradeoff (R1 I-R1-2):** rows 17/18/19 are structurally similar ("per-`@N` multi-source consistency mismatch with named attribute"). They could collapse to a single row with an attribute parameter (`fingerprint` | `path`). V3 keeps them split for SPEC clarity (distinct exit-stage attribution; row 17 = slot-vs-inline-fp; row 18 = derived-vs-inline-fp; row 19 = slot-vs-inline-path), matching the v0.5 row enumeration style. If R2 prefers collapse, refactor at that point; the byte-exact stderr texts can stay distinct under one logical rule.

**Wire-format note (R1 I-R1-4):** `--slot @N.path=` is a CLI input-aliasing convenience, NOT a separate wire-format mode. The toolkit's post-parse mutation of `MdDescriptor.path_decl` (per §2 row 4) means the wire bytes only carry the resulting paths (inside `path_decl`, or the equivalent populated `tlv.origin_path_overrides` if Phase 4 R0 lands on that route). verify-bundle re-parses md1 and reads paths from the descriptor structure — it does NOT need the original `--slot @N.path=` input form to reproduce the verification.

**Per-row firing stage (R0 I3 fold, V5 Q1-reversal updated):**

- **Row 15**: REMOVED in V5; not a refusal class. Replaced by stderr info notice on bundle emission when default path inference fires.
- **Row 16** (bare-tr no-key): fires pre-parse via NUMS-sentinel detection's bare-tr scan, OR (fallback) post-parse via rust-miniscript's error string captured by `friendly.rs`.
- **Row 17** (slot-fingerprint vs inline-fingerprint mismatch): fires post-parse, pre-binding. Both values are known after lex+resolve.
- **Row 18** (phrase-derived vs inline-fingerprint mismatch): fires during phrase binding (post-derive), in `bundle_run_unified_descriptor`'s phrase branch.
- **Row 19** (slot-path vs inline-path mismatch): fires post-parse, pre-binding via `compare_slot_path_vs_inline_path`.

Rows 13/14 continue firing post-binding as before (BIP-388 distinct-key + annotation consistency).

### §6.9 byte-exact error texts

Add stderr literals for rows 15-18 as v0.7-era amendment block. Pin in integration tests under `tests/error_messages/`.

### §6.10.7 gui_projection mapping table — additions

| Subcmd | SPEC row | toolkit const | Predicate | Effect | Status |
|---|---|---|---|---|---|
| bundle | §6.6 row 15 | (n/a — GUI-internal warning) | (GUI-internal: `is_descriptor_non_canonical(state) AND any @N lacks origin`) | (n/a — GUI-internal `descriptor_non_canonical_missing_origin_warning` helper) | (GUI-internal warning) — Option-A pattern (no schema encoding) — R1 N-R1-2 fold |
| bundle | §6.6 row 16 | (n/a — pre-parse refusal) | (n/a — CLI-only; GUI's descriptor text field accepts any string) | (n/a) | NOT-ENCODED (CLI-rejection-sufficient) |
| bundle | §6.6 rows 17/18 | (n/a — CLI-only) | (similar to existing rows 13/14 wontfix rationale) | (n/a) | NOT-ENCODED (CLI-rejection-sufficient) |

Matches the v0.7.1/v0.7.2 Option-A precedent. Cross-citation discipline preserved.

### NEW §4.12 — Non-canonical descriptor mode (formal definition, V5)

Brief subsection (~40 lines) defining:

- "Canonical" = `md_codec::canonical_origin::canonical_origin(tree).is_some()` (lists the 5 canonical shapes verbatim from md-codec at `canonical_origin.rs:45-79`).
- "Non-canonical" = `canonical_origin(tree).is_none()`.
- **Default origin path inference (V5 Q1):** for non-canonical descriptors, when an `@N` has no inline `[fp/path]@N` AND no `--slot @N.path=`, the toolkit assigns `m/48'/<coin>'/<account>'/2'` (BIP-48 cosigner path) where `<coin>` and `<account>` derive from `--network` (mainnet→`0'`, testnet/signet/regtest→`1'`) and `--account` (defaults to `0'`). Applies uniformly to wsh, sh-wsh, and tr wrappers, and uniformly to multisig and singlesig non-canonical descriptors.
- Per-`@N` priority order for origin source: (1) `--slot @N.path=` > (2) inline `[fp/path]@N` > (3) default. Within (1)+(2), mismatch → refuse row 19. (3) only applies if neither (1) nor (2) supplied.
- Toolkit emits a stderr info notice listing the `@N` indices that received the default.
- Fingerprint sources (`--slot @N.fingerprint=`, inline `[fp/...]@N`, phrase-derived master fingerprint) cross-validated per rows 17/18.
- BIP-388 distinct-key invariant (§4.11.b) still applies — non-canonical descriptors with shared `@N` placeholders still require distinct `(xpub, path)` tuples post-resolution. **Note:** when all `@N` use the same default path AND all phrases differ, distinctness is satisfied via xpub-only divergence. When phrases match across `@N` AND all paths default (same), distinctness FAILS at row 13 (existing gate) — this is correct.

## §4 NUMS sentinel handling (§4.12.a)

The literal token `NUMS` is the only sentinel; no other case-folded variants. Substitution happens in `substitute_synthetic` at `parse_descriptor.rs:268-302` BEFORE rust-miniscript parses. The substituted descriptor is what gets walked, encoded, and round-tripped. md1 wire carries the literal NUMS hex (no sentinel in wire); verify-bundle re-parses the literal hex without sentinel awareness.

**Stderr notice on emission** (Option-A pattern, before bundle JSON/text emit):

```
notice: tr() with NUMS internal key — key-path spending is disabled by construction.
        Wallet is P2TR (bech32m addresses); spending proceeds via the tap-script path only.
        NUMS = 50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0 (BIP-341 H_point).
```

## §5 Phase decomposition + reviewer-loop expectations

Per `[[feedback-plan-artifact-mirror-project-convention]]` discipline; reviewer-loop iterated to **0 Critical / 0 Important** per phase boundary; opus per `[[feedback-opus-primary-review-agent]]`.

### Phase 1 — SPEC patches (no code)

Apply §3's SPEC additions. Single doc patch to `design/SPEC_mnemonic_toolkit_v0_5.md`. R0 verifies cross-citations + byte-exact-text format. Estimated 1 reviewer round.

### Phase 2 — Toolkit slot grammar extension (TDD)

Tests first: extend `slot_input.rs::tests` with cells exercising `[Phrase, Path]` + `[Phrase, Fingerprint, Path]` legal under non-canonical context, illegal otherwise. Then implement `LegalSetContext` enum + `is_legal_set` arm. Wire `is_non_canonical` helper.

**Verification:** new tests pass; existing 30+ binary tests stay green; `cargo nextest run -p mnemonic-toolkit` clean. R0+R1 expected.

### Phase 3 — Toolkit NUMS sentinel + tr() refusal

**Phase-3 R0 prerequisite (R0 C3 fold):** before writing any test cells, compile-check every proposed golden-corpus tr-fixture against rust-miniscript by running `MsDescriptor::<DescriptorPublicKey>::from_str(<fixture>)` in a throwaway test harness. If a fixture won't parse (e.g., BIP-342 tap-leaf miniscript context rejects `older(<N>)` when `<N>` has different semantics, or `v:pk` is wrong wrapper for tapscript), pin the working alternative in the plan-doc BEFORE writing the implementation. Captures the `[[feedback-architect-must-run-prose-commands]]` discipline.

Tests first (after parse-check): golden bundle for `tr(NUMS, and_v(v:pk([fp/86h/0h/0h]@0), after(12000000)))` IF parse-checked OK; refusal test for bare `tr(<ms>)`. Then implement `substitute_nums_sentinel` as a new helper called at `parse_descriptor.rs:691` (before `lex_placeholders`); emit stderr notice on bundle emission.

**Verification:** new tests pass; verify-bundle round-trips the tr(NUMS, ...) bundle byte-exact; bech32m P2TR address derivation post-cycle confirms (manual smoke per §9 verification). R0+R1+possibly R2 (taproot semantics are subtle; tap-leaf context fragment availability is the biggest unknown).

### Phase 4 — Toolkit full-pipeline wiring (TDD)

Tests first: golden bundle for the user's `wsh(andor(...))` with phrase slots + inline `[fp/path]@N`. Then golden bundle for the same with `--slot @N.path=` instead. Then mixed case. Then refusal cases (rows 15/17/18). Implement the phrase-slot path-source plumbing through `bundle_run_unified_descriptor` to `parse_descriptor`'s `use_site_path_overrides`.

**Verification:** golden bundles byte-exact-stable; BIP-388 distinct-key still passes for 3 distinct phrases → 3 distinct xpubs at the same path; verify-bundle round-trips. R0+R1+R2 expected (this is the meatiest phase). **R4 I1 fold:** Phase 4's refusal-test enumeration is "rows 17/18/19" (row 15 removed in V5; acceptance test for default-inference covered in §6 corpus item #6).

### Phase 5 — Manual mirror

Patch `41-mnemonic.md` with new subsection. Add the two worked examples from §1 + the refusal recipes. Run `make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...` per CLAUDE.md mirror invariant.

**Verification:** mirror lint passes; user can walk the example end-to-end. R0 expected; possibly R1 for prose quality.

### Phase 6 — GUI lockstep

Port descriptor-canonicity classifier (small AST recogniser, ~50 LOC mirroring md-codec's table verbatim — simpler than depending on toolkit subprocess). Add `is_descriptor_non_canonical` helper + `descriptor_non_canonical_missing_origin_warning` helper in `conditional.rs`. Wire warning banner adjacent to slot grid. Update kittest cells.

**Verification:** existing 240 GUI tests stay green; new cells exercise non-canonical-descriptor banner rendering + dismissal; argv assembler tests cover `--slot @N.path=` emission. R0+R1 expected.

### Phase 7 — End-of-cycle opus review + lockstep release

Single opus dispatch (`feature-dev:code-reviewer`, `model: "opus"`) reading the full toolkit + GUI diff against the locked SPEC patches. Filter for unfolded I (Important) findings per `[[feedback-r2-blocking-vs-cosmetic-gate]]`.

Release sequence (matches v0.18.1/v0.7.2 lockstep pattern):
1. Bump `Cargo.toml` versions in both repos.
2. Update `install.sh` self-pin (toolkit) + GUI pin (toolkit) + toolkit pin (GUI side, `pinned-upstream.toml`).
3. Tag `mnemonic-toolkit-v0.19.0` on toolkit master.
4. Tag `mnemonic-gui-v0.8.0` on GUI master.
5. Cross-repo install-pin-check CI gate (per `c3214e1`) verifies the self-pin is current — drift = release blocker.
6. GitHub releases live for both.
7. Update `design/FOLLOWUPS.md`: close `miniscript-beyond-bip388`; refile any deferred items (non-canonical-address-derivation, tr-multileaf-deep-cycles, etc.).

R0 review only; no further iteration unless findings surface.

## §6 Test corpus

**Golden bundles** (byte-exact-stable, pinned in `tests/golden/`):

1. **wsh-miniscript-inline-origin** — user's `wsh(andor(pkh([deadbeef/48h/0h/0h/2h]@0), after(12000000), or_i(and_v(v:pkh([deadbeef/48h/0h/0h/2h]@1), older(4032)), and_v(v:pkh([deadbeef/48h/0h/0h/2h]@2), older(32768)))))` with beef-phrases @0..@2. Inline origin form.
2. **wsh-miniscript-slot-path** — same descriptor but with bare `@0..@2` + `--slot @N.path=m/48'/0'/0'/2'`. Slot-path form.
3. **wsh-miniscript-mixed** — `@0` inline, `@1` and `@2` via slot-path.
4. **tr-NUMS-andor** — `tr(NUMS, and_v(v:pk([fp/86h/0h/0h]@0), after(12000000)))` (single tap-leaf for minimality). beef-phrase @0.
5. **tr-with-real-internal-key** — `tr([fp/86h/0h/0h]@0, and_v(v:pk([fp/86h/0h/0h']@1), after(12000000)))` (Q2 mentioned this is supported by the same sentinel handling; verifies real-`@N` internal key works alongside NUMS sentinel).

**Refusal corpus** (exit-code-pinned, stderr-byte-exact):

6. **non-canonical-default-path-applied** — `wsh(andor(pkh(@0), after(12000000), or_i(and_v(v:pkh(@1), older(4032)), and_v(v:pkh(@2), older(32768)))))` with 3 phrase slots, no inline `[fp/path]`, no `--slot path=`, `--account 0`, `--network mainnet`. **Expected: bundle emits successfully** with default `m/48'/0'/0'/2'` applied to @0/@1/@2 + stderr info notice listing `@0,@1,@2`. V5 Q1-reversal flips this from refusal (V4) to acceptance with default. This is the user's exact target invocation.
7. **bare-tr-no-key** — `tr(andor(pkh(@0), older(1000)))`. Exit 2 + row 16 stderr.
8. **fingerprint-mismatch-slot-vs-inline** — user supplies `--slot @0.fingerprint=cafebabe` AND descriptor has `[deadbeef/48h/...]@0`. Exit 2 + row 17 stderr.
9. **fingerprint-mismatch-phrase-derived** — phrase @0 derives master_fp `abcdef00`, descriptor inline `[deadbeef/48h/...]@0`. Exit 2 + row 18 stderr.
10. **canonical-with-phrase-path** — canonical `wpkh(@0)` template with `--slot @0.phrase=... --slot @0.path=m/...`. Exit 2 + row 4 stderr (the existing row 4 refusal still gates canonical-mode).

**BIP-388 distinct-key invariant test:**

11. **non-canonical-distinct-keys** — same beef-phrase across @0, @1, @2 (all derive identical xpubs at identical paths) → exit 2 + row 13 stderr (existing distinct-key gate still fires).

## §7 Cross-repo coordination

- **md-codec:** NO change. Toolkit imports `md_codec::canonical_origin::canonical_origin` (newly used in toolkit-side classifier). No version bump on `descriptor-mnemonic`.
- **md-cli:** No change. Precedent only (the `--key @i=XPUB` flag is a divergent grammar that toolkit's `--slot @N.<subkey>=<value>` deliberately doesn't mirror).
- **mnemonic-gui:** Lockstep release `v0.8.0`. Toolkit pin bump (`pinned-upstream.toml` toolkit version → `v0.19.0`); install.sh GUI pin bump (`v0.7.2` → `v0.8.0`).
- **mnemonic-secret / mnemonic-key:** No change. ms1/mk1 wire format unaffected.

## §8 Risks & FOLLOWUPs

**Inherent risks:**

- **R1 — Tap-leaf path-classifier ambiguity.** Tap-leaves can be deep miniscript; rust-miniscript may accept fragments that downstream wallets reject (Sparrow, Specter inconsistencies). **Mitigation:** Phase 4 + Phase 6 use rust-miniscript as the sole gate (per Q3); refused fragments are documented as upstream's responsibility. Test corpus pins user's example + 2-3 representative shapes; broader fragment-by-fragment compatibility deferred to a downstream-compatibility cycle.
- **R2 — Fingerprint cross-check edge cases.** Phrase + inline `[fp/path]@N` + `--slot @N.fingerprint=` 3-way disagreement matrix is non-trivial. **Mitigation:** Phase 4 enumerates all 3 cells (rows 17 + 18 + their interaction); reviewer-loop exercises each.
- **R3 — GUI classifier porting.** Mirroring md-codec's `canonical_origin` table GUI-side risks drift. **Mitigation:** test cell that compares GUI's classifier verdict against toolkit's via `mnemonic gui-schema --classify-descriptor` (new helper subcommand, ~20 LOC); drift-gate this in CI.
- **R4 — NUMS hex literal in md1.** Verify-bundle re-parses the literal hex without sentinel awareness — this is correct (sentinel is pure user-input UX). **Mitigation:** Phase 3 test corpus #4 exercises round-trip; no special-casing needed in `verify_bundle.rs`.
- **R5 (V6 R4 I2 fold) — Silent inference is a UX footgun for descriptor-mistype cases.** If a user types `wsh(addor(...))` (typo) thinking it's canonical-equivalent and the toolkit silently emits at `m/48'/0'/0'/2'` (because typo → non-canonical → default fires), the user may not notice the mis-emission until they try to recover. Precedent: v0.18.1 reverted disable_options for exactly this UX-flaw class (per `[[project-v0-18-1-v0-7-2-b1-bugfix-closed]]` — "reviewer-loop + plan-doc R0-R4 + opus end-of-cycle missed the UX flaw; user-running-the-feature is a distinct reviewer dimension"). **Mitigation (already in plan):** stderr info notice on every default-inference emission; GUI banner adjacent to slot grid; placeholder text in slot path field. **Additional Phase 7 reviewer mandate:** user-run-the-feature smoke against the user's exact wsh(andor(...)) + tr(NUMS, ...) examples BEFORE tagging — confirm the stderr notice is legible, the GUI banner renders, and the user has at least one obvious clue that a default was assumed. The cycle does NOT ship if Phase 7 smoke surfaces a perceptibility failure.

**New FOLLOWUPs to file at cycle close:**

- `tr-multileaf-non-canonical-deep-paths` — when tr() has ≥2 tap-leaves and each leaf has its own per-key origin: requires deeper integration with md-codec's TapTree branch encoding. Defer to v0.20+.
- `address-derivation-non-canonical` — extend `mnemonic address` (per FOLLOWUP `address-derivation-from-xpub-path` v0.7-resolution) to handle non-canonical descriptors. Defer until user demand surfaces.
- `descriptor-canonicity-classifier-shared` — possibly extract a tiny shared crate (`descriptor-canonicity`) that both toolkit and GUI depend on, to avoid the porting in §2/Phase 6. Defer until a third consumer materializes.

## §9 Verification (end-to-end)

After Phase 7 tagging:

1. `cd /scratch/code/shibboleth/mnemonic-toolkit && cargo nextest run --workspace` → all green.
2. `cd /scratch/code/shibboleth/mnemonic-gui && cargo nextest run --workspace` → 245+ tests green.
3. Run user's wsh example (V5: with `--account 0`, bare `@N`, no inline origin — exercising the default-path-inference path): bundle emits with stderr info notice → `verify-bundle` round-trip → cards reproduce byte-exact. Re-run with explicit inline `[fp/path]@N` form → same wire output, no info notice.
4. Run user's `tr(NUMS, ...)` example: bundle emits with stderr notice → cards reproduce.
5. Run all 6 refusal cases: exit codes + stderr texts byte-exact (per §6 refusal corpus items 6-10 + the row 19 path-mismatch case).
6. `make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...` → clean.
7. CI on both repos: tag firings green; install-pin-check passes.

**Additional verification items added at R1 I-R1-3 fold:**

8. md1 wire round-trip carries the NUMS pubkey bytes (R2 #6 fold): decode the emitted md1 → inspect `tlv.pubkeys[<NUMS-internal-key-index>]` → assert bytes 32..64 of the 65-byte payload equal `0x50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (the 32-byte X-only key form). The literal `NUMS` token does NOT appear on the wire — it's resolved to the pubkey bytes during pre-parse substitution. Re-encoding produces byte-identical bytes.
9. GUI warning banner kittest cell: render a form with descriptor=`wsh(andor(pkh(@0),...))` (non-canonical, no per-`@N` origin) + 3 phrase slots → assert banner text matches `descriptor_non_canonical_missing_origin_warning` output; assert banner dismisses when user adds inline `[fp/path]` annotations to the descriptor.
10. Byte-exact stderr pin tests for SPEC §6.6 rows 15/16/17/18/19 — each row exercises its trigger condition and asserts the stderr text matches the SPEC §6.9 amendment verbatim. These are pre-existing test patterns at `tests/error_messages/`; cycle adds 5 new test cells.

## §10 Reviewer-loop sequencing summary

| Phase | Reviewer | Expected rounds | Convergence target |
|---|---|---|---|
| 1 (SPEC) | opus | R0+R1 | 0C/0I |
| 2 (slot grammar) | opus | R0+R1 | 0C/0I |
| 3 (NUMS + tr refusal) | opus | R0+R1+R2 | 0C/0I |
| 4 (full pipeline) | opus | R0+R1+R2 | 0C/0I |
| 5 (manual mirror) | opus | R0 (maybe R1) | 0C/0I |
| 6 (GUI lockstep) | opus | R0+R1 | 0C/0I |
| 7 (end-of-cycle) | opus | R0 | 0C/0I |

**Sizing (R0 I2 fold, post-C1-correction).** With the corrected understanding that inline `[fp/path]@N` already passes md-codec wire-validation via `path_decl`, Phase 4 is smaller than opus's revised estimate but larger than the original because the fingerprint cross-validation matrix (rows 17/18) is a 3-source-of-truth comparison (phrase-derived fp ↔ `--slot fingerprint=` ↔ inline `[fp/...]`):

- Toolkit code: ~200-250 LOC (slot_input 10 + bundle 100 incl. default-inference + comparison helper + guard restructure + parse_descriptor 30 + friendly 30 + gui_schema --classify-descriptor 20 + stderr-notice helper 15) — V6 R4 I3 fold adjusts upward from V4's 170-200 estimate.
- Toolkit tests: ~500-700 LOC (5 golden bundles + ~5 refusal cases — row 15 removed in V5; default-inference acceptance test #6 + GUI schema drift-test churn at `cli_gui_schema_v3_extensions.rs:46-86` and `cli_gui_schema_conditional_rules.rs:102` per R4 C2 fold)
- GUI code: ~80-100 LOC (classifier port + warning helper + slot_editor pass-through)
- GUI tests: ~200-300 LOC (kittest cells for non-canonical-banner, argv_assembler new pairs, conditional drift gate cells)
- SPEC: ~100 LOC (matrix amendments + 4 new row entries + §4.12 + §6.10.7 patches)
- Manual: ~190 LOC (new subsection + 2 worked examples + refusal recipes + 1 row for `gui-schema --classify-descriptor` per R2 #2 fold)

Cycle wall-clock: 2-3 weeks. Opus dispatches: 14-18 (revised up from 12-16 to account for Phase 3's R0 fixture parse-check + Phase 4's fingerprint matrix R2 possibility + Phase 6 R3 if GUI classifier port doesn't converge cleanly).

**Escalation gate:** if Phase 4 cannot converge by R3 (i.e., R0/R1/R2/R3 all return Critical findings), pause execution and split the cycle into v0.19.0 (inline-form-only) + v0.20.0 (slot-path-form + fingerprint cross-validation). The inline form is the smaller scope and already wire-validated; the slot-path form is the larger scope with the fingerprint matrix.

## §11 Critical files (file:line index, for execution session entry)

- `crates/mnemonic-toolkit/src/slot_input.rs:60-70` — `is_secret_bearing` / `is_watch_only` (taxonomy)
- `crates/mnemonic-toolkit/src/slot_input.rs:225-310` — `validate_slot_set` + `is_legal_set` (extension site)
- `crates/mnemonic-toolkit/src/parse_descriptor.rs:268-302` — `substitute_synthetic` (NUMS site)
- `crates/mnemonic-toolkit/src/parse_descriptor.rs:691-744` — `parse_descriptor` pipeline (`use_site_path_overrides` is already populated by `resolve_placeholders`)
- `crates/mnemonic-toolkit/src/cmd/bundle.rs:118-130` — `mode_text` byte-exact consts (extension site for rows 15-18)
- `crates/mnemonic-toolkit/src/cmd/bundle.rs:892-1161` — `bundle_run_unified_descriptor` (phrase-slot path-source wiring site)
- `crates/mnemonic-toolkit/src/cmd/bundle.rs:200-206` — `DESCRIPTOR_WITH_NONZERO_ACCOUNT` gate (V6 R4 N2 fold: **restructured in V5; moves into `bundle_run_unified_descriptor` post-canonicity-check** so canonical descriptors still refuse `--account != 0` while non-canonical descriptors consume it for default-path inference)
- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:483-509` — existing GUI projection rule pinning `--account → 0` on `--descriptor` present (V6 R4 C1 fold: rule preserved toolkit-side; GUI Option-A overrides for non-canonical descriptors; drift-gate cell needs documented exception)
- `mnemonic-gui/src/form/conditional.rs:144-149` — current `--account → PinValue(0)` push (V6 R4 C1 fold: wrap in canonicity gate)
- `descriptor-mnemonic/crates/md-codec/src/canonical_origin.rs:45` — `canonical_origin()` public function (toolkit imports)
- `descriptor-mnemonic/crates/md-codec/src/validate.rs:182` — `validate_explicit_origin_required` (already enforces the wire-level invariant)
- `mnemonic-gui/src/form/conditional.rs:105-170` — `bundle()` (Option-A extension site)
- `mnemonic-gui/src/form/conditional.rs:189` — `template_slot_count_warning` (Option-A precedent for `descriptor_non_canonical_missing_origin_warning`)
- `mnemonic-gui/src/form/slot_editor.rs:147-168` — `detect_slot_index_gaps` (Option-A precedent)
- `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md:189-237` — §6.6 + §6.6.b (patch sites)
- `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md:284-450` — §6.10 (mapping-table extension site)
- `mnemonic-toolkit/design/FOLLOWUPS.md:1175-1182` — `miniscript-beyond-bip388` (reclassification site)

## §12 R0 outcomes + remaining open questions (for R1)

### R0 findings resolved in V2

- **C1** (TLV name conflation): folded — §2 now distinguishes `use_site_path_overrides` from `origin_path_overrides`; corrects "already works today" to specify the `path_decl` route used by inline annotations.
- **C2** (no current pathway): partially WRONG per source ground truth (`validate.rs:194-204` + test 588-599 confirm `path_decl` is a valid origin source). Inline form already works end-to-end today modulo `is_legal_set`. Only `--slot @N.path=` needs new plumbing.
- **C3** (tap-leaf parse-check): folded — Phase 3 now mandates fixture parse-check before test cells written.
- **I1** (validate_slot_set ordering): locked — option (ii) split structural/grammar; documented at §2.1.
- **I2** (sizing): revised — see §10 + escalation gate.
- **I3** (row firing stages): folded — §3's "Rows 13/14/15-18 fire post-binding" replaced with per-row stage.
- **I4** (Concern F + D): verified, no plan change.
- **I5** (NUMS helper naming): folded — `substitute_nums_sentinel` is named and ordered explicitly at `parse_descriptor.rs:691` before `lex_placeholders`.
- **N1-N4**: folded into C1's correction (cite drift) or acknowledged as accurate.

### R1 findings resolved in V3

- **C-R1-1** (synthetic-injection chicken-and-egg fp): folded — §2 row 4 rewritten to **post-parse mutation of `MdDescriptor.path_decl`**. No string-injection, no synthetic fp problem, existing binding code reads from the mutated structure unchanged.
- **C-R1-2** (GUI classifier infeasible): folded — §2 GUI-changes locked to **string-level heuristic** (~50 LOC matching md-codec's 5 canonical shapes) + drift-gate kittest cell. Options (a) port-full-parser and (b) shell-out rejected with reasons.
- **C-R1-3** (NUMS dataflow): folded — §2 row 6 now explicitly states `let input = substitute_nums_sentinel(input)?;` rebinding so the substituted form flows through BOTH `lex_placeholders` AND `substitute_synthetic`.
- **I-R1-1** (cite drift §2 vs §11): folded — call site pinned to `parse_descriptor.rs:696` (first executable line).
- **I-R1-2** (rows 17/18/19 parsimony): folded — explicit tradeoff statement added after row 19; V3 keeps them split for clarity.
- **I-R1-3** (§9 verification gaps): folded — added items 8/9/10 covering NUMS hex round-trip, GUI banner kittest, and rows 15-19 byte-exact stderr pin tests.
- **I-R1-4** (verify-bundle round-trip semantics): folded — wire-format note added after row 19 stating `--slot @N.path=` is CLI input convenience only.
- **N-R1-1** (row 18 prose): folded — row 18 description in §3 references existing `bundle.rs:984-989` site explicitly.
- **N-R1-2** (§6.10.7 status text): folded — "ENCODED v3" replaced with "(GUI-internal warning) — Option-A pattern (no schema encoding)".

### R2 findings resolved in V4

- **R2 #1** (citation drift `bundle.rs:1094` → `:950-954`): folded — §2 row 4 corrected.
- **R2 #2** (drift-gate mechanism unpinned): folded — new toolkit-changes row added for `mnemonic gui-schema --classify-descriptor` subcommand (~20 LOC + 1 manual chapter row); §2 GUI-drift-cell row updated to shell out to this subcommand. §10 LOC totals adjusted: toolkit 160 → ~180; manual +~40.
- **R2 #3** (row 19 comparison function spec gap): folded — `compare_slot_path_vs_inline_path(idx, slot_inputs, inline_path)` helper added to §2 row 4 with call-site pinned BEFORE the path_decl mutation.
- **R2 #4** (`vec.len() == n` invariant): folded — invariant statement added to §2 row 4 (`use OriginPath::empty()` for bare slots).
- **R2 #5** (wrapper-form regex match): folded — §2 GUI classifier row replaced "strip prefixes then match" with explicit 5-regex form.
- **R2 #6** (md1 wire round-trip wording): folded — §9 item 8 now references `tlv.pubkeys` X-only bytes.

### V5 user-direction reversal (2026-05-16)

User reversed Q1 lock: from "strict explicit-required" to "silent default inference with stderr info notice." Default = `m/48'/<coin>'/<account>'/2'` for all non-canonical multisig wallets (wsh, sh-wsh, tr), including the user's `wsh(andor(...))` + `tr(<key-or-NUMS>, <ms>)` cases. V5 also relaxes the existing `DESCRIPTOR_WITH_NONZERO_ACCOUNT` refusal for non-canonical mode (so `--account N` becomes meaningful). Row 15 removed from the SPEC §6.6 refusal ladder; replaced by the stderr info notice. GUI gains a path-placeholder hint + info banner (still Option-A inline; no new §6.10 vocab). Sensible defaults for 5 ambiguities chosen and documented in §1 Q1.a (network parameterization, account parameterization, n=1 uniformity, stderr-not-silent, GUI placeholder render).

### R4 findings resolved in V6

- **R4 C1** (missed GUI projection `--account → PinValue(0)` rule consequence at `gui_schema.rs:483-509`): folded — V6 keeps the gui-schema rule intact (still emitted, drift-test still asserts presence), adds a GUI Option-A override at `conditional.rs:144-149` that wraps the existing pin push in a canonicity gate. Schema vocabulary unchanged (Q4 lock preserved). Drift-gate cell gains a documented exception for the non-canonical case.
- **R4 C2** (drift-test churn enumeration): folded — toolkit-tests sizing range raised to 500-700 LOC explicitly accounting for `cli_gui_schema_v3_extensions.rs` + `cli_gui_schema_conditional_rules.rs` updates. Phase 4 also lists these as test sites under "rows 17/18/19 + acceptance test for default-inference".
- **R4 I1** (stale Phase 4 "rows 15/17/18" reference): folded — Phase 4 prose now reads "rows 17/18/19 + acceptance test for default-inference."
- **R4 I2** (silent-inference footgun risk): folded — §8 R5 added with Phase 7 user-run-the-feature smoke mandate; cycle does NOT ship if perceptibility fails.
- **R4 I3** (sizing not updated): folded — §10 toolkit LOC range raised to ~200-250, tests to ~500-700.
- **R4 I4** (GUI placeholder reads user-typed not pin-coerced): folded — §2 GUI changes table `slot_editor.rs:191` row pins this requirement.
- **R4 N1** (§6 corpus bucket reorg): item #6 explicitly described as acceptance-with-default (still under §6 "Refusal corpus" heading for index continuity; prose disambiguates).
- **R4 N2** (§11 line 324 stale): folded — bundle.rs:200-206 row now annotates restructure.
- **R4 N3** (§12 V5 narrative under-cites GUI rule): folded — this V6 section addition fills the narrative.

### Remaining R3 questions (deferrable to Phase 4 R0)

1. **Fingerprint-source provenance in row 17/18 error text** — stderr texts should match existing `friendly.rs` precedents (look at the v0.4 fingerprint-error patterns to align wording). Phase 4 R0 pins exact strings.
2. **Test fixture beef-phrases checksum status** — user's example phrases (`beef ×11 beef|access|action`) are presumed BIP-39-checksum-valid; if any fails, replace with closest checksum-valid phrase in the same shape. Phase 4 R0 pins the actual phrases + checksum status.
3. **`path_decl` vs `tlv.origin_path_overrides` for post-parse mutation site** — V3/V4 chose `path_decl` mutation for backwards-compat with existing binding code reading from it (cite verified at `bundle.rs:950-954`). Alternative would be populating `tlv.origin_path_overrides` and updating binding code to check overrides first. Phase 4 R0 decides if reviewer prefers the wire-format-cleaner separation.
4. **`make_use_site_path` behavior unchanged** — V3 §2 affirms multipath/wildcard remains in `use_site_path` (lines 193-200 of `parse_descriptor.rs`) and is not touched by the post-parse mutation. R2 confirmed.
