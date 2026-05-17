# IMPLEMENTATION PLAN — `repair` + `inspect` feature for v0.22.0 cycle

**Toolkit target:** `mnemonic-toolkit-v0.22.0` (minor bump)
**GUI target:** none (no lockstep — CLI surface only; GUI dispatches to CLI per established pattern)
**Predecessor:** `mnemonic-toolkit-v0.21.0` (`5778688`, shipped 2026-05-17)
**Brainstorm:** `design/BRAINSTORM_repair_v0_22.md` (approved across all 6 design sections)
**Plan status:** R0 complete (7C + 11I + 7N folded); R1 complete (5C + 3I + 2N folded; user picked helper-refactor architecture for C1); R2 pending

## §0 Context

Per BRAINSTORM §0: users recovering corroded / mis-engraved m-format cards (`ms1` / `mk1` / `md1`) today see hard decode failures with no fix suggestion. The underlying BIP-93 codex32 BCH(93,80,8) supports up to t=4 substitution corrections per chunk. This cycle:

1. Adds a pure-function `repair_card()` primitive in toolkit-side `src/repair.rs` (vendored 3-HRP `Checksum` impls over `rust-bech32 v0.11`).
2. Adds 2 new subcommands: `mnemonic repair` + `mnemonic inspect`.
3. **Refactors `emit_verify_checks` / `emit_multisig_checks` / `emit_md1_checks` to return `Result<_, ShortCircuit>`** so auto-fire can propagate via `?` through the helper hierarchy (per R1 C1 + user's architecture pick).
4. Wires auto-fire short-circuit into **11 existing decode-failure sites** (8 in `verify_bundle.rs` across the 3 helper functions + 2 in `convert.rs::compute_outputs`; `bundle.rs::self_check_bundle` excluded per D16 + R0 C7 fold).
5. Adds per-`run()` `no_auto_repair: bool` parameter propagated from a global clap flag (per R0 I6 fold).
6. Adds new exit code 5 (`REPAIR_APPLIED`) wired via `ShortCircuit { exit_code: u8 }` propagation.
7. Files 11 cross-repo FOLLOWUPs (7 from brainstorm + 4 from plan-doc folds) for sibling-codec API symmetry + sibling-CLI repair flags + cycle-residuals.

User intent (2026-05-17): "Repair should run whenever decode or inspect or verify fails. Fail loudly when no correction is possible. Error on stderr; corrected string on stdout."

## §1 Locked decisions

Carry forward all 10 decisions from BRAINSTORM §2 (D1-D10):

- **D1 — Hybrid scope** (toolkit-first + cross-repo FOLLOWUPs).
- **D2 — Both `mnemonic repair` AND `mnemonic inspect`** subcommands.
- **D3 — Short-circuit auto-fire UX** (exit 5; stderr = error + correction; stdout = corrected string only).
- **D4 — Flag-repeated input** (`--ms1 <s>` / `--mk1 <s> --mk1 <s> ...` / `--md1 <s> --md1 <s> ...`).
- **D5 — Approach A** (shared `repair` module + per-site call-site integration).
- **D6 — `--no-auto-repair` global flag** (default `false`; standalone `repair` ignores).
- **D7 — Toolkit owns the correction primitive** via rust-bech32 v0.11; NOT mk-codec's native API (cross-HRP uniformity).
- **D8 — Multi-chunk atomic semantics** (any chunk fails → whole call fails).
- **D9 — Sensitive-secret leakage discipline** (stderr warning on ms1 stdout emission).
- **D10 — HRP corruption distinct** from BCH corruption (`RepairError::HrpMismatch` separate variant).

Plus 7 additional locks resolving BRAINSTORM §9 open questions + R0/R1 folds:

- **D11 — `ms` HRP TARGET_RESIDUE locked TODAY: codex32-standard `SECRETSHARE32` = `0x10ce0795c2fd1e62a`.** Sourced from upstream `rust-codex32` + BIP-93. Vendored in `repair.rs::ms_checksum::MS_NUMS_TARGET` with `#[cfg(test)]` recomputation drift-gate.
- **D12 — mk-codec long-code BCH(108,93,8) FOLDED INTO v0.22 (revised 2026-05-17).** Original plan deferred long-code to FOLLOWUP, but Phase 1 stability testing revealed that the FIRST chunk of typical mk1 emissions (the xpub-bearing chunk, 108-char data-part) uses the LONG code. Deferring would have made the feature unable to repair the chunks users most often need fixed. Resolution: `mk_codec::string_layer::bch_decode::decode_long_errors` (already promoted in the v0.3.1 lockstep release) handles long-code decode; `repair.rs` adds length-based dispatch via `bch_code_for_length` + new `MK_LONG_TARGET = mk_codec::MK_LONG_CONST = 0x41890d7e441cbe97273`. `ms` and `md` do not define long-code variants in their v0.1 codecs, so length-detected long-code chunks for those HRPs return `RepairError::UnsupportedCodeVariant` (clear typed error). Reserved-invalid lengths [94, 95] return `RepairError::ReservedInvalidLength`. Net cost: ~40 LOC dispatch + 5 unit-test cells (mk1 long stability, mk1 long happy-path, mk1 long passthrough, ms1 long fail-fast, reserved-invalid). FOLLOWUP `repair-mk-codec-long-code-support` is REMOVED from the cycle-close list.
- **D13 — `--no-auto-repair` is a GLOBAL clap flag, propagated as 5th `no_auto_repair: bool` param** to `bundle::run`, `verify_bundle::run`, `convert::run`, `inspect::run` (4 signatures change; `repair::run` omits). Per R0 I6 fold (no `RunContext` refactor).
- **D14 — Auto-fire short-circuit emits TEXT-form repair report regardless of `--json`.**
- **D15 — Manual chapter placement: new dedicated chapters** `42-repair.md` + `43-inspect.md`.
- **D16 — `bundle.rs::self_check_bundle` is NOT in auto-fire scope** (toolkit-synthesized chunks; repair would mask toolkit bugs).
- **D17 — Helper-signature refactor for short-circuit propagation (R1 C1 + user architecture pick).** `emit_verify_checks` and `emit_multisig_checks` change return type from `Vec<VerifyCheck>` to `Result<Vec<VerifyCheck>, ShortCircuit>`. `emit_md1_checks` gains a `&mut Option<ShortCircuit>` sidechannel parameter (existing `&mut Vec<VerifyCheck>` already-mutated style). `convert.rs::compute_outputs` adopts the same `Result<..., ShortCircuit>` pattern OR absorbs `ShortCircuit` into a new `ToolkitError::RepairShortCircuit { exit_code: u8 }` variant whose `exit_code()` returns 5 (decision in Phase 5 R0: cleaner to add a new ToolkitError variant since compute_outputs already returns Result<_, ToolkitError>; verify_bundle helpers don't currently return Result so need the new ShortCircuit type). **Architecture lock:** introduce `ToolkitError::RepairShortCircuit { exit_code: u8 }` variant; convert.rs uses `?` propagation natively; verify_bundle.rs helpers change to `Result<Vec<VerifyCheck>, ToolkitError>` (using the existing error type's new variant). This uniforms the propagation pipeline across all auto-fire sites.

## §2 Architectural strategy (file-level inventory)

### §2.1 New module: `crates/mnemonic-toolkit/src/repair.rs`

**Public API (R0 C5 + R1 C4 fold — generic trait bounds matching codebase convention):**

```rust
use bech32::primitives::correction::{CorrectableError, Corrector};
use bech32::{Checksum, Fe32};

/// Which m-format card kind drives this repair invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKind { Ms1, Mk1, Md1 }

impl CardKind {
    pub fn hrp(self) -> &'static str {
        match self { Self::Ms1 => "ms", Self::Mk1 => "mk", Self::Md1 => "md" }
    }
}

/// Per-chunk correction report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairDetail {
    pub chunk_index: usize,
    pub original_chunk: String,
    pub corrected_chunk: String,
    pub corrected_positions: Vec<(usize, char, char)>,
}

#[derive(Debug, Clone)]
pub struct RepairOutcome {
    pub kind: CardKind,
    pub corrected_chunks: Vec<String>,
    pub repairs: Vec<RepairDetail>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepairError {
    #[error("repair: no chunks supplied")]
    EmptyInput,
    #[error("repair: chunk {chunk_index} HRP mismatch — expected '{expected}', found '{found}' (HRP is not BCH-protected; re-type the prefix)")]
    HrpMismatch { chunk_index: usize, expected: &'static str, found: String },
    #[error("repair: chunk {chunk_index} has too many errors to correct uniquely (exceeds singleton bound = {bound}); cannot suggest correction")]
    TooManyErrors { chunk_index: usize, bound: usize },
    #[error("repair: chunk {chunk_index} parse failed before correction could run: {detail}")]
    UnparseableInput { chunk_index: usize, detail: String },
}

/// Per-chunk atomic per D8. Pure function, no I/O.
pub fn repair_card(kind: CardKind, chunks: &[String]) -> Result<RepairOutcome, RepairError>;

/// Auto-fire convenience wrapper. Per R2 C1 fold — does NOT take an
/// `original_err` parameter; the caller retains its typed `Err` value
/// and propagates it via `return Err(orig.into())` on the fall-through
/// branch (matching §2.3 inspect.rs:296 and §2.4 wire-up pattern).
///
/// Return semantics:
///   Ok(())                            — repair FAILED; caller falls through to its own error path
///   Err(RepairShortCircuit { 5 })     — repair SUCCEEDED; caller `?` propagates to exit 5
///
/// Generic trait bounds match existing codebase convention.
pub fn try_repair_and_short_circuit<O, E>(
    kind: CardKind,
    chunks: &[String],
    stdout: &mut O,
    stderr: &mut E,
) -> Result<(), ToolkitError>
where
    O: std::io::Write,
    E: std::io::Write,
{
    // Best-effort repair. If repair fails (HrpMismatch / TooManyErrors /
    // EmptyInput / UnparseableInput), return Ok(()) so caller falls
    // through to its own existing error handling with the ORIGINAL
    // typed error. We deliberately do NOT propagate the repair-side
    // error — the user's original decode error is strictly more
    // informative per §5 fallthrough discipline.
    let outcome = match repair_card(kind, chunks) {
        Ok(o) => o,
        Err(_repair_err) => return Ok(()),       // fall-through
    };
    emit_repair_report(&outcome, stdout, stderr)
        .map_err(ToolkitError::Io)?;
    // D9 sensitive-secret stderr warning fires inside emit_repair_report
    // (or in the standalone repair::run path; see §2.2). Short-circuit
    // propagation via the always-Err return on success branch.
    Err(ToolkitError::RepairShortCircuit { exit_code: 5 })
}

fn emit_repair_report<O: std::io::Write, E: std::io::Write>(
    outcome: &RepairOutcome,
    stdout: &mut O,
    stderr: &mut E,
) -> std::io::Result<()>;
```

**Note on the asymmetric Ok/Err semantics:** `Err` on repair-success enables `?` short-circuit (caller exits 5 cleanly via `ToolkitError::RepairShortCircuit`'s `exit_code()` mapping). `Ok(())` on repair-failure means "I couldn't help; you handle the original error your own way" — the caller follows with `return Err(orig.into())` to surface its typed decode error per pre-cycle UX. The `try_repair_and_short_circuit` body therefore never NEEDS `original_err` itself — the typed error stays with the caller for natural fall-through.

**Private submodules** (one per HRP — D7 + D11 + R0 C3 fold):

```rust
mod ms_checksum {
    use bech32::{Checksum, Fe32};
    pub(crate) struct MsCodex32;
    impl Checksum for MsCodex32 {
        type MidstateRepr = u128;
        const CHECKSUM_LENGTH: usize = 13;
        const CODE_LENGTH: usize = 93;
        const GENERATOR_SH: [u128; 5] = MS_GEN_REGULAR;
        const TARGET_RESIDUE: u128 = MS_NUMS_TARGET;
        const ROOT_GENERATOR: Fe32 = Fe32::S;             // R0 N6 + R1 verifies in Phase 0
        const ROOT_EXPONENTS: core::ops::RangeInclusive<usize> = 9..=16;
    }
    pub(crate) const MS_GEN_REGULAR: [u128; 5] = [/* BIP-93 standard generator polynomial */];
    pub(crate) const MS_NUMS_TARGET: u128 = 0x10ce0795c2fd1e62a;   // D11
}
mod mk_checksum {
    use bech32::{Checksum, Fe32};
    use mk_codec::MK_REGULAR_CONST;   // pub const at mk-codec/src/consts.rs:18 = 0x1062435f91072fa5c
    pub(crate) struct MkCodex32;
    impl Checksum for MkCodex32 {
        type MidstateRepr = u128;
        const CHECKSUM_LENGTH: usize = 13;
        const CODE_LENGTH: usize = 93;
        const GENERATOR_SH: [u128; 5] = MK_GEN_REGULAR;
        const TARGET_RESIDUE: u128 = MK_REGULAR_CONST;
        const ROOT_GENERATOR: Fe32 = Fe32::S;
        const ROOT_EXPONENTS: core::ops::RangeInclusive<usize> = 9..=16;
    }
    pub(crate) const MK_GEN_REGULAR: [u128; 5] = [/* mk-codec GEN_REGULAR; Phase 0 confirms public access */];
}
mod md_checksum {
    use bech32::{Checksum, Fe32};
    pub(crate) struct MdCodex32;
    impl Checksum for MdCodex32 {
        type MidstateRepr = u128;
        const CHECKSUM_LENGTH: usize = 13;
        const CODE_LENGTH: usize = 93;
        const GENERATOR_SH: [u128; 5] = MD_GEN_REGULAR;
        const TARGET_RESIDUE: u128 = MD_NUMS_TARGET;
        const ROOT_GENERATOR: Fe32 = Fe32::S;
        const ROOT_EXPONENTS: core::ops::RangeInclusive<usize> = 9..=16;
    }
    // md-codec's `bch` module is module-private (NOT in lib.rs pub-mod list per R0 C3); CANNOT import directly. Vendored value; drift-gate via #[cfg(test)] SHA-256 recomputation (see §4.1 cell 8).
    pub(crate) const MD_GEN_REGULAR: [u128; 5] = [/* same BIP-93 generator structure */];
    pub(crate) const MD_NUMS_TARGET: u128 = 0x0815c07747a3392e7;   // From md-codec/src/bch.rs:17 (Phase 0 R0 verifies)
}
```

### §2.2 New subcommand: `crates/mnemonic-toolkit/src/cmd/repair.rs`

```rust
use clap::Args;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct RepairArgs {
    #[arg(long, value_name = "MS1", group = "kind")]
    pub ms1: Option<String>,
    #[arg(long, value_name = "MK1", group = "kind")]
    pub mk1: Vec<String>,
    #[arg(long, value_name = "MD1", group = "kind")]
    pub md1: Vec<String>,
    #[arg(long)]
    pub json: bool,
}

// D6 / D13: standalone `repair` IGNORES --no-auto-repair (the whole point
// of this subcommand IS repair). Per R0 I2: omits the parameter; clap's
// global=true silently allows the user to pass --no-auto-repair anyway.
//
// Generic trait bounds per R1 C4 (match existing codebase convention).
pub fn run<R: Read, W: Write, E: Write>(
    args: &RepairArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError>
{
    let (kind, chunks) = resolve_kind_and_chunks(args, stdin)?;  // handles --ms1=- / --mk1=- / --md1=- stdin
    let outcome = repair::repair_card(kind, &chunks).map_err(ToolkitError::Repair)?;

    if args.json {
        emit_repair_json(&outcome, stdout)?;
    } else {
        emit_repair_text(&outcome, stdout)?;
    }

    // D9: emit sensitive-secret stderr warning when kind is ms1 AND
    // outcome includes corrected chunks. Per R1 I1, the helper is CREATED
    // in Phase 2 (`secret_advisory.rs::secret_on_stdout_warning`) with
    // `()` return signature matching `secret_in_argv_warning` (errors
    // silently swallowed per existing convention).
    if matches!(kind, CardKind::Ms1) && !outcome.corrected_chunks.is_empty() {
        crate::secret_advisory::secret_on_stdout_warning(kind, stderr);
    }

    Ok(if outcome.repairs.is_empty() { 0 } else { 5 })
}
```

**Stdout text-form:** `# Repair report\n#   ms1 chunk 0: 1 correction at position 47: '8' -> 'f'\nms10entrsq...` (corrected chunks one per line after comment lines).

**Stdout JSON-form (schema_version=1):** `{"schema_version":"1","kind":"ms1","corrected_chunks":[...],"repairs":[{"chunk_index":0,"corrected_positions":[{"position":47,"was":"8","now":"f"}],"original_chunk":"...","corrected_chunk":"..."}]}`.

### §2.3 New subcommand: `crates/mnemonic-toolkit/src/cmd/inspect.rs`

```rust
use clap::Args;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct InspectArgs {
    #[arg(long, value_name = "MS1", group = "kind")] pub ms1: Option<String>,
    #[arg(long, value_name = "MK1", group = "kind")] pub mk1: Vec<String>,
    #[arg(long, value_name = "MD1", group = "kind")] pub md1: Vec<String>,
    #[arg(long)] pub json: bool,
    /// Reveal entropy hex on ms1 inspection (default: print length + bit-strength only).
    #[arg(long)] pub reveal_secret: bool,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &InspectArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError>
{
    let (kind, chunks) = resolve_kind_and_chunks(args, stdin)?;
    let chunks_ref: Vec<&str> = chunks.iter().map(String::as_str).collect();

    // Per R0 C1 fold: actual sibling-codec public APIs:
    //   ms_codec::decode(s: &str) -> Result<(Tag, Payload)>
    //   mk_codec::decode(strings: &[&str]) -> Result<KeyCard>
    //   md_codec::chunk::reassemble(strings: &[&str]) -> Result<Descriptor>
    let decoded = match kind {
        CardKind::Ms1 => ms_codec::decode(chunks_ref[0]).map(InspectPayload::Ms1)
                                                       .map_err(ToolkitError::MsCodec),
        CardKind::Mk1 => mk_codec::decode(&chunks_ref).map(InspectPayload::Mk1)
                                                     .map_err(ToolkitError::MkCodec),
        CardKind::Md1 => md_codec::chunk::reassemble(&chunks_ref).map(InspectPayload::Md1)
                                                                .map_err(ToolkitError::MdCodec),
    };

    let payload = match decoded {
        Ok(p) => p,
        Err(orig) => {
            // Auto-fire short-circuit (item #11 in §2.4 wire-up list).
            if !no_auto_repair {
                // try_repair_and_short_circuit is always-Err on success;
                // `?` propagates ToolkitError::RepairShortCircuit (exit 5).
                // Returns Ok(()) when repair fails — falls through below
                // to surface the typed original decode error per §5.
                repair::try_repair_and_short_circuit(kind, &chunks, stdout, stderr)?;
            }
            return Err(orig);  // fall-through preserves typed original error per §5
        }
    };

    if args.json { emit_inspect_json(&payload, args.reveal_secret, stdout)?; }
    else         { emit_inspect_text(&payload, args.reveal_secret, stdout)?; }
    Ok(0)
}
```

### §2.4 Auto-fire integration sites (10 in existing code; 11th in new `cmd/inspect.rs`)

Per R0 C2/C6/C7 + R1 C5 folds. Source-grep confirmed against current `master` via `grep -n 'ms_codec::decode\|mk_codec::decode\|md_codec::chunk::reassemble' crates/mnemonic-toolkit/src/cmd/`:

| # | File:line | Enclosing function | Decode call | Kind |
|---|---|---|---|---|
| 1 | `cmd/verify_bundle.rs:887` | `emit_verify_checks` (single-sig branch) | `ms_codec::decode(supplied_ms1)` | Ms1 |
| 2 | `cmd/verify_bundle.rs:937` | `emit_verify_checks` (single-sig branch) | `mk_codec::decode(&mk1_strs)` | Mk1 |
| 3 | `cmd/verify_bundle.rs:1096` | `emit_multisig_checks` (per-cosigner grouping; `.ok()`-mapped) | `mk_codec::decode(&strs)` | Mk1 |
| 4 | `cmd/verify_bundle.rs:1101` | `emit_multisig_checks` (per-cosigner grouping; `.ok()`-mapped) | `mk_codec::decode(&strs)` | Mk1 |
| 5 | `cmd/verify_bundle.rs:1122` | `emit_multisig_checks` (per-group decode) | `mk_codec::decode(g)` | Mk1 |
| 6 | `cmd/verify_bundle.rs:1127` | `emit_multisig_checks` (supplied-md1; drives positional-fallback flag — R1 C5) | `md_codec::chunk::reassemble(&supplied_md1_strs)` | Md1 |
| 7 | `cmd/verify_bundle.rs:1219` | `emit_multisig_checks` (per-cosigner ms1) | `ms_codec::decode(s)` | Ms1 |
| 8 | `cmd/verify_bundle.rs:1532` | `emit_md1_checks` (shared md1 supplied decode) | `md_codec::chunk::reassemble(&supplied_md1)` | Md1 |
| 9 | `cmd/convert.rs:1268` | `compute_outputs` (`--from ms1=<s>`) | `ms_codec::decode(s)` | Ms1 |
| 10 | `cmd/convert.rs:1307` | `compute_outputs` (`--from mk1=<s>`) | `mk_codec::decode(&[s])` | Mk1 |
| 11 | `cmd/inspect.rs::run` (new) | `inspect::run` | per-kind decode | per-kind |

**EXCLUDED from auto-fire scope** (per R0 C2/C7 + R1 C5):

- `verify_bundle.rs:981/1421/1550` — EXPECTED-side decodes (`.expect(...)`); repair would corrupt deterministically-synthesized expected bundle.
- `cmd/convert.rs --from md1=<s>` — does NOT EXIST (`NodeType` has no `Md1` variant).
- `cmd/bundle.rs::self_check_bundle` (lines 1420/1440/1469) — toolkit-synthesized chunks per D16.

**Uniform short-circuit pattern via `?` propagation (per D17 helper refactor):**

```rust
// Inside emit_verify_checks / emit_multisig_checks / emit_md1_checks
// (signature now: Result<Vec<VerifyCheck>, ToolkitError> per D17):
match codec::decode(&s) {
    Ok(payload) => /* normal flow */,
    Err(orig) => {
        if !no_auto_repair {
            // try_repair_and_short_circuit is always-Err on success;
            // `?` propagates ToolkitError::RepairShortCircuit (exit_code: 5) up to run().
            // Returns Ok(()) when repair fails (HrpMismatch / TooManyErrors / etc.);
            // falls through to existing error path with typed `orig` intact.
            repair::try_repair_and_short_circuit(kind, &[s.to_string()], stdout, stderr)?;
        }
        // Fall-through: orig is in scope per §5 fallthrough discipline.
        // For verify_bundle helpers: emit VerifyCheck { passed: false, decode_error: Some(format!("{:?}", orig)) }
        // and continue collecting other checks (existing behavior).
        // For convert::compute_outputs: return Err(orig.into()) per existing semantics.
    }
}
```

**Helper signature changes** (per D17 + R1 C1):

- `pub fn emit_verify_checks(...) -> Result<Vec<VerifyCheck>, ToolkitError>` (was: `Vec<VerifyCheck>`)
- `fn emit_multisig_checks(...) -> Result<Vec<VerifyCheck>, ToolkitError>` (was: `Vec<VerifyCheck>`)
- `fn emit_md1_checks(..., checks: &mut Vec<VerifyCheck>) -> Result<(), ToolkitError>` (was: returns unit; modifies `checks`)
- Convert.rs `compute_outputs` signature UNCHANGED — already returns `Result<ComputeOutputsResult, ToolkitError>`; `?` works natively.

At `run()` boundary in verify_bundle.rs:

```rust
let checks = match emit_verify_checks(...) {
    Ok(checks) => checks,
    Err(ToolkitError::RepairShortCircuit { exit_code }) => return Ok(exit_code),  // 5
    Err(e) => return Err(e),
};
// ... existing flow continues
```

`compute_outputs` callers in convert.rs use `?` natively; the new `RepairShortCircuit` variant propagates to `convert::run` whose `Result<u8, ToolkitError>` return type carries the exit code via `error.rs::exit_code()` → 5.

### §2.5 Global `--no-auto-repair` flag + per-fn propagation

In `crates/mnemonic-toolkit/src/main.rs::Cli` (currently at line 29-38, has NO global flags):

```rust
#[derive(Parser, Debug)]
#[command(name = "mnemonic", about = "...", version)]
struct Cli {
    /// Skip auto-fire repair on decode failures; preserve pre-v0.22 exit policy.
    #[arg(long, global = true)]
    no_auto_repair: bool,
    #[command(subcommand)]
    command: Command,
}
```

Propagation per R0 I6 fold (5th positional param; no `RunContext` refactor):

```rust
let result: Result<u8, ToolkitError> = match &cli.command {
    Command::Bundle(args)        => cmd::bundle::run(args, stdin, stdout, stderr, cli.no_auto_repair).map(|_| 0),
    Command::VerifyBundle(args)  => cmd::verify_bundle::run(args, stdin, stdout, stderr, cli.no_auto_repair),
    Command::Convert(args)       => cmd::convert::run(args, stdin, stdout, stderr, cli.no_auto_repair),
    Command::ExportWallet(args)  => cmd::export_wallet::run(args, stdout, stderr).map(|_| 0),
    Command::DeriveChild(args)   => cmd::derive_child::run(args, stdin, stdout, stderr).map(|_| 0),
    Command::FinalWord(args)     => cmd::final_word::run(args, stdin, stdout, stderr),
    Command::SeedXor(args)       => cmd::seed_xor::run(args, stdin, stdout, stderr),
    Command::Slip39(args)        => cmd::slip39::run(args, stdin, stdout, stderr),
    Command::GuiSchema(args)     => { /* unchanged */ },
    Command::Repair(args)        => cmd::repair::run(args, stdin, stdout, stderr),                    // D6: ignores no_auto_repair
    Command::Inspect(args)       => cmd::inspect::run(args, stdin, stdout, stderr, cli.no_auto_repair),
};
```

Per R1 I2 note: `bundle::run` currently returns `Result<(), ToolkitError>` (per `bundle.rs:132`); the 5th `no_auto_repair: bool` parameter is added for signature consistency but the body uses `let _ = no_auto_repair;` per D16 (self_check excluded). `verify_bundle::run` / `convert::run` already return `Result<u8, ToolkitError>` so propagation is native.

### §2.6 New exit code + new `ToolkitError` variant

Per R0 I9 + R1 C1: extend `src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ToolkitError {
    // ... existing variants ...
    #[error("repair: {0}")]
    Repair(#[from] crate::repair::RepairError),
    #[error("repair short-circuit (exit {exit_code})")]
    RepairShortCircuit { exit_code: u8 },
}

impl ToolkitError {
    pub fn exit_code(&self) -> u8 {
        match self {
            // ... existing branches ...
            ToolkitError::Repair(_) => 2,   // user-input class
            ToolkitError::RepairShortCircuit { exit_code } => *exit_code,
        }
    }
}
```

`main.rs` dispatch maps `ToolkitError` to exit code via `e.exit_code()` (per main.rs:103). **Per R2 I1 fold**, the dispatch must SPECIAL-CASE `RepairShortCircuit` to suppress the stale `writeln!(io::stderr(), "{}", e)` at main.rs:101 — otherwise the user sees a confusing trailing `repair short-circuit (exit 5)` Display noise after `emit_repair_report`'s clean stderr report, violating D3 ("stderr = error + correction" — nothing else). Phase 5 step (j) edits main.rs:98-104:

```rust
let exit = match result {
    Ok(code) => ExitCode::from(code),
    // R2 I1: short-circuit fires CLEAN repair report on stderr inside the helper;
    // do NOT also emit the ToolkitError Display impl (which would tack on
    // "repair short-circuit (exit 5)" noise).
    Err(ToolkitError::RepairShortCircuit { exit_code }) => ExitCode::from(exit_code),
    Err(e) => {
        let _ = writeln!(io::stderr(), "{}", e);
        ExitCode::from(e.exit_code())
    }
};
```

The new `RepairShortCircuit` variant propagates `5` via either the special-case OR (theoretically) the generic Err branch; the special-case ensures stderr stays clean. Documented in:
- Each new subcommand's `--help` epilog.
- The new manual chapters (`42-repair.md` + `43-inspect.md`) — exit-code matrix table.
- Inline comments at the auto-fire integration sites.

Existing exit codes (per `error.rs::ms_codec_exit_code` / `mk_codec_exit_code` / `md_codec_exit_code`): 0 success, 1 generic, 2 user-input, 3 reserved, 4 verify-bundle mismatch. **5 = NEW: `REPAIR_APPLIED`**.

### §2.7 New `Cargo.toml` dep

Per R1 C2 (Cargo.lock already pins `bech32 = "0.11.1"` transitively):

```toml
[dependencies]
bech32 = "=0.11.1"            # exact-pin matching existing transitive lock
```

Phase 1 R0 verifies via `cargo tree | grep bech32` that adding the direct dep doesn't trigger conflict / version-bump.

### §2.8 Manual chapter placement

Per D15:
- New `docs/manual/src/40-cli-reference/42-repair.md` (~200 LOC).
- New `docs/manual/src/40-cli-reference/43-inspect.md` (~200 LOC).
- Update `docs/manual/src/40-cli-reference/40-index.md`.
- Update `docs/manual/src/SUMMARY.md` (mdbook TOC).
- Update `docs/manual/src/60-appendices/61-glossary.md` (ms1/mk1/md1 entries cross-ref chapters 42/43).
- Update `docs/manual/src/40-cli-reference/41-mnemonic.md` closing paragraph (line ~399-418 from v0.21.0) to mention auto-fire interaction.

## §3 Phase decomposition

8 phases total. Each phase culminates in per-phase opus reviewer-loop (R0 → Rn if needed → 0C/0I before next phase). All dispatches use `feature-dev:code-reviewer` with `model: "opus"` per `[[feedback-opus-primary-review-agent]]`.

| Phase | Scope | Reviewer expectation |
|---|---|---|
| **0** | **Reconnaissance gate.** PREREQ: `cargo fetch` to populate local registry (R0 N7). Source-grep + cite line content for: (a) `rust-codex32 v0.1.0` at `~/.cargo/registry/src/index.crates.io-*/codex32-*/src/` — confirm `TARGET_RESIDUE = 0x10ce0795c2fd1e62a` per BIP-93; (b) `mk_codec::consts::{HRP, NUMS_DOMAIN, MK_REGULAR_CONST}` (confirmed: `MK_REGULAR_CONST = 0x1062435f91072fa5c` at `mk-codec/src/consts.rs:18`); (c) `md_codec::bch` NOT in `lib.rs pub mod` list (confirmed); vendor `MD_NUMS_TARGET = 0x0815c07747a3392e7`; (d) all 11 auto-fire sites from §2.4 table — re-grep + cite line content + verify line 1127 is supplied-md1 (R1 C5 disambiguated YES, distinct from 1532); (e) `rust-bech32 v0.11.1` `Corrector` + `Checksum` trait + `Fe32::S` const at `~/.cargo/registry/src/index.crates.io-*/bech32-0.11.1*/src/primitives/correction.rs` + `lib.rs` (R0 N6 verifies `Fe32::S` associated-const name); (f) `cargo tree | grep bech32` shows `0.11.1` already pulled in transitively; (g) `compute_outputs` callers in convert.rs (R1 N2) via `grep -n 'compute_outputs' cmd/convert.rs` — confirm only `run()` calls it (no test callers); (h) `crate::secret_advisory` module audit — confirm `secret_in_argv_warning` returns `()` (NOT `Result<()>`) so Phase 2 helper signature matches convention. Reproduce a corruption-induced decode failure under current v0.21.0 binary (flip 1 char in valid ms1 from v0.21.0 verification recipe; confirm exit 2 + decode-fail stderr). Document findings as `.v0_22_0-phase0-artifact.md`. NO code touched. | R0 (opus): all 8 cites verbatim; pre-fix decode-failure repro captured; line 1127 distinct from 1532 confirmed; `Fe32::S` verified; `secret_advisory` helper convention locked; bech32 tree-grep cited. |
| **1** | **`Cargo.toml` dep + 3 vendored `Checksum` impls + drift-gate tests + cross-impl parity smoke (R1 N1).** Add `bech32 = "=0.11.1"` to `crates/mnemonic-toolkit/Cargo.toml`. Create `crates/mnemonic-toolkit/src/repair.rs` SKELETON containing ONLY the 3 private submodules with vendored constants per Phase 0 lock. Drift-gate `#[cfg(test)]` cells: (a) `assert_eq!(mk_checksum::MK_GEN_REGULAR, mk_codec_public_gen_regular)` for mk; (b) `assert_eq!(ms_checksum::MS_NUMS_TARGET, bech32_string_to_residue(b"SECRETSHARE32"))`; (c) `assert_eq!(md_checksum::MD_NUMS_TARGET, sha256_top65_to_residue(b"shibbolethnums"))`; (d) **NEW R1 N1 parity smoke:** corrupt a known-valid mk1 chunk by 1 char, run both `repair_card(Mk1, &[corrupted])` AND `mk_codec::string_layer::bch_correct_regular(b"mk", &corrupted_bytes)`, assert identical correction (same byte at same position). If divergence found, file blocker FOLLOWUP before Phase 2. `cargo build` + `cargo test --workspace --all-features` green. | R0+R1 (opus): all 3 NUMS constants verbatim; drift-gate tests passing; mk-codec native parity smoke green; bech32 v0.11.1 exact-pin doesn't bump transitive. |
| **2** | **`repair.rs` core + unit tests.** Implement `CardKind`, `RepairDetail`, `RepairOutcome`, `RepairError`, `repair_card()`, `try_repair_and_short_circuit()` (always-Err on success per D17), `emit_repair_report()`. Per-chunk atomic semantics per D8. Stdout/stderr format per §2.1 + §2.2. Add `ToolkitError::{Repair(RepairError), RepairShortCircuit { exit_code: u8 }}` variants in `src/error.rs` + `exit_code()` branches (5 for RepairShortCircuit; 2 for Repair). **Create `crate::secret_advisory::secret_on_stdout_warning(kind: CardKind, stderr: &mut impl Write)` helper** (per R0 I3 + R1 I1) with `()` return matching `secret_in_argv_warning` convention. Add unit tests per §4.1 (8 cells). Full suite green. | R0+R1+R2 (opus, R0 I8 + R1): all 8 unit cells passing; per-chunk atomic semantics confirmed; D9 stderr warning fires correctly; always-Err return-shape verified at unit level; secret-memory audit cited. |
| **3** | **`mnemonic repair` subcommand.** Create `crates/mnemonic-toolkit/src/cmd/repair.rs` per §2.2. Wire into top-level CLI dispatch (`main.rs` `Command::Repair(RepairArgs)`). Add `--ms1=- / --mk1=- / --md1=-` stdin handling per existing `*-stdin` pattern (cite `cmd/convert.rs::resolve_stdin_value` for the canonical pattern). Generic trait bounds match existing convention (R1 C4). Add integration tests per §4.2 (6 cells). Update `--help` epilog to document exit-code matrix + the "ignores --no-auto-repair" behavior per D6/D13. Full suite green. | R0 (opus): 6 integration cells passing; text + JSON output byte-exact; stdin form mirrors existing `*-stdin` pattern; standalone repair confirmed-NOT to read `no_auto_repair`. |
| **4** | **`mnemonic inspect` subcommand.** Create `crates/mnemonic-toolkit/src/cmd/inspect.rs` per §2.3 with 5th-param `no_auto_repair: bool`. Wire into top-level CLI dispatch (`Command::Inspect(InspectArgs)`). Per-kind text + JSON output. Auto-fire repair on own decode-failure (item #11 in §2.4) reading `no_auto_repair`. Add integration tests per §4.3 (4 cells). Full suite green. | R0 (opus): per-kind output structure confirmed; --reveal-secret gate verified (default suppresses entropy hex); auto-fire short-circuit exits 5 per D3. |
| **5** | **Helper refactor + auto-fire wire-up + global flag (per R1 C1 architecture).** Sub-steps: (a) Refactor `emit_verify_checks` → `Result<Vec<VerifyCheck>, ToolkitError>`; (b) Refactor `emit_multisig_checks` → same; (c) Refactor `emit_md1_checks` → `Result<(), ToolkitError>` (still modifies `&mut Vec<VerifyCheck>`); (d) **Update ALL helper-callers** (per R2 C2 fold) — `emit_verify_checks` has 4 production callers at `verify_bundle.rs:283, 338, 420, 682` + 6 in-file unit-test callers at `verify_bundle.rs:1671, 1718, 1748, 1822, 1924, 1999` (the v0.21.0 `helper_multisig_*` tests per `[[feedback-r0-must-read-source-off-by-n]]`; each pattern `let checks = emit_verify_checks(...)` must become `let checks = emit_verify_checks(...).unwrap()` for tests OR `?`-propagate for production); `emit_multisig_checks` callers grep'd at verify_bundle.rs:862 (production) + any test sites — Phase 5 R0 enumerates; `emit_md1_checks` callers at verify_bundle.rs:967 + 1059 (production) + any test sites; (e) Add `no_auto_repair: bool` global flag to `Cli` per §2.5; (f) extend `cmd::bundle::run`, `cmd::verify_bundle::run`, `cmd::convert::run`, `cmd::inspect::run` signatures with 5th `no_auto_repair: bool` param; (g) propagate via main.rs dispatch; (h) insert short-circuit pattern at sites #1-#10 from §2.4 using `?` propagation per §2.4 final code block; (i) `bundle::run` accepts param but `let _ = no_auto_repair;` per D16; (j) **add main.rs special-case for `RepairShortCircuit` per R2 I1 fold** (see §2.6); (k) add integration tests per §4.4 (5 cells); (l) verify v0.21.0 + v0.20.0 regression cells still pass per §4.5. Full suite green + clippy clean. | R0+R1+R2+R3 (opus): all 10 short-circuit-insertion sites + all 10+ helper-caller sites quoted post-edit; uniform `?` pattern verified; helper signature changes traced through ALL callers (production + tests); `no_auto_repair` propagation traced; D16 (`bundle::run` accepts + doesn't wire) verified; main.rs `RepairShortCircuit` special-case verified to suppress trailing Display noise; existing regression cells confirmed green; per `[[feedback-verify-bundle-round-trip-per-phase-r0-scope]]` exercise round-trip with explicit matrix: clean v0.21.0 bundle → exit 0; 1-char-corrupted bundle → exit 5 (no trailing `repair short-circuit (exit 5)` noise on stderr); `--no-auto-repair` + corrupted → existing exit policy. |
| **6** | **Manual regeneration.** Create `docs/manual/src/40-cli-reference/42-repair.md` covering: subcommand overview, flag table, text + JSON output examples (byte-exact per `[[feedback-architect-must-run-prose-commands]]`), exit-code matrix, error-mode taxonomy from §5, --no-auto-repair interaction. Create `docs/manual/src/40-cli-reference/43-inspect.md` similarly. Update `docs/manual/src/40-cli-reference/40-index.md` + `docs/manual/src/SUMMARY.md`. Update glossary ms1/mk1/md1 entries. Update `41-mnemonic.md` post-inheritance paragraph (line ~399-418 from v0.21.0) to mention auto-fire. Run manual lint — all 6 sub-checks green. | R0 (opus): MUST paste actual stdout/stderr of `mnemonic repair --ms1 <corrupted>` + `mnemonic inspect --ms1 <valid>` + auto-fire repro per `[[feedback-architect-must-run-prose-commands]]`. Manual lint clean. |
| **7** | **Release.** Bump `crates/mnemonic-toolkit/Cargo.toml:3` to `0.22.0`. Bump `scripts/install.sh:32`. Run end-of-cycle verification per §8. **Phase 7a (toolkit-side):** commit explicit paths (no `git add -A` per CLAUDE.md); file 11 cycle-close FOLLOWUPs per §5.2; PAUSE before push per release-pause checkpoint (cite project-v0_21_0 precedent); tag `mnemonic-toolkit-v0.22.0`; push master + tag; `gh release create`. **Phase 7b (sibling-repo companions, per R0 I11 + R1 I3):** for cross-repo FOLLOWUPs #1/#2/#3/#5/#6, file COMPANION entries in 3 sibling repos (`mnemonic-secret`, `descriptor-mnemonic`, `mnemonic-key`) via doc-only commits on each sibling's `master` branch with cross-citing `Companion:` lines per CLAUDE.md mirror invariant. Filed AFTER toolkit tag push; no PR review required (FOLLOWUPS doc-only). | R0 (opus, end-of-cycle): single dispatch reading FULL diff. 0C/0I. Per `[[feedback-default-cargo-test-runs-sibling-dependent-tests]]`: `cargo test --workspace --all-features`. Verify Phase 7b sibling companions filed before declaring cycle closed. |

Estimated 12-18 opus dispatches total (Phase 5 takes 4 rounds; other phases 1-3). Cycle wall-clock: 3-4 days. LOC delta: ~800 toolkit (R1 C1 expansion: +80 LOC for helper refactor across callers) + ~400 manual + ~30 SPEC. Test corpus delta: **+23 cells**.

## §4 Test corpus

### §4.1 Unit tests in `src/repair.rs::mod tests` (8 cells)

1. **Happy-path per HRP (×3 sub-cells).** For each of `Ms1` / `Mk1` / `Md1`: encode known payload, flip 1 char at deterministic position, assert `repair_card` returns Ok with `corrected_positions = [(N, was, now)]`. Plus a `corrections_applied == 0` (already-valid input pass-through) sub-cell.
2. **t=4 boundary.** For one HRP, flip exactly 4 chars: assert Ok with 4 RepairDetails. Flip exactly 5: assert `Err(TooManyErrors { bound: 8 })`.
3. **HRP mismatch.** `repair_card(Ms1, &["mk1foo...".to_string()])` → `Err(HrpMismatch { expected: "ms", found: "mk", chunk_index: 0 })`. Same for unknown HRP `xs1foo...`.
4. **Multi-chunk all-valid mk1.** 3 valid mk1 chunks → Ok with empty `repairs` vec + pass-through chunks.
5. **Multi-chunk one-corrupted mk1.** Flip 1 char in chunk 1 of 3-chunk mk1 → Ok with 1 RepairDetail at `chunk_index: 1`; chunks[0] and chunks[2] unchanged.
6. **Multi-chunk atomic failure.** Corrupt chunk 1 irreparably AND chunk 2 reparably → `Err(TooManyErrors { chunk_index: 1, … })`; chunk 2's correction NOT applied (atomic per D8).
7. **EmptyInput.** `repair_card(Mk1, &[])` → `Err(EmptyInput)`.
8. **Cross-codec NUMS-target constancy.** Drift-gate cells:
   - `assert_eq!(MS_NUMS_TARGET, bech32_string_to_residue(b"SECRETSHARE32"))`
   - `assert_eq!(MK_NUMS_TARGET, mk_codec::MK_REGULAR_CONST)`
   - `assert_eq!(MD_NUMS_TARGET, sha256_top65_to_residue(b"shibbolethnums"))` (per R0 C3 fold path b)

### §4.2 Integration tests in `tests/cli_repair.rs` (6 cells)

9. **Standalone repair text-form ms1 happy-path.** `mnemonic repair --ms1 <1-error>` → exit 5 + stdout matches `# Repair report\n#   ms1 chunk 0: 1 correction at position N: 'X' -> 'Y'\n<corrected>$`.
10. **Standalone repair `--json` ms1.** Same input; JSON envelope shape per §2.2.
11. **Standalone repair already-valid input.** Exit 0 + stdout = input echo, no comment lines.
12. **Standalone repair unrepairable.** `mnemonic repair --ms1 <6-error>` → exit 2 + stderr matches `TooManyErrors` template.
13. **Standalone repair multi-chunk mk1.** 3 chunks, chunk 1 flipped → exit 5 + stdout emits all 3 with chunk 1 corrected.
14. **Stdin form.** `mnemonic repair --ms1=-` reads from stdin per existing `*-stdin` pattern.

### §4.3 Integration tests in `tests/cli_inspect.rs` (4 cells)

15. **Inspect ms1 happy-path.** `mnemonic inspect --ms1 <valid>` → exit 0 + text-form output with kind/tag/bit_strength.
16. **Inspect mk1 + md1 happy-paths.** Verify per-kind text-form output structure (combined cell).
17. **Inspect --reveal-secret gate on ms1.** Default suppresses entropy_hex; `--reveal-secret` emits it.
18. **Inspect auto-fire on bad input.** `mnemonic inspect --ms1 <bad>` → exit 5 + corrected ms1 on stdout.

### §4.4 Integration tests in `tests/cli_auto_repair.rs` (5 cells)

19. **convert auto-fire ms1.** `mnemonic convert --from ms1=<1-error> --to phrase` → exit 5 + corrected ms1 on stdout + stderr explains.
20. **convert auto-fire mk1.** Same shape for mk1.
21. **verify-bundle auto-fire.** `mnemonic verify-bundle --bundle-json <json-with-bad-mk1-chunk> ...` → exit 5; original verify output suppressed; repair output emitted.
22. **`--no-auto-repair` suppresses.** Same inputs as #19 with global flag set → exit code reverts to pre-cycle policy (exit 2 for convert; existing VerifyCheck fail for verify-bundle); stdout/stderr match pre-feature byte-exactly.
23. **bundle --self-check NOT auto-firing.** Per D16: synthetic corruption injected into freshly-emitted bundle → `bundle --self-check` returns `ToolkitError::BundleMismatch` per current UX (NOT exit 5).

### §4.5 Cross-cycle regression guards (Phase 5 verification)

Per R0 N5 fold:
- v0.21.0 cell `descriptor_mode_3_of_3_emits_per_slot_ms1_post_v0_21` (added in v0.21.0; at `tests/cli_verify_bundle_multi_cosigner_mk1.rs`) → unchanged + green.
- v0.20.0 cells 1-6 in `tests/cli_verify_bundle_multi_cosigner_mk1.rs` (cell 7 was v0.21.0; total 7 cells in file) → unchanged + green.

### §4.6 Manual lint (Phase 6)

6/6 sub-checks (markdownlint, cspell, lychee, flag-coverage, glossary-coverage, bidirectional-index).

**Total new: +23 cells.** Toolkit suite grows from 1055 (post-v0.21.0) to ~1078.

## §5 Risks & cycle-close FOLLOWUPs

### §5.1 Risks

- **R1 — `ms` HRP TARGET_RESIDUE drift.** D11 locks `0x10ce0795c2fd1e62a`. Phase 1 drift-gate test catches future drift.
- **R2 — `bech32 v0.11.1` API stability.** Direct dep exact-pin (per R1 C2 fix). Primitives-tier API has weaker semver guarantees. FOLLOWUP `bech32-correction-api-version-pin` tracks.
- **R3 — `no_auto_repair` propagation may miss a site.** 4 `run()` signatures + 11 short-circuit sites. Phase 5 R0+R1+R2+R3 explicitly traces propagation.
- **R4 — Manual regen byte-drift.** Phase 6 R0 MUST paste byte-exact stdout/stderr per `[[feedback-architect-must-run-prose-commands]]`.
- **R5 — Sibling-CLI lockstep expectation.** FOLLOWUPs #3-#5 deferred; release notes communicate.
- **R6 — Auto-fire silently changes verify-bundle UX.** `--no-auto-repair` opt-out exists; release notes call out. FOLLOWUP `verify-bundle-auto-fire-feature-flag-survey`.
- **R7 — Multi-chunk atomic-failure UX surprises users.** Documented in manual chapter 42.
- **R8 — md-codec `bch` module is module-private.** Drift-gate via `#[cfg(test)]` recomputation; cross-repo FOLLOWUP #2 tracks promotion.
- **R9 (NEW R1) — Helper refactor cascades.** `emit_verify_checks` / `emit_multisig_checks` / `emit_md1_checks` signature changes touch ALL callers within verify_bundle.rs + potentially test files calling helpers directly. Phase 5 R0 enumerates all caller sites; Phase 5 R1 re-verifies after refactor.
- **R10 (NEW R1) — Helper-internal `?` operator availability.** `?` requires the function's `Err` type to be convertible from the called function's `Err`. `ToolkitError::RepairShortCircuit { exit_code }` IS already convertible (via `From<ToolkitError> for ToolkitError = identity`). Phase 5 R0 confirms no error-type-coercion issues at the helper call sites.

### §5.2 Cycle-close FOLLOWUPs (Phase 7a — toolkit-side; Phase 7b — sibling companions)

11 entries total. Per R0 I11 + R1 I3 + CLAUDE.md mirror invariant, items #1, #2, #3, #5, #6 require COMPANION entries in sibling repos (`bg002h/mnemonic-secret`, `bg002h/descriptor-mnemonic`, `bg002h/mnemonic-key`):

1. `ms-codec-decode-with-correction-public-api` — cross-repo. Companion: `bg002h/mnemonic-secret`.
2. `md-codec-decode-with-correction-public-api` — cross-repo. Companion: `bg002h/descriptor-mnemonic`.
3. `ms-cli-repair-flag` — blocked on (1); cross-repo. Companion: `bg002h/mnemonic-secret`.
4. `mk-cli-repair-flag` — unblocked; companion: `bg002h/mnemonic-key`.
5. `md-cli-repair-flag` — blocked on (2); cross-repo. Companion: `bg002h/descriptor-mnemonic`.
6. `toolkit-repair-consume-native-codec-api` — cross-repo dep; awaits (1)+(2).
7. `hrp-correction-heuristics` — Levenshtein-1 over `{ms, mk, md}`.
8. ~~`repair-mk-codec-long-code-support` (D12)~~ — **SHIPPED in v0.22 cycle** (D12 revised 2026-05-17; long-code dispatch folded in).
9. `repair-json-short-circuit-output` (D14) — JSON envelope on auto-fire when `--json` context.
10. `bech32-correction-api-version-pin` (R2) — track bech32 v0.11→0.12.
11. `verify-bundle-auto-fire-feature-flag-survey` (R6) — survey default-on vs default-off.

Phase 7b execution: 3 separate sibling-repo clones (already present at `/scratch/code/shibboleth/{mnemonic-secret,descriptor-mnemonic,mnemonic-key}/`); for each cross-repo FOLLOWUP, append a companion entry to that sibling's `design/FOLLOWUPS.md` + commit directly to master + push. No PR required (doc-only). Commit message cites toolkit's FOLLOWUP ID.

## §6 Reviewer-loop expectations

| Phase | Reviewer | Expected rounds | Convergence target |
|---|---|---|---|
| 0 (recon) | opus | R0 (read-only) | All 8 cites verbatim; pre-fix decode-failure repro captured; line 1127 distinct from 1532 confirmed; `Fe32::S` verified; `secret_advisory` helper convention locked; bech32 tree-grep cited; `compute_outputs` callers enumerated |
| 1 (deps + checksum impls + parity smoke) | opus | R0+R1 | 0C/0I; 3 NUMS constants verbatim; drift-gate tests passing; mk-codec native parity smoke green; bech32 exact-pin doesn't bump transitive |
| 2 (repair.rs core) | opus | R0+R1+R2 (R0 I8 + R1 — new-domain bech32 v0.11 + t=4 boundary + always-Err return semantics) | 0C/0I; 8 unit cells passing; per-chunk atomic confirmed; D9 stderr warning correct; always-Err shape verified |
| 3 (`repair` subcommand) | opus | R0 | 0C/0I; 6 integration cells; text + JSON byte-exact; stdin form |
| 4 (`inspect` subcommand) | opus | R0 | 0C/0I; per-kind output; --reveal-secret gate; auto-fire short-circuit exits 5 |
| 5 (helper refactor + auto-fire wire-up + flag) | opus | **R0+R1+R2+R3** (R1 C1 architecture expansion: helper signature refactor + 10 wire-up sites + flag plumbing + cross-cycle regression; mirrors v0.21.0 4-round trajectory) | 0C/0I; 10 sites quoted post-edit; helper signature changes traced; `no_auto_repair` propagation traced; D16 verified; regression cells green; round-trip matrix exercised |
| 6 (manual regen) | opus | R0 | 0C/0I; byte-exact stdout pasted; manual lint clean |
| 7 (release) | opus | R0 (end-of-cycle) | 0C/0I; clippy clean; GH release notes accurate; 11 FOLLOWUPs filed; Phase 7b sibling companions filed |

All dispatches use `feature-dev:code-reviewer` with `model: "opus"` per `[[feedback-opus-primary-review-agent]]`.

## §7 Critical files

- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml` — version bump + `bech32 = "=0.11.1"` exact-pin
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs` — NEW (~250 LOC + tests)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/repair.rs` — NEW (~150 LOC)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/inspect.rs` — NEW (~200 LOC)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:856,1083,1526` — 3 helper signatures change to `Result<_, ToolkitError>`; 8 auto-fire sites at 887/937/1096/1101/1122/1127/1219/1532 (R1 C5 promoted 1127); **10+ helper-caller sites** (R2 C2): `emit_verify_checks` callers at 283/338/420/682 production + 1671/1718/1748/1822/1924/1999 in-file tests (the v0.21.0 `helper_multisig_*` cells per `[[feedback-r0-must-read-source-off-by-n]]`); `emit_multisig_checks` callers at 862 production + test sites enumerated Phase 5 R0; `emit_md1_checks` callers at 967, 1059 production + test sites enumerated Phase 5 R0
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/convert.rs:1268,1307` — 2 auto-fire sites inside `compute_outputs` (line 979); compute_outputs signature unchanged (already Result)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs::self_check_bundle` — EXCLUDED per D16; `bundle::run` adds 5th param passthrough
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/main.rs:29-95` — Cli + global `no_auto_repair` flag + dispatch propagation + 2 new subcommand registrations
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/error.rs:253` — 2 new `ToolkitError` variants + `exit_code()` branches
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/secret_advisory.rs` — NEW helper `secret_on_stdout_warning(kind, &mut impl Write)` (per R1 I1 — explicit creation)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_repair.rs` — NEW (6 cells per §4.2)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_inspect.rs` — NEW (4 cells per §4.3)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_auto_repair.rs` — NEW (5 cells per §4.4)
- `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/40-cli-reference/42-repair.md` — NEW (~200 LOC)
- `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/40-cli-reference/43-inspect.md` — NEW (~200 LOC)
- `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/40-cli-reference/40-index.md` — index update
- `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/SUMMARY.md` — 2 new chapter entries
- `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/60-appendices/61-glossary.md` — cross-refs
- `/scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh:32` — install-pin self-update
- `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md` — Phase 7a appends 11 entries
- `/scratch/code/shibboleth/mnemonic-secret/design/FOLLOWUPS.md` — Phase 7b: companions for items #1, #3
- `/scratch/code/shibboleth/descriptor-mnemonic/design/FOLLOWUPS.md` — Phase 7b: companions for items #2, #5
- `/scratch/code/shibboleth/mnemonic-key/design/FOLLOWUPS.md` — Phase 7b: companion for item #4

## §8 Verification (end-of-cycle, Phase 7)

After Phase 7 tagging. Recipes use `jq` for surgical JSON corruption per R0 I10:

1. `cargo test --workspace --all-features` → all green; ~1078 total (1055 + 23).
2. `cargo clippy --workspace --all-targets -- -D warnings` → clean.
3. **Standalone repair recipe** (jq-based, parameterized bech32-alphabet substitution):
   ```sh
   VALID_MS1=$(./target/release/mnemonic convert --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" --to ms1 | tail -1)
   LAST=${VALID_MS1: -1}
   case "$LAST" in q) NEW=p ;; *) NEW=q ;; esac
   BAD_MS1="${VALID_MS1%?}${NEW}"
   ./target/release/mnemonic repair --ms1 "$BAD_MS1" ; echo "exit=$?"
   # Expect: exit=5, stdout shows "# Repair report" + corrected ms1 matching $VALID_MS1
   ```
4. **Auto-fire verify-bundle recipe** (jq surgical corruption):
   ```sh
   ./target/release/mnemonic bundle --network mainnet --descriptor 'wpkh(@0)' \
     --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
     --json > /tmp/v022-bundle.json
   jq '.ms1[0] = (.ms1[0] | (.[:-1] + (if (.[-1:]) == "q" then "p" else "q" end)))' \
     /tmp/v022-bundle.json > /tmp/v022-bundle-corrupt.json
   ./target/release/mnemonic verify-bundle --network mainnet --descriptor 'wpkh(@0)' \
     --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
     --bundle-json /tmp/v022-bundle-corrupt.json ; echo "exit=$?"
   # Expect: exit=5
   ```
5. **`--no-auto-repair` suppression:**
   ```sh
   ./target/release/mnemonic --no-auto-repair verify-bundle --network mainnet \
     --descriptor 'wpkh(@0)' \
     --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
     --bundle-json /tmp/v022-bundle-corrupt.json ; echo "exit=$?"
   # Expect: pre-cycle exit policy
   ```
6. Manual lint: `make -C docs/manual lint ...` → 6/6 green.
7. GUI drift gate: `cd /scratch/code/shibboleth/mnemonic-gui && MNEMONIC_BIN=$(which mnemonic) cargo test --test canonicity_drift` → green.
8. Install-pin check fires at tag push → green.
9. **Phase 7b verification:** `git log -1 --format='%H' /scratch/code/shibboleth/{mnemonic-secret,descriptor-mnemonic,mnemonic-key}/design/FOLLOWUPS.md` shows fresh commits with toolkit FOLLOWUP-ID cites.

## §9 Rn fold log

**R0 (plan-doc, 2026-05-17, opus).** 7C + 11I + 7N. Verdict: FOLD-REQUIRED. See prior version. Highlights:
- C1: `decode_multi` doesn't exist → §2.3 uses real APIs.
- C2: convert.rs has no md1 → site count corrected.
- C3: md_codec MD_REGULAR_CONST private → vendored + `#[cfg(test)]` drift-gate.
- C4: ms-codec NUMS not in source → D11 lock today (codex32-standard).
- C5: RepairShortCircuit shape carries exit code; Vec<String> ownership.
- C6: verify_bundle.rs sites re-grounded; fictitious function names dropped.
- C7: self_check_bundle excluded → D16.
- I1-I11 + N1-N7: all folded per §9 table in prior plan-doc version.

**R1 (plan-doc, 2026-05-17, opus, agent `ac7bc97910e3a4141`).** 5C + 3I + 2N. Verdict: FOLD-REQUIRED.

| # | R1 Finding | Fold action |
|---|---|---|
| C1 | Auto-fire short-circuit pattern incompatible with 9 of 10 sites (enclosing functions return Vec<VerifyCheck>, not Result<u8, _>). | **User picked Option 1: refactor helpers.** D17 added: `emit_verify_checks` + `emit_multisig_checks` → `Result<Vec<VerifyCheck>, ToolkitError>`; `emit_md1_checks` → `Result<(), ToolkitError>` (still modifies `&mut Vec<VerifyCheck>`). New `ToolkitError::RepairShortCircuit { exit_code: u8 }` variant. `try_repair_and_short_circuit` returns ALWAYS-Err on success (Ok would mean "continue normal flow" — wrong semantic). `?` propagation works via the new ToolkitError variant. Phase 5 expanded scope; R0+R1+R2+R3 expected (mirrors v0.21.0). |
| C2 | `bech32 = "=0.11.0"` conflicts with transitive `0.11.1`. | §2.7 changed to `bech32 = "=0.11.1"`. Phase 0 row (f) added: `cargo tree | grep bech32` verification. |
| C3 | §6.1-§6.4 dead cross-references. | §4 reorganized into §4.1-§4.6 subsections with explicit cell enumeration. All cross-refs (§2.1, §3, §5) updated. |
| C4 | `dyn Write` / `dyn Read` inconsistent with codebase generic-trait convention. | All new `run()` signatures + `try_repair_and_short_circuit` rewritten to use generic `<R: Read, W: Write, E: Write>` trait bounds. |
| C5 | Line 1127 is distinct supplied-md1 site (drives positional-fallback), not duplicate of 1532. | §2.4 table promoted line 1127 to active site #6; total count: 8 in verify_bundle.rs + 2 in convert.rs + 1 in inspect = 11. §3 Phase 5 site count updated. §7 critical files list updated. |
| I1 | `secret_on_stdout_warning` doesn't exist; helper signature ambiguous. | §3 Phase 2 step explicitly CREATES helper; §2.2 / §2.3 code-blocks use `()` return matching `secret_in_argv_warning` convention. §10 ADDENDUM-4 dropped. |
| I2 | `bundle::run` returns `Result<()>` not `Result<u8>`; plan didn't note. | §3 Phase 5 step (b) now notes bundle's distinct return shape. §2.5 dispatcher comment cites. |
| I3 | Phase 7 cross-repo companion sequencing underspecified. | §3 Phase 7 split into 7a (toolkit-side) + 7b (sibling companions). §5.2 + §7 enumerate per-sibling-repo paths. §8 step 9 adds verification. |
| N1 | mk-codec native parity smoke would catch interim-cycle drift. | §3 Phase 1 R0 adds parity smoke cell. §4.1 cell 8 unchanged (drift-gates per-HRP NUMS targets); parity smoke is separate cell in Phase 1 (not §4). |
| N2 | Phase 0 should grep `compute_outputs` callers. | §3 Phase 0 row (g) added. |

**R2 (plan-doc, 2026-05-17, opus, agent `a9829c970a994e905`).** 2C + 2I + 0N. Verdict: FOLD-REQUIRED.

| # | R2 Finding | Fold action |
|---|---|---|
| C1 | `try_repair_and_short_circuit` body referenced undefined `original_err_to_toolkit_error` placeholder; `original_err: &OrigErr` parameter was unnecessary given §2.3 inspect.rs and §2.4 wire-up patterns explicitly retain typed `orig` in the caller via `return Err(orig.into())`. | §2.1 rewritten: drop `original_err` param; body returns `Ok(())` on repair-failure (caller falls through), `Err(RepairShortCircuit { 5 })` on repair-success. §2.3 + §2.4 caller patterns updated to drop the `&orig` argument; typed `orig` remains in scope for fall-through. Doc-comments rewritten. |
| C2 | §3 Phase 5 step (d) didn't enumerate helper-callers; v0.21.0 `helper_multisig_*` unit-tests at verify_bundle.rs:1671/1718/1748/1822/1924/1999 would break on signature change. | §3 Phase 5 step (d) expanded with explicit caller enumeration: `emit_verify_checks` 4 production callers (283/338/420/682) + 6 in-file unit tests (1671/1718/1748/1822/1924/1999); `emit_multisig_checks` + `emit_md1_checks` test-sites enumerated Phase 5 R0 grep. §7 critical-files updated with all 10+ caller anchors. |
| I1 | `RepairShortCircuit` ToolkitError variant's Display impl would print `repair short-circuit (exit 5)` to stderr AFTER the clean repair report — violates D3. | §2.6 + Phase 5 step (j) add main.rs special-case at line 98-104: `Err(ToolkitError::RepairShortCircuit { exit_code }) => ExitCode::from(exit_code)` BEFORE the generic `Err(e)` arm. Suppresses the trailing Display noise. |
| I2 | §10 ADDENDUM list incomplete (missing slots for Phase 0 rows f/g/h additions). | §10 added ADDENDUM-6 (bech32 tree-grep), ADDENDUM-7 (sibling-helper test-callers), ADDENDUM-8 (secret_advisory `()` convention). |

**R4 (Phase 0 reconnaissance discovery, 2026-05-17).** 1 Critical fold from Phase 0 execution.

**C1 (BLOCKER discovered during Phase 0):** rust-bech32 v0.11.1 (latest published; release date 2025-12-02) does NOT have `primitives/correction.rs` — the file exists ONLY in unreleased upstream master. Verified via direct source-read of every `.rs` file in the cached crate at `/home/bcg/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bech32-0.11.1/` (zero hits for `correction|Corrector|bch_errors|singleton_bound`). Crates.io API confirms max_version=0.11.1. The brainstorm research agent + R1 reviewer conflated GitHub master with the published crate.

**User-locked architecture pivot (Option C):** mk-codec promotes 4 internals from `pub(crate)` / `pub(super)` / `pub(in crate::string_layer)` to `pub`; toolkit's `repair.rs` consumes mk-codec's public BCH primitives directly. Avoids forking ~870 LOC of cryptographically-sensitive code into a 3rd location (per mk-codec's own bch_decode.rs header noting it was already forked from md-codec pending `mc-codex32` shared-crate extraction).

**mk-codec lockstep release shipped:** mk-codec v0.3.1 (`88bdb29`) published to crates.io 2026-05-17 with 4 visibility promotions:
- `string_layer/mod.rs:23` — `pub mod bch_decode` (was `pub(crate)`)
- `string_layer/bch_decode.rs:517` — `pub fn decode_regular_errors` (was `pub(super)`)
- `string_layer/bch_decode.rs:531` — `pub fn decode_long_errors` (was `pub(super)`)
- `string_layer/bch.rs:272` — `pub fn polymod_run` (was `pub(in crate::string_layer)`)

**Plan-doc supersedes:**
- §2.7: drop `bech32 = "=0.11.1"` direct dep; bump `mk-codec = "0.3.1"` (was `0.3.0`).
- §2.1 mod ms_checksum / mk_checksum / md_checksum private submodules REPLACED by direct calls to `mk_codec::string_layer::{bch, bch_decode}` with per-HRP target constants (MS_NUMS_TARGET = `0x10ce0795c2fd1e62a` vendored; MK_NUMS_TARGET = `mk_codec::MK_REGULAR_CONST`; MD_NUMS_TARGET = `0x0815c07747a3392e7` vendored).
- §3 Phase 1 scope: implement `repair.rs` using mk-codec public primitives (not Checksum impls on rust-bech32).
- §3 Phase 0.5 NEW (already shipped): mk-codec v0.3.1 lockstep release.

All other architecture details (D8 atomic multi-chunk; D9 secret-warning; D13 per-fn flag propagation; D17 helper-refactor for verify_bundle; D6/D16 self-check excluded; etc.) carry forward unchanged.

**R3 (plan-doc, 2026-05-17, opus, agent `ada1a721573acc6df`).** 0 Critical + 0 Important + 4 below-threshold Nice. Verdict: **READY-FOR-EXITPLANMODE** at R3 — superseded by R4 BLOCKER discovery during Phase 0 execution.

R3 confirmed all 4 R2 fold targets landed correctly:
- C1 signature rewrite: clean, consistent across §2.1, §2.3, §2.4; no stale `original_err_to_toolkit_error` references.
- C2 helper-caller enumeration: 6 test-cell sites at verify_bundle.rs:1671/1718/1748/1822/1924/1999 source-verified to exist + call `emit_verify_checks` directly.
- I1 main.rs special-case: §2.6 + Phase 5 step (j); ordering correct (special-case before generic Err).
- I2 ADDENDUMs 1-8 present, each maps to a Phase 0 row.

4 sub-threshold Nice observations (not folded; all confidence < 80):
- N1: Phase 5 4-round budget realistic vs v0.21.0 precedent.
- N2: `bundle::run` only called from main.rs:77 (no external callers); 5-arg breaking change internal-only.
- N3: D9 stderr warning emission from `emit_repair_report` is a Phase 2 implementation detail (R0 verifies); plan-doc adequately gestures at it.
- N4: Standalone `repair::run` returns `Ok(0|5)` directly (no `RepairShortCircuit`); intentional asymmetry with auto-fire path.

**Convergence achieved at R3.** Plan-doc is ready for user approval + Phase 0 execution.

## §10 ADDENDUMs (post-Phase-0 fill-ins)

After Phase 0 reconnaissance:

- **ADDENDUM-1:** `MS_NUMS_TARGET = 0x10ce0795c2fd1e62a` confirmed against upstream `rust-codex32` source.
- **ADDENDUM-2:** Line 1127 (supplied-md1 in `emit_multisig_checks`) distinct from line 1532 (in `emit_md1_checks`) — R1 C5 already disambiguated, Phase 0 verifies.
- **ADDENDUM-3:** `Fe32::S` const presence in bech32 v0.11.1 API verified (R0 N6).
- **ADDENDUM-4:** `mk_codec::GEN_REGULAR` public-export status verified (Phase 0 row b).
- **ADDENDUM-5:** `compute_outputs` callers in convert.rs (R1 N2) — confirm only `run()` calls it.
- **ADDENDUM-6 (R2 I2):** `cargo tree | grep bech32` output captured (Phase 0 row f) — confirm exact `0.11.1` transitive pin doesn't conflict with our direct `=0.11.1` exact-pin.
- **ADDENDUM-7 (R2 I2):** `emit_multisig_checks` + `emit_md1_checks` test-caller sites enumerated (Phase 5 R0 expansion of R2 C2 fold — `emit_verify_checks` already enumerated at verify_bundle.rs:283/338/420/682 production + 1671/1718/1748/1822/1924/1999 tests; sibling helpers need same enumeration).
- **ADDENDUM-8 (R2 I2):** `secret_advisory.rs::secret_in_argv_warning` convention confirmed — returns `()` (NOT `Result<()>`); errors silently swallowed. Phase 2 `secret_on_stdout_warning` matches this signature.

## §11 Next steps

1. R2 plan-doc dispatch (should converge tight — R1 folds are mostly mechanical).
2. R3 if R2 surfaces additional findings (mirrors v0.21.0 4-round trajectory).
3. ExitPlanMode at 0C/0I; user approval; execute phases.
