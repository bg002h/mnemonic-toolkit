# BRAINSTORM: `repair` + `inspect` feature for v0.22.0 cycle

**Date:** 2026-05-17
**Toolkit target:** `mnemonic-toolkit-v0.22.0` (minor bump)
**Status:** brainstorm approved across all 6 design sections; ready for SPEC + IMPLEMENTATION_PLAN drafting
**Predecessor cycle:** `mnemonic-toolkit-v0.21.0` (SPEC §5.8 per-slot ms1 conformance, shipped 2026-05-17)

## §0 Motivation

The m-format constellation emits BCH-checksummed bech32-family cards (`ms1` / `mk1` / `md1`) designed for steel-plate engraving and physical recovery. A user recovering a corroded / mis-engraved card today gets a hard decode failure with no fix suggestion. The underlying BCH(93,80,8) codex32 code (BIP-93) supports up to t=4 substitution corrections per chunk — but neither the toolkit nor 2 of 3 sibling codecs surface this capability publicly.

**User intent (2026-05-17):**
> "'repair' for all incorrect ms1, mk1, and md1 strings. When decode fails, the repair function should run automatically and the error should be indicated on stderr and the corrected string returned on stdout. If no corrected string can be generated, fail loudly. … Repair should run whenever decode or inspect or verify fails."

## §1 Research baseline (3 parallel agents, 2026-05-17)

### §1.1 Existing toolkit surface

- **Zero CLI-level repair today.** Neither `verify-bundle` nor `convert` nor `bundle --self-check` attempts correction on decode failure.
- `verify-bundle` emits `VerifyCheck { passed: false, decode_error: Some(...) }` on per-card decode fail (`cmd/verify_bundle.rs:887/937/981`); exit 0 unless `--fail-fast`.
- `convert` returns `Err(ToolkitError::from(...))` on decode fail (`cmd/convert.rs:1268/1307`); exit 2.
- No FOLLOWUPs filed explicitly for repair (closest: `bip93-invalid-corpus-granular-error-pin` in ms-codec).

### §1.2 Sibling-codec API asymmetry

| Codec | `decode_with_correction` public? | Cite |
|---|---|---|
| `ms-codec` | NO — delegates to `rust-codex32` (no public correction in upstream) | `ms-codec/src/decode.rs` + upstream `rust-codex32 v0.1.0` |
| `mk-codec` | YES — `bch_correct_regular()` + `bch_correct_long()` public, t=4 capacity, 8+ injection tests | `mk-codec/src/string_layer/bch.rs:392/450/1088-1305` |
| `md-codec` | NO — `bch_verify_regular()` is `pub(crate)` only | `md-codec/src/bch.rs:70` (`pub(crate)`) |

All three codecs use BIP-93 codex32 BCH(93,80,8); cross-codec asymmetry is API-surface-only, not algorithm-level.

### §1.3 Upstream `rust-bech32 v0.11`

Ships full Berlekamp-Massey + Forney corrector at `src/primitives/correction.rs` (MIT). Public API:
```rust
trait CorrectableError { fn correction_context<Ck: Checksum>(&self) -> Option<Corrector<Ck>>; }
impl<Ck: Checksum> Corrector<Ck> {
    pub fn singleton_bound(&self) -> usize;
    pub fn add_erasures(&mut self, locs: &[usize]);
    pub fn bch_errors(&self) -> Option<ErrorIterator<'_, Ck>>;
}
```
Single best-guess only (no top-N). `None` on uniqueness violation (too many errors). HRP-corruption unrecoverable by design (HRP injected outside BCH-protected bit-range). Reference: <https://github.com/rust-bitcoin/rust-bech32/blob/master/src/primitives/correction.rs>.

`Checksum` trait requires per-HRP boilerplate (~8 lines per HRP): `CHECKSUM_LENGTH=13`, `CODE_LENGTH=93`, `GENERATOR_SH=[…]` (BIP-93 generator), `TARGET_RESIDUE=<NUMS-derived>`, `ROOT_GENERATOR=Fe32::S`, `ROOT_EXPONENTS=9..=16`.

`TARGET_RESIDUE` per HRP (codex32 NUMS targets — SHA-256-derived domain separators):
- `ms` HRP: codex32's standard `"SECRETSHARE32"` target OR a sibling-codec-specific NUMS — confirm against `ms-codec` source during Phase 0.
- `mk` HRP: `SHA-256("shibbolethnumskey")` top-65-bits per `mk-codec/src/consts.rs::MK_REGULAR_CONST`.
- `md` HRP: `SHA-256("shibbolethnums")` top-65-bits per `md-codec/src/bch.rs:17::MD_REGULAR_CONST = 0x0815c07747a3392e7`.

## §2 Locked design decisions (post-clarifying-questions)

**D1 — Scope: hybrid (toolkit-first + cross-repo FOLLOWUPs).** Toolkit lands repair this cycle by vendoring 3 thin `Checksum` impls atop rust-bech32 v0.11. Cross-repo FOLLOWUPs (filed at cycle close) extend `ms-codec` + `md-codec` with public `decode_with_correction()` APIs in lockstep follow-on cycles. Sibling CLIs (`ms` / `md`) get `--repair` flags once their codecs surface the API.

**D2 — Two new subcommands: `mnemonic repair` AND `mnemonic inspect`.** Toolkit grows from 8 to 10 subcommands. `repair` is the explicit user entry point for "I know my card is broken; fix it." `inspect` is the structured-metadata viewer for `ms1` / `mk1` / `md1` cards (per-HRP fields surfaced + auto-fires repair on its own decode failure).

**D3 — Auto-fire UX: short-circuit semantics.** When decode fails inside an existing subcommand AND a unique correction exists: stderr emits original error + correction details; stdout emits the corrected string ONLY; exit code = 5 (new `REPAIR_APPLIED`). User must re-run with the corrected card. Never silently substitutes user input. Cryptographically honest: surfaces every correction for user trust evaluation.

**D4 — Input shape: flag-repeated, matching `verify-bundle`.** `mnemonic repair --ms1 <string>` / `mnemonic repair --mk1 <chunk> --mk1 <chunk> ...` / `mnemonic repair --md1 <chunk> --md1 <chunk> ...`. Mirror flags on `mnemonic inspect`. Multi-chunk output on stdout = chunks separated by newlines, preserving order. Stdin form via existing `*-stdin` pattern (`--ms1=-`, etc.).

**D5 — Architecture: Approach A (shared `repair` module + per-site call-site integration).** Pure-function `repair_card(CardKind, &[String]) -> Result<RepairOutcome, RepairError>` in new `src/repair.rs`. Each existing decode-failure site (verify-bundle, convert, bundle --self-check, bundle --bundle-json intake, the new inspect subcommand) gains a uniform 5-line short-circuit pattern. NO middleware abstraction (Approach B/C rejected — entangles I/O with primitive; ripples error variants through every layer).

**D6 — `--no-auto-repair` suppression flag.** Global flag (default `false`); when set, all auto-fire sites skip repair entirely; original decode-fail exit policy fires. Standalone `mnemonic repair` IGNORES this flag.

**D7 — Cross-codec uniformity preserved by single source-of-truth in toolkit.** Even though `mk-codec` already exposes `bch_correct_regular()` publicly, the toolkit's `repair.rs` deliberately does NOT call it — using rust-bech32 v0.11 directly keeps the correction pipeline byte-identical across all 3 HRPs. Cross-codec NUMS-target constants are vendored locally + drift-gated against the sibling codecs via `#[cfg(test)]` `assert_eq!(VENDORED, ms_codec::ms_nums_target())` cells per `[[feedback-build-rs-stub-fallback-security-audit]]`.

**D8 — Multi-chunk atomic semantics.** Within a multi-chunk mk1/md1 input: if ANY chunk fails repair, the WHOLE `repair_card` call returns `Err` naming that chunk; successfully-repaired sibling chunks are NOT emitted. Rationale: partial-repair is ambiguous; atomic all-or-nothing makes the report unambiguous.

**D9 — Sensitive-secret leakage discipline.** Repair emits the corrected ms1 (which encodes BIP-39 entropy) to stdout per user request; mirrors existing `bundle` argv-leakage discipline by emitting a stderr `warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')`. NO diagnostic / debug logging of corrected chunks.

**D10 — HRP corruption distinct from BCH corruption.** `RepairError::HrpMismatch` is its own variant (not collapsed into `TooManyErrors`) because the user-actionable fix differs (re-type prefix vs. re-engrave card) AND the HRP is mathematically outside the BCH corrector's scope. Future cycle could add HRP heuristics (Levenshtein-1 over `{ms, mk, md}`); out of scope this cycle.

## §3 Architecture

### §3.1 Module layout

```
crates/mnemonic-toolkit/src/
├── repair.rs          NEW (~250 LOC + tests)
│   ├── mod ms_checksum  // private; impl bech32::Checksum for MsCodex32
│   ├── mod mk_checksum  // private; impl bech32::Checksum for MkCodex32
│   ├── mod md_checksum  // private; impl bech32::Checksum for MdCodex32
│   ├── pub enum CardKind { Ms1, Mk1, Md1 }
│   ├── pub struct RepairDetail { chunk_index, original_chunk, corrected_chunk, corrected_positions: Vec<(usize, char, char)> }
│   ├── pub struct RepairOutcome { kind: CardKind, corrected_chunks: Vec<String>, repairs: Vec<RepairDetail> }
│   ├── pub enum RepairError { EmptyInput, HrpMismatch{…}, TooManyErrors{…}, UnparseableInput{…} }
│   ├── pub fn repair_card(CardKind, &[String]) -> Result<RepairOutcome, RepairError>
│   ├── pub fn try_repair_and_short_circuit(kind, chunks, &orig_err, stdout, stderr) -> Result<RepairShortCircuit, RepairError>
│   ├── pub enum RepairShortCircuit { Exit5 }
│   └── fn emit_repair_report(&outcome, &orig_err, stdout, stderr) -> io::Result<()>
├── cmd/
│   ├── repair.rs      NEW (~150 LOC) — `mnemonic repair --ms1/--mk1/--md1`
│   ├── inspect.rs     NEW (~200 LOC) — `mnemonic inspect --ms1/--mk1/--md1`
│   ├── verify_bundle.rs  EDIT — auto-fire at 4 sites (3 reparse_bundle per-card + 1 --bundle-json intake)
│   ├── convert.rs        EDIT — auto-fire at --from {ms1,mk1,md1} decode sites (3 sites)
│   └── bundle.rs         EDIT — auto-fire at --self-check per-chunk decode (1 site)
├── lib.rs / main.rs   EDIT — register 2 new subcommands; new ExitCode::REPAIR_APPLIED = 5; new global --no-auto-repair flag
```

### §3.2 Dependency

New `Cargo.toml` direct dep: `bech32 = "0.11"` (gates `primitives::correction` module). MIT-licensed; pure-Rust; std/alloc feature-gated. No heavy transitives.

### §3.3 Auto-fire integration sites (6 total + 1 in the new `inspect` subcommand)

| # | File | Decode call | Behavior on decode-fail |
|---|---|---|---|
| 1 | `cmd/verify_bundle.rs::reparse_bundle` ms1 path (~:887) | `ms_codec::decode(s)` | Call `try_repair_and_short_circuit(Ms1, &[s], &orig, stdout, stderr)`. Ok → exit 5. Err → existing per-card `VerifyCheck { passed: false, decode_error }` path. |
| 2 | `cmd/verify_bundle.rs::reparse_bundle` mk1 path (~:937) | `mk_codec::decode(...)` per chunk | Same pattern, kind=Mk1. |
| 3 | `cmd/verify_bundle.rs::reparse_bundle` md1 path (~:981) | `md_codec::decode(...)` | Same, kind=Md1. |
| 4 | `cmd/convert.rs --from ms1=<s>` | `ms_codec::decode(s)` | Short-circuit; exit 5 on repair-success. |
| 5 | `cmd/convert.rs --from mk1=<s>` | `mk_codec::decode(s)` | Same. |
| 6 | `cmd/convert.rs --from md1=<s>` | `md_codec::decode(s)` | Same. |
| 7 | `cmd/bundle.rs::self_check_bundle` per-chunk decode | per-card `<codec>::decode(...)` | Same. Per `[[feedback-self-check-bypasses-csi-grouping]]`, self-check iterates chunks separately — repair is per-chunk independent, matches the model. |
| 8 | `cmd/verify_bundle.rs::verify_bundle_json_intake` (downstream decode after JSON parse) | per-card decode | Same. |
| 9 | `cmd/inspect.rs::run` (the new subcommand's own decode-then-render flow) | per-card decode | Same. |

Final count: 9 short-circuit insertion sites (8 in existing code + 1 in new `inspect`).

## §4 CLI surface details

### §4.1 `mnemonic repair`

```
USAGE:
    mnemonic repair [OPTIONS]

OPTIONS:
        --ms1 <MS1>     ms1 string (single-chunk). Mutex with --mk1/--md1.
        --mk1 <MK1>     mk1 chunk; repeat for multi-chunk cards. Mutex with --ms1/--md1.
        --md1 <MD1>     md1 chunk; repeat for multi-chunk cards. Mutex with --ms1/--mk1.
        --json          Emit structured JSON instead of text.
    -h, --help          Print help
```

Text-form stdout:
```
# Repair report
#   ms1 chunk 0: 1 correction at position 47: '8' -> 'f'
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

JSON-form stdout (schema_version=1):
```json
{
  "schema_version": "1",
  "kind": "ms1",
  "corrected_chunks": ["ms10entrsq..."],
  "repairs": [
    { "chunk_index": 0,
      "corrected_positions": [{"position": 47, "was": "8", "now": "f"}],
      "original_chunk":  "ms10entrsq...v8f",
      "corrected_chunk": "ms10entrsq...v7f" }
  ]
}
```

### §4.2 `mnemonic inspect`

Same flag shape as `repair`. Default text-form output prints structured metadata:
- ms1: codec version, tag (`ENTR` / future tags), payload byte count, bit-strength. Hex payload gated behind `--reveal-secret` flag (default off — protects engraved entropy from accidental disclosure).
- mk1: chunk count, chunk_set_id, xpub fingerprint, derivation path, xpub depth, network kind.
- md1: chunk count, descriptor wire-tag, decoded descriptor body, wallet-policy mode flag.

Auto-fires repair on own decode failure (item #9 in §3.3).

### §4.3 Exit codes

| Code | Meaning |
|---|---|
| 0 | Success (or repair found input already valid — no-op) |
| 1 | Generic toolkit failure |
| 2 | User-input error (existing) — includes `RepairError::*` from standalone `repair` |
| 3 | (reserved by existing convention) |
| 4 | Verify-bundle mismatch (existing) |
| 5 | **NEW: `REPAIR_APPLIED`** — repair-succeeded-with-corrections; user must re-run with corrected card |

## §5 Error handling

Per Section 5 design table (failure-mode taxonomy):

| Variant | When | Stderr | Exit |
|---|---|---|---|
| `EmptyInput` | Zero chunks supplied | `error: repair: no chunks supplied` | 2 |
| `HrpMismatch` | Prefix doesn't match flag's kind / unknown HRP | `error: repair: chunk N HRP mismatch — expected 'X', found 'Y' (HRP is not BCH-protected; re-type the prefix)` | 2 |
| `TooManyErrors` | `Corrector::bch_errors()` returns `None` (corruption > t=4) | `error: repair: chunk N has too many errors to correct uniquely (exceeds singleton bound = 8); cannot suggest correction` | 2 |
| `UnparseableInput` | bech32 layer parse-fail (malformed char, missing separator, etc.) | `error: repair: chunk N parse failed before correction could run: <detail>` | 2 |

**Auto-fire fallthrough discipline.** When `repair_card` returns `Err(_)` inside an auto-fire site, the site falls through to existing error-handling with the ORIGINAL decode error preserved. So `convert --from ms1=<irreparable>` still exits with existing exit-2 + existing decode error message, unchanged from today's UX. Repair never makes things worse than status quo.

**`--no-auto-repair` interaction.** Auto-fire sites skip `try_repair_and_short_circuit` entirely; original decode error fires immediately. Standalone `mnemonic repair` ignores this flag.

## §6 Testing strategy

### §6.1 Unit tests in `src/repair.rs::mod tests` (8 cells)

1. Happy-path per HRP (×3): encode → flip 1 char → assert correct repair + position. Plus `corrections_applied == 0` pass-through cell.
2. t=4 boundary: 4 substitutions → Ok with 4 reports; 5 substitutions → Err(TooManyErrors).
3. HRP mismatch: `repair_card(Ms1, &["mk1foo...".to_string()])` → Err(HrpMismatch).
4. Multi-chunk all-valid mk1: 3 valid chunks → Ok with empty `repairs`.
5. Multi-chunk one-corrupted mk1: chunk 1 flipped → Ok with 1 RepairDetail at chunk_index=1.
6. Multi-chunk atomic failure: chunk 1 irreparable + chunk 2 repairable → Err(TooManyErrors, chunk_index=1); chunk 2's potential correction NOT applied.
7. EmptyInput: `repair_card(Mk1, &[])` → Err(EmptyInput).
8. Cross-codec NUMS-target constancy: `assert_eq!(MS_NUMS_TARGET, ms_codec::ms_nums_target())` × 3 HRPs. Drift gate.

### §6.2 Integration tests in `tests/cli_repair.rs` (6 cells)

9. Standalone repair text-form ms1 happy-path.
10. Standalone repair `--json` ms1.
11. Standalone repair already-valid input → exit 0 pass-through.
12. Standalone repair unrepairable → exit 2 with `TooManyErrors` stderr.
13. Standalone repair multi-chunk mk1 (3 chunks, chunk 1 flipped).
14. Stdin form `--ms1=-` per existing `*-stdin` pattern.

### §6.3 Auto-fire integration tests in `tests/cli_auto_repair.rs` (5 cells)

15. `convert --from ms1=<1-error>` → exit 5 + corrected ms1 on stdout.
16. `verify-bundle --bundle-json <json-with-bad-mk1-chunk>` → exit 5; existing verify output suppressed.
17. `bundle --self-check` with synthetic chunk-corruption → short-circuit.
18. `--no-auto-repair` flag → reverts to existing decode-fail exit policy byte-exactly.
19. `inspect --ms1 <bad>` → exit 5 + corrected.

### §6.4 Cross-cycle regression guards

20. v0.21.0 `descriptor_mode_3_of_3_emits_per_slot_ms1_post_v0_21` cell still passes (auto-fire integration must not break existing pass-path).
21. v0.20.0 multi-cosigner round-trip cells (cli_verify_bundle_multi_cosigner_mk1.rs cells 1-7) all pass.
22. Manual lint (`make -C docs/manual lint ...`) green after the new chapter for `repair` / `inspect`.

### §6.5 Discipline memos applied

- `[[feedback-default-cargo-test-runs-sibling-dependent-tests]]` — full suite via `cargo test --workspace --all-features`.
- `[[feedback-opus-primary-review-agent]]` — all R0 dispatches use `feature-dev:code-reviewer` with `model: "opus"`.
- `[[feedback-architect-must-run-prose-commands]]` — Phase 4 manual chapter regen pastes byte-exact stdout/stderr from running the new subcommands.
- `[[feedback-verify-bundle-round-trip-per-phase-r0-scope]]` — auto-fire tests inside verify-bundle exercise the round-trip.
- `[[feedback-r0-must-read-source-off-by-n]]` — every plan-doc + phase R0 cite confirmed by source-grep.
- `[[feedback-build-rs-stub-fallback-security-audit]]` — vendored NUMS targets drift-gated via `#[cfg(test)]` equality assertions against the sibling codecs.

## §7 Cross-repo FOLLOWUPs (file at cycle close per D1)

1. `ms-codec-decode-with-correction-public-api` — extend `ms-codec` to surface a public `decode_with_correction(s) -> Result<(Tag, Payload, CorrectionReport), Error>` that wraps either rust-bech32 v0.11's `Corrector` OR (preferred) drives a rewritten `rust-codex32`-on-`rust-bech32` substrate.
2. `md-codec-decode-with-correction-public-api` — promote `md-codec::bch::bch_verify_regular` family from `pub(crate)` to `pub` + add `decode_with_correction`. Companion to (1).
3. `ms-cli-repair-flag` — add `ms repair <ms1>` subcommand to `ms-cli`. Blocked on (1).
4. `mk-cli-repair-flag` — add `mk repair <mk1>` subcommand to `mk-cli`. Unblocked (mk-codec already has the API).
5. `md-cli-repair-flag` — add `md repair <md1>` subcommand to `md-cli`. Blocked on (2).
6. `toolkit-repair-consume-native-codec-api` — once (1) + (2) land, refactor toolkit's `repair.rs` to call `<codec>::decode_with_correction()` instead of its own vendored `Checksum` impls. NUMS-target constants get deleted from toolkit.
7. `hrp-correction-heuristics` — Levenshtein-1 over `{ms, mk, md}` for the prefix; reject ambiguous (Lev-1 of `xs1` = both `ms1` and `mk1` and `md1`?). UX nicety; not load-bearing.

## §8 Out-of-scope this cycle

- **Top-N candidates / confidence scores.** rust-bech32 only exposes single-best-guess; future cycle could fork or extend if needed.
- **Erasure correction with hinted positions.** rust-bech32 supports `Corrector::add_erasures()` for known-bad positions; future cycle could surface `--erasure N` flags.
- **Interactive "did you mean…?" prompts.** Single-best-guess only; user re-runs.
- **HRP-correction heuristics.** Filed as FOLLOWUP #7.
- **Sibling-CLI repair flags.** Filed as FOLLOWUPs #3-#5.
- **Sibling-codec API extensions.** Filed as FOLLOWUPs #1-#2.

## §9 Risks & open questions for SPEC + IMPLEMENTATION_PLAN drafting

- **R1: `Checksum::TARGET_RESIDUE` for `ms` HRP.** The codex32 BIP-93 standard target is `"SECRETSHARE32"` (Bech32 string), giving residue `0x10ce0795c2fd1e62a`. The sibling `ms-codec` might use this standard OR a shibboleth-specific NUMS — verify in Phase 0 by reading `ms-codec/src/`.
- **R2: `mk-codec` long-code (BCH(108,93,8)) support.** Toolkit's `repair.rs` initially supports the REGULAR code (BCH(93,80,8), 13-char checksum). Long-code support requires a 4th `Checksum` impl with different `CODE_LENGTH = 108` + `CHECKSUM_LENGTH = 15`. Decide in SPEC: include long-code at launch OR file as FOLLOWUP.
- **R3: `--no-auto-repair` placement (global vs per-subcommand).** Global is cleaner UX; per-subcommand is more granular. Plan-doc to decide.
- **R4: Auto-fire interaction with `--json` / `--fail-fast` modes.** When verify-bundle runs with `--json --fail-fast` and an auto-fire short-circuit happens, does the short-circuit emit JSON or text? Plan-doc to decide.
- **R5: Manual chapter placement.** New `docs/manual/src/40-cli-reference/{N}-repair.md` and `{N+1}-inspect.md` OR sub-chapters under the existing 41-mnemonic.md? Decide in Phase 4.

## §10 Cycle scope estimate

- **LOC delta:** ~250 (repair.rs) + ~150 (cmd/repair.rs) + ~200 (cmd/inspect.rs) + ~90 (9 auto-fire-site insertions × 5 LOC × overhead — though inspect's site is internal to cmd/inspect.rs's own LOC) + ~30 (lib.rs/main.rs additions) ≈ **~700 LOC**.
- **Tests:** +19 cells (8 unit + 6 integration repair + 5 integration auto-fire). Plus 3 cross-cycle regression guards.
- **Manual:** new chapter (~200 LOC of markdown).
- **SPEC:** new section §X for repair semantics (~150 LOC). Update `design/SPEC_mnemonic_toolkit_v0_5.md` carry-forward block.
- **Phases:** estimated 7 phases (recon → SPEC → core repair module → standalone subcommands → auto-fire integration → tests → manual + release). Higher than recent cycles due to scope.
- **Reviewer-loop:** 1-2 rounds per phase + end-of-cycle pass per `[[feedback-opus-primary-review-agent]]`.

## §11 Next steps

1. Spec self-review (per superpowers:brainstorming) — placeholder scan, internal consistency, scope check, ambiguity check.
2. User reviews this written spec at `design/BRAINSTORM_repair_v0_22.md`; iterates if needed.
3. Invoke `superpowers:writing-plans` to draft the IMPLEMENTATION_PLAN at `design/IMPLEMENTATION_PLAN_repair_v0_22.md`.
4. Plan-doc reviewer-loop per `[[feedback-plan-artifact-mirror-project-convention]]` (R0 → R1 → ... → 0C/0I).
5. ExitPlanMode → execute phases per the approved plan.
