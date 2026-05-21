# mnemonic-toolkit-v0.31.6 Implementation Plan (Cycle 13 — seedqr-digits-from-input-unification)

> **For agentic workers:** REQUIRED SUB-SKILL: `superpowers:subagent-driven-development` or direct execution.

**Goal:** Ship `mnemonic-toolkit-v0.31.6` (SemVer-PATCH; additive surface + deprecation warning). Closes `seedqr-digits-from-input-unification` FOLLOWUP. Extends the shared `cmd/convert.rs::FromInput` / `NodeType` with a new `Seedqr` node type; wires `--from seedqr=<digits>` end-to-end for `mnemonic convert` (Option 3) and adds it to `mnemonic seedqr decode` as the canonical input form. `--digits` is preserved as a deprecated alias that emits a stderr warning when used.

**Architecture:** New `NodeType::Seedqr` variant in the shared enum. `mnemonic convert --from seedqr=<digits>` is wired by pre-decoding the digit-string to a phrase via `crate::seedqr::decode()` immediately after `primary_value` stdin-resolution in `convert::run` (L808), then substituting `primary` + `primary_value` with the equivalent `NodeType::Phrase` form. Downstream conversion dispatch (`Phrase | Entropy => ...` at L1061 etc.) sees a normal Phrase input and produces the same output as `--from phrase=<decoded>` would.

For `mnemonic seedqr decode`, the existing `--digits <value>` flag becomes optional + emits a stderr deprecation warning when used. New `--from <FromInput>` flag accepts `--from seedqr=<digit-string>` (`--from phrase=` etc. refused as wrong-node). Mutex: **clap-level `#[arg(conflicts_with = "from")]` on `digits`** (R0 I3 fold; mirrors the `--passphrase` / `--passphrase-stdin` pattern; exit 2 at parse-time). Required-input refusal at runtime if neither supplied.

**Tech Stack:** Rust; zero new deps; 1 new `NodeType` variant + 1 new `SECRET_NODE_TYPES` entry; zero `ToolkitError` variants. Toolkit-only (no GUI lockstep gates per the schema_mirror flag-name-vs-value-content scope clarification at memory `v0.28+ Wave 3 SHIPPED R0 I1`; optional GUI mirror filed as follow-on FOLLOWUP).

**P0 STRICT-GATE recon (verified at master HEAD `0693479`):**
- `cmd/convert.rs:31-45` — `NodeType` enum.
- `cmd/convert.rs:48-114` — `as_str` / `from_token` / `is_secret_bearing` / `is_argv_secret_bearing` / `is_side_input_only`.
- `cmd/convert.rs:120-151` — `FromInput` + `parse_from_input`.
- `cmd/convert.rs:751-808` — `primary` + `primary_value` resolution. Insertion site at L808+.
- `cmd/seedqr.rs:30-40` — `SeedqrDecodeArgs.digits: String` (currently required).
- `cmd/seedqr.rs:90-110` — `run_decode` consumption of `args.digits`.
- `crates/mnemonic-toolkit/src/secret_taxonomy.rs:78-84` — `SECRET_NODE_TYPES` array.

**SemVer rationale (v0.31.5 → v0.31.6 PATCH):**
- Pure additive surface: new `--from seedqr=` token + new optional `--from` flag on `seedqr decode`; existing `--digits` still works (with stderr deprecation warning).
- No CLI flag-name removal.
- New `NodeType::Seedqr` variant is a `pub` enum addition (Rust SemVer note: adding a variant to a non-exhaustive enum is a minor; the enum is currently non-`#[non_exhaustive]` so technically a public-API break for library consumers — but the toolkit is git-pinned only, not yet on crates.io). Treat as PATCH per project convention.
- GUI schema_mirror gate (clap-flag-NAME parity) does NOT fire on new `--from` value-enumeration tokens.

## File structure

### Source files modified (toolkit)
- `crates/mnemonic-toolkit/src/cmd/convert.rs`:
  - L31-45: add `Seedqr` variant to `NodeType`.
  - L66-83 `from_token` + L48-63 `as_str`: add `"seedqr"` token mapping.
  - L85-96 `is_secret_bearing`: add `Self::Seedqr` (the digit-string encodes a BIP-39 phrase).
  - L137 parser error message: add `seedqr` to the expected-tokens list.
  - L160-176 clap help block: add `seedqr` line.
  - L808-810 insertion: after `_pin_primary` and BEFORE `targets`-parse (L811), pre-decode `Seedqr` → `Phrase` and substitute `primary` + `primary_value` (synthetic `FromInput` owned via local binding). All downstream `primary.node` checks (L837-845, L857, L868, L894, L923-939 auto-fire, L994, L1008, L1061+ main dispatch) see the substituted Phrase node and proceed transparently. R0 I1 fold — cascade explicitly enumerated.
- `crates/mnemonic-toolkit/src/cmd/seedqr.rs`:
  - L30-40 `SeedqrDecodeArgs`: make `digits` an `Option<String>` with **`#[arg(conflicts_with = "from")]`** (R0 I3 fold — clap-level mutex; exit 2 at parse-time, mirrors `--passphrase` / `--passphrase-stdin`); add new `from: Option<FromInput>` field with `value_parser = parse_from_input`.
  - L90+ `run_decode`: dispatch on `(args.digits, args.from)` — both None → required-input error (clap can't enforce "at least one of two" cleanly; runtime BadInput exit 1); `args.digits = Some(_)` → emit stderr deprecation warning + use the digit value; `args.from = Some(fi)` → assert `fi.node == NodeType::Seedqr` + use the value. Clap enforces "not both" via `conflicts_with`.
- `crates/mnemonic-toolkit/src/secret_taxonomy.rs:80`:
  - Add `"seedqr"` to `SECRET_NODE_TYPES`.

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/src/cmd/convert.rs` (in-file `tests` mod if it has one, otherwise via integration tests):
  - Coverage left to integration tests; no in-file unit cells required.
- `crates/mnemonic-toolkit/tests/cli_convert.rs` (or wherever convert tests live):
  - `convert_from_seedqr_to_phrase_happy_path` — `convert --from seedqr=<DIGITS_12> --to phrase` outputs PHRASE_12.
  - `convert_from_seedqr_to_entropy_happy_path` — pipes through to entropy.
  - `convert_from_seedqr_invalid_digits_refused` — decode error surfaces correctly.
  - **`convert_from_seedqr_stdin_to_phrase_happy_path`** — `convert --from seedqr=- --to phrase` consumes digits from stdin (R0 M1 fold).
- `crates/mnemonic-toolkit/tests/cli_seedqr.rs`:
  - `decode_from_seedqr_happy_path` — `seedqr decode --from seedqr=<digits>` succeeds with expected phrase.
  - `decode_digits_deprecation_warning` — `seedqr decode --digits <value>` still succeeds + emits stderr warning containing `"deprecated"` and citing `--from seedqr=`.
  - `decode_both_digits_and_from_refused` — both flags supplied → exit 1 + conflict error message.
  - `decode_neither_digits_nor_from_required_input` — neither flag supplied → exit 1 + required-input error message (clap-level OR runtime).
  - `decode_from_non_seedqr_node_refused` — `seedqr decode --from phrase=<phrase>` refused at the node-check (the seedqr decode subcommand only accepts the seedqr node type via `--from`).

### Documentation modified (toolkit)
- `docs/manual/src/40-cli-reference/41-mnemonic.md`:
  - `mnemonic convert` section: add `seedqr` row to the `--from` node enumeration with brief description.
  - `mnemonic seedqr decode` section: document the new `--from seedqr=<digits>` canonical form + `--digits` deprecation note.

### Release tooling
- `crates/mnemonic-toolkit/Cargo.toml:3` — `0.31.5` → `0.31.6`.
- `CHANGELOG.md` — new `## [0.31.6]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.31.5` → `mnemonic-toolkit-v0.31.6`.
- `design/FOLLOWUPS.md` — close `seedqr-digits-from-input-unification`. File NEW: `gui-seedqr-node-type-help-mirror` (optional GUI v0.16.2 mirror; supply-chain drift snapshot will fire and need acknowledgment).

## Tasks

### Task 1: Phase 2 — convert.rs NodeType extension

- [ ] Add `Seedqr` variant + `from_token` / `as_str` / `is_secret_bearing` updates + parser error-message + clap help block update.
- [ ] Add the pre-decode substitution block in `convert::run` after L808 (`_pin_primary`).
- [ ] Update `secret_taxonomy::SECRET_NODE_TYPES` to include `"seedqr"`.
- [ ] Build + run relevant tests.
- [ ] Commit Phase 2.

### Task 2: Phase 3 — seedqr decode flag extension

- [ ] Make `args.digits` an `Option<String>`; add `args.from: Option<FromInput>`.
- [ ] Implement dispatch + deprecation warning + mutex.
- [ ] Build + run.
- [ ] Commit Phase 3.

### Task 3: Phase 4 — Integration tests

- [ ] Add convert-side cells.
- [ ] Add seedqr-side cells.
- [ ] Build + run.
- [ ] Commit Phase 4.

### Task 4: Phase 5 — Manual chapter mirror

- [ ] Update convert + seedqr sections.
- [ ] Run manual lint.
- [ ] Commit Phase 5.

### Task 5: Phase 6 — Cycle close

- [ ] Version bump + install.sh + CHANGELOG.
- [ ] Pre-tag audit (test + clippy + manual lint).
- [ ] Opus end-of-cycle review.
- [ ] Commit + tag + push + GH Release.
- [ ] Wait for install-pin-check CI.
- [ ] Close FOLLOWUP + file `gui-seedqr-node-type-help-mirror` follow-on.

## Cross-phase invariants

- Opus R0 review on plan-doc BEFORE Phase 2 dispatch.
- Opus end-of-cycle review BEFORE tagging.
- No GUI lockstep gate (schema_mirror is flag-name-parity, not value-content). GUI mirror filed as optional FOLLOWUP.
- install-pin-check CI gate.

## Risk register

- **`is_argv_secret_bearing` semantics** — adding `Seedqr` to `is_secret_bearing` automatically makes `is_argv_secret_bearing` true via the existing `is_secret_bearing() || MiniKey` composition. `--from seedqr=<digits>` should emit the argv-leakage advisory. Verify via integration cell.
- **Stdin-mutex coexistence** — `--from seedqr=-` participates in the single-stdin-per-invocation invariant via the existing `primary_uses_stdin = primary.value == "-"` check at L759. The substitution happens AFTER this check, so the mutex still fires correctly.
- **NodeType drift across files** — `slot_input.rs::SlotSubkey` already has `Seedqr` (added in v0.31.3); now `cmd/convert.rs::NodeType` gets a parallel `Seedqr`. Two separate enums; intentional (slot grammar vs convert grammar). Cross-file naming consistency lookup OK.
- **`SECRET_NODE_TYPES` GUI supply-chain drift gate** — adding `"seedqr"` will fire the v0.3.3 canonical fallback gate at GUI compile time on next pin bump. Handle in the optional GUI v0.16.2 follow-on.
- **`flag_is_secret("--digits")` lockstep** (R0 I2 fold) — `secrets.rs::SECRET_FLAG_NAMES` includes `"--digits"`; v0.31.6 deprecation does NOT change this (deprecated values still leak). Explicitly preserve `--digits` in `SECRET_FLAG_NAMES`. The new `--from` flag is value-dependent and already covered by `is_argv_secret_bearing` flow.

## Self-review (pre-R0 dispatch)

- ✓ P0 recon line citations match HEAD.
- ✓ Insertion-site at L808 chosen for stdin-mutex coexistence.
- ✓ SemVer PATCH justified (additive + deprecation-warning only).
- ✓ Mutex + deprecation UX explicit.
- ✓ Test surface enumerated (3 convert + 5 seedqr-decode cells).
- ✓ GUI lockstep correctly classified as optional follow-on (memory R0 I1 lesson).
