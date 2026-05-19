# Plan: `compare-cost` — wsh-vs-tr per-spending-condition cost comparison

**Target release**: `mnemonic-toolkit-v0.26.0` + `mnemonic-gui-v0.11.0` (lockstep).
**Plan-doc revision**: R3 (folds R0 + R1 + R2 opus architect-reviews).

## Context

`mnemonic-toolkit` currently emits and verifies engraving bundles but has no surface for *cost analysis*. Users assembling a multi-cosigner or timelocked policy have no way to ask "what does each way of spending this script actually cost on chain, and how does that change if I wrap it as Segwit v0 (`wsh`) versus Taproot (`tr`)?"

This cycle adds a new `mnemonic compare-cost` subcommand that takes a miniscript (or a descriptor with a miniscript inside it), enumerates every minimal satisfying assignment ("spending condition") of the script, and reports vbyte and sat-at-feerate cost for each condition under both wrapper choices, row-aligned. The output is a single plaintext table (default) or a JSON envelope (`--json`). A new `mnemonic-gui` panel is auto-derived from `gui-schema` in lockstep.

**Scope guardrails**:
- No policy compilation this cycle (`--policy` is deferred — user can compile a policy to a miniscript externally before invocation).
- No multi-leaf tr layout exploration: `tr(NUMS, {M})` is single-leaf by construction, so the comparison is unambiguous.
- No `--wallet` input this cycle (wallet-import cycle is brainstorming, not shipped; filed as FOLLOWUP).

---

## Locked-in decisions (from brainstorming + R0 fold)

- **Scope**: new top-level subcommand `mnemonic compare-cost` + auto-derived `mnemonic-gui` panel in lockstep. No `bundle` / `verify-bundle` auto-fire integration this cycle.
- **Input forms, one commit each (Phases 1–3 below)**:
  - `--miniscript <STR>`: bare miniscript fragment with abstract key labels (`pk(A)`, `pk(B)`, …) — cost is key-agnostic so we substitute deterministic dummy keys internally.
  - `--descriptor <STR>`: full descriptor. We strip the outermost wrapper (`wsh`, `sh(wsh(...))`, single-leaf `tr(IK, {M})`) to recover the inner miniscript M, then run the comparison on M.
  - stdin fallback when no input flag is given.
- **`--policy` and `--wallet` are FOLLOWUPs**, not in this cycle.
- **Multi-leaf tr(...) input is refused** with a structured error pointing the user at `--miniscript` "one leaf at a time".
- **Cost decomposition**: every minimal satisfying assignment is enumerated and gets its own row. Rows are aligned across wsh and tr columns by spending condition. Hard cap default `4096` (overridable with `--max-conditions <N>`); soft warn-trail threshold = `min(256, --max-conditions)`. Exceeding the hard cap is an exit-3 error (no truncation).
- **Cost metrics (columns)**: `wsh vB`, `tr vB`, `Δ vB`; `wsh sats`, `tr sats`, `Δ sats`. `Δ = tr − wsh` (negative means tr cheaper). Sats = `round(vbytes × feerate)`.
- **`--feerate <SATS_PER_VB>`**: `f64`, default `1.0`. Accepts decimals (1.5, 7.3, 32.7). `value_parser` upper bound at `10_000.0` sat/vB.
- **Output format**: plaintext aligned-column table on stdout by default; `--json` flag swaps to a JSON envelope.
- **GUI surface shape**: auto-derived from `gui-schema` JSON, no custom widgets.
- **Worktree strategy**: one worktree per parallelizable phase (Phases 2, 3, 4 can fan out concurrently once Phase 1 lands).

---

## SPEC

### §1 — Subcommand surface

```
mnemonic compare-cost [INPUT] [OPTIONS]

INPUT (exactly one of):
  --miniscript <STR>           bare miniscript fragment
  --descriptor <STR>           full descriptor (wsh, sh(wsh(…)), or single-leaf tr)
  (none, with stdin attached)  read input from stdin                        (Phase 3)

OPTIONS:
  --feerate <SATS_PER_VB>      sats-per-vbyte for the sats columns (f64; default 1.0; max 10000.0)
  --max-conditions <N>         single hard cap on enumerated conditions (default 4096; min 1)
                               When > 256, a soft warn-trail fires at 256 with note in `notes[]`.
  --json                       emit machine-readable JSON envelope on stdout
  -h, --help                   print help
```

Exit codes (toolkit convention):
- `0` — success (including `--feerate 0.0`, with a notes[] entry).
- `2` — input parse error (malformed miniscript / descriptor).
- `3` — input rejected by SPEC (multi-leaf tr; unsupported wrapper; too many conditions; degenerate miniscript with zero satisfactions).
- `64` — clap argument-parse error (mutual-exclusion violation; feerate out of range; non-numeric feerate).

### §2 — Input parsing and wrapper-stripping

| Input wrapper                             | Inner miniscript M extracted                     | Notes                                                                          |
| ----------------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------ |
| `--miniscript <M>`                        | M directly                                        | Parse as both `Miniscript<DefiniteDescriptorKey, Segwitv0>` and `Miniscript<DefiniteDescriptorKey, Tap>` (with rewriting per §2.1, key-substitution per §2.2). |
| `--descriptor wsh(M)`                     | M                                                | The wsh wrapper IS the comparison's wsh side; we still re-wrap for symmetry. |
| `--descriptor sh(wsh(M))`                 | M                                                | sh-wsh wrapper is stripped; comparison ignores p2sh-redeem-script overhead.  |
| `--descriptor tr(IK, {M})`                | M (single-leaf only)                              | Strip both `tr` wrapper and the internal key. **NUMS** used in re-wrapping; §5 emits a `notes[]` advisory when `IK ≠ NUMS` (per §2.3). |
| `--descriptor tr(IK, {M₁, M₂, …})`        | (refused)                                        | Multi-leaf tr is rejected with exit 3 + "use --miniscript one leaf at a time". |
| `--descriptor` with any other wrapper     | (refused)                                        | Exit 3 with `unsupported wrapper`.                                              |
| stdin (no flag)                           | First non-blank line; classify as miniscript or descriptor; apply rules above. | Auto-classify via parse attempt: miniscript-parse first, fall back to descriptor-parse on failure. |

#### §2.1 — Context rewriting (multi ↔ multi_a, sortedmulti ↔ sortedmulti_a)

`multi(k, …)` is valid only in Segwitv0 context; `multi_a(k, …)` is valid only in Tap context. To parse the same logical miniscript M in both contexts, we rewrite as needed:

| Input fragment                                | Segwitv0 form                | Tap form                       |
| --------------------------------------------- | ---------------------------- | ------------------------------ |
| `multi(k, P₁, P₂, …)`                         | `multi(k, P₁, P₂, …)` (as-is) | `multi_a(k, P₁, P₂, …)`         |
| `multi_a(k, P₁, P₂, …)`                       | `multi(k, P₁, P₂, …)`         | `multi_a(k, P₁, P₂, …)` (as-is) |
| `sortedmulti(k, P₁, P₂, …)`                   | `sortedmulti(k, …)` (as-is)   | `sortedmulti_a(k, …)`           |
| `sortedmulti_a(k, P₁, P₂, …)`                 | `sortedmulti(k, …)`           | `sortedmulti_a(k, …)` (as-is)   |

Pubkey type is also coerced: Segwitv0 uses compressed-secp 33-byte keys, Tap uses x-only 32-byte keys. The DummyKey set provides both.

If the rewrite still fails to parse in one context — e.g., a fragment with no analog in the other context (`combo(...)` is descriptor-only and rejected; `pk_h(...)` Tap-acceptance to be source-verified in Phase 1 P0 spike against `miniscript::miniscript::context::Tap::check_terminal_non_malleable`), or any other context-specific limitation — exit 3 with `compare-cost: miniscript valid in <ctx> only; cannot wrap as <other>: <error>`.

#### §2.2 — DefiniteDescriptorKey substitution

User-supplied abstract labels (`pk(A)`, `pk(B)`, `pk(C)`, …) are substituted with deterministic dummy keys. rust-miniscript v13's `Descriptor::plan(...)` API operates on `Descriptor<DefiniteDescriptorKey>` — wildcard-free descriptor keys with concrete fingerprints/paths/pubkeys. So substitution targets `DefiniteDescriptorKey`, not bare `bitcoin::PublicKey`.

**Derivation** (deterministic, unconditionally succeeds):

1. `scalar = sha256("compare-cost-dummy-key:{label}")` — 32 bytes.
2. `secret_key = bitcoin::secp256k1::SecretKey::from_slice(&scalar)` — succeeds for all but a measure-zero set of inputs (zero scalar or scalar ≥ curve order); if it fails, salt and re-hash with a 1-byte counter. ("`compare-cost-dummy-key:{label}:{counter}`")
3. `public_key = secret_key.public_key(&secp)` — always succeeds.
4. For Segwitv0: serialize compressed (33 bytes); for Tap: x-only (32 bytes).
5. Wrap as `DescriptorPublicKey::Single(SinglePub { key: SinglePubKey::FullKey(pk) | SinglePubKey::XOnly(xpk), origin: None })`, then `DefiniteDescriptorKey::new(dpk)` (returns `Result<Self, NonDefiniteKeyError>`; `Single` has no wildcards so `Err` is unreachable — `.expect("Single has no wildcards")` is the idiomatic call). Phase 1 must source-check the exact `SinglePubKey::{FullKey, XOnly}` variant names against `miniscript::descriptor::key` before locking the body.

Cost is key-agnostic in miniscript (signature size is constant per scheme), so the choice of dummy key does not affect output vbytes; the substitution exists only so `Descriptor::plan(...)` has concrete keys to work with.

If the input has **concrete hex pubkeys** (33-byte compressed or 32-byte x-only), they pass through unchanged via the same `DescriptorPublicKey::Single` wrapping — the comparison runs against the user's real keys. (Cost is identical either way.)

#### §2.3 — `tr(IK, {M})` IK ≠ NUMS handling

When `--descriptor tr(IK, {M})` is supplied with a real internal key `IK` (not the canonical NUMS x-only), the tool:
1. Strips the wrapper to recover M.
2. Re-wraps the tr side as `tr(NUMS, {M})` for the comparison.
3. Adds a `notes[]` advisory to the output: `"input had a non-NUMS internal key IK; this report compares script-path-only cost (tr modeled as tr(NUMS, {M})). Keyspend-via-IK costs ~58 vB total (under SIGHASH_DEFAULT) and is the cheapest spend if signing with IK is acceptable."`

The `~58 vB` figure: P2TR keyspend witness under SIGHASH_DEFAULT = 1-byte witness-stack-count + 1-byte sig-length-prefix + 64-byte Schnorr sig = 66 witness bytes; total vbytes = `(164 + 66 + 3) / 4 = 58` (per §4: includes the 41 vB SegWit input overhead). Tested in Phase 2.

The `NUMS_XONLY` literal used in re-wrapping is the BIP-341 H-point `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`. Phase 1 must source-check whether the toolkit already has this as a constant (search `MS_NUMS_TARGET`, `MD_NUMS_TARGET`, `NUMS_XONLY_HEX` per `wallet_export/bip388.rs`); if so, reuse; otherwise define `crate::cost::NUMS_XONLY` and cite the BIP-341 source.

### §3 — Per-spending-condition enumeration (algorithm)

The plan-doc's primary load-bearing algorithmic choice. **Substrate**: rust-miniscript v13's `Descriptor::plan<P>(self, provider: &P) -> Result<Plan, Self>` API on `Descriptor<DefiniteDescriptorKey>` (note: takes `self` by value, returns the descriptor back on failure — see §3.1 clone-per-iteration). Age/height satisfaction is supplied by the caller's `AssetProvider` implementation via `check_older(relative::LockTime) -> bool` and `check_after(absolute::LockTime) -> bool` — **not** as method-level arguments on `plan` itself. (The R1-cited `plan_at_age_and_height` does not exist in rust-miniscript v13.) The returned `Plan` exposes `witness_size() -> usize` (witness data byte count) — that's what §4 reads.

#### §3.1 — Enumeration loop

For a parsed miniscript M (after §2 rewriting and §2.2 substitution to `DefiniteDescriptorKey`):

```rust
let wsh_desc: Descriptor<DefiniteDescriptorKey> =
    Descriptor::Wsh(Wsh::new(m_segv0)?);
let tr_desc:  Descriptor<DefiniteDescriptorKey> =
    Descriptor::Tr(Tr::new(nums_xonly_definite_key, Some(TapTree::Leaf(Arc::new(m_tap))))?);

let assets = walk_ast_to_collect_assets(m);  // signers, preimages, abs/rel timelocks

// §3.3 eager combinatorial precheck — fail fast before enumeration:
let raw_size = 4_usize.checked_mul(
    2_usize.checked_pow((assets.signers.len() + assets.preimages.len()) as u32)
        .ok_or(CompareCostError::ConditionsTooMany)?
).ok_or(CompareCostError::ConditionsTooMany)?;
if raw_size > hard_cap { return Err(CompareCostError::ConditionsTooMany(raw_size, hard_cap)); }

let mut rows = Vec::new();
'outer: for cfg in enumerate_configurations(&assets) {
    let provider = SyntheticAssetProvider::from(&cfg);   // §3.4
    // `plan` consumes self; clone the invariant descriptor each iteration:
    let wsh_plan = wsh_desc.clone().plan(&provider);
    let tr_plan  = tr_desc.clone().plan(&provider);
    if let (Ok(wp), Ok(tp)) = (wsh_plan, tr_plan) {
        if is_minimal(&cfg, &assets, &wsh_desc, &tr_desc) {    // shrinkage check
            rows.push(Row { label: cfg.label(), wsh_witness_bytes: wp.witness_size(), tr_witness_bytes: tp.witness_size() });
            if rows.len() >= hard_cap { break 'outer; }
        }
    }
}
```

#### §3.2 — AST asset collection (signers, preimages, timelocks)

Walk the `Miniscript` AST collecting:
- `signers`: every `pk(K)`, `pk_k(K)`, `pk_h(K)` — these are signers that may be required.
- `preimages`: every `sha256(H)`, `hash256(H)`, `ripemd160(H)`, `hash160(H)` — these are preimage requirements.
- `absolute_timelocks`: every `after(N)` — block-height or median-time-past lock targets.
- `relative_timelocks`: every `older(N)` — relative-time lock targets.

#### §3.3 — Configuration enumeration

A **configuration** is `(signing_keys: Set<Pk>, known_preimages: Set<HashLock>, abs_timelock_satisfied: bool, rel_timelock_satisfied: bool)` representing "the user has these signatures and preimages; the spending transaction sets nLockTime / nSequence such that absolute / relative timelocks are satisfied (yes/no)".

Enumeration:
1. **Eager precheck** (per §3.1): `raw_size = 4 × 2^(|signers| + |preimages|)` — if `raw_size > hard_cap`, exit 3 immediately. (Prevents memory blow-up before the cap-after-minimality guard fires.)
2. Generate the power-set of `signers × known_preimages` (lazy iterator, not eager Vec).
3. For each `(K, P)`, iterate the 4 timelock states `{(abs=F,rel=F), (abs=T,rel=F), (abs=F,rel=T), (abs=T,rel=T)}`.
4. For each `(K, P, abs, rel)`, build a `SyntheticAssetProvider` (§3.4) and call `wsh_desc.plan(&provider)` and `tr_desc.plan(&provider)`. Skip if either fails.
5. **Minimality**: a configuration is minimal iff for every element `e ∈ (K ∪ P ∪ {abs_flag, rel_flag})`, the configuration `(K∖e, P∖e, …)` fails `plan()` on at least one side. Enforced by a one-pass shrinkage check.
6. Append minimal configurations to `rows`; break enumeration when `rows.len() ≥ hard_cap`.
7. When `rows.len() ≥ 256` (or `--max-conditions` if below 256), emit a `notes[]` soft-trail advisory: `"enumeration reached soft threshold; <count> conditions shown"`.

Step 1's precheck means: with default `hard_cap=4096`, the user must have `|signers| + |preimages| ≤ 10` (since `4 × 2^10 = 4096`). Larger policies must raise `--max-conditions` explicitly.

#### §3.4 — Why `Plan` is the substrate; `SyntheticAssetProvider` shape

rust-miniscript's `Plan` returns a *non-malleable, optimal* witness for a given `AssetProvider`'s assets. `Plan::witness_size() -> usize` is the byte count of the witness data for that satisfaction in that context (Segwitv0 or Tap), including:
- per-stack-item length prefixes
- stack-count varint
- scriptCode (wsh: serialized script on the stack) or tapscript + control block (tr)
- script-context-specific signature sizes (73 for ECDSA, 64 for Schnorr SIGHASH_DEFAULT)

(`Plan::satisfaction_weight()` returns weight units; per §4, we use `witness_size()` which is byte-count — equivalent to weight-units for witness data since each witness byte contributes 1 wu.)

`SyntheticAssetProvider` implements rust-miniscript's `AssetProvider` trait. Per docs.rs/miniscript/13 the required methods include `provider_lookup_ecdsa_sig`, `provider_lookup_tap_key_spend_sig`, `provider_lookup_tap_leaf_script_sig`, `provider_lookup_sha256`, `provider_lookup_hash256`, `provider_lookup_ripemd160`, `provider_lookup_hash160`, plus `check_older(relative::LockTime) -> bool` and `check_after(absolute::LockTime) -> bool` for timelocks. (Both are `LockTime` variants from `bitcoin` 0.32's `absolute` and `relative` modules — they are NOT `Sequence`.) Exact method names will be source-verified in Phase 1 P0 spike.

Our impl:
- Sigs: return `Some(dummy_sig)` iff the queried pubkey is in `cfg.signing_keys`. Dummy sig contents are irrelevant to byte-count (`Plan::witness_size` only uses lengths).
- Preimages: return `Some(zero-bytes)` iff the queried hash is in `cfg.known_preimages`.
- `check_older(_: relative::LockTime) -> bool`: return `cfg.rel_timelock_satisfied`.
- `check_after(_: absolute::LockTime) -> bool`: return `cfg.abs_timelock_satisfied`.

`Plan` returns the smallest non-malleable satisfaction for that asset configuration. We do not hand-roll any witness composition — rust-miniscript is authoritative. Phase 1 implementation is the §3.3 enumeration loop wrapping a `plan(&provider)` call on each pre-built descriptor.

`Descriptor<DefiniteDescriptorKey>::plan` is non-malleable by default; if the policy contains intentionally-malleable paths (rare in practice), `plan_mall` is the malleable variant — Phase 1 sticks to `plan` only.

#### §3.5 — Label format

Each minimal configuration is labeled compactly:
- Signers: letter labels in AST-left-to-right order (`A`, `B`, `C`, …) — if input was abstract; or short hex suffix (`A=0x02ab…cd`) if input was concrete.
- Preimages: `preimage(h₁)`, `preimage(h₂)` … in AST order.
- Timelocks: `older(144)` (relative), `after(h₁₄₄)` (block height absolute), `after(t₁₅₀₀…)` (time absolute).
- Joined with `+`: e.g., `A + B + older(144)`, `A + preimage(h₀)`.

### §4 — Cost computation

For each minimal configuration produced by §3:

```
let wsh_witness_bytes = wsh_plan.witness_size();           // includes scriptCode (the witnessScript)
let tr_witness_bytes  = tr_plan.witness_size();            // includes tapscript + control block

// 41 = 36-byte outpoint + 1-byte scriptSig-length-0 + 4-byte sequence;
// applies to every SegWit input regardless of wrapper. This makes absolute
// vB numbers match what Sparrow / Bitcoin Core / mempool fee-estimators show.
const SEGWIT_INPUT_BASE_WU: usize = 164;  // = 41 × 4

let wsh_total_weight = SEGWIT_INPUT_BASE_WU + wsh_witness_bytes;
let tr_total_weight  = SEGWIT_INPUT_BASE_WU + tr_witness_bytes;

let wsh_vbytes = (wsh_total_weight + 3) / 4;               // round up (BIP141 weight discipline)
let tr_vbytes  = (tr_total_weight  + 3) / 4;
let delta_vb   = (tr_vbytes as i64) - (wsh_vbytes as i64); // sign-preserving; constant overhead cancels

let wsh_sats   = (wsh_vbytes as f64 * feerate).round() as i64;
let tr_sats    = (tr_vbytes  as f64 * feerate).round() as i64;
let delta_sats = tr_sats - wsh_sats;
```

**Vbyte rounding note**: BIP141's true vbyte computation rounds once on total tx weight; per-condition `(W+3)/4` rounding can introduce ±1 vB drift in absolute numbers but the **Δ vB column is robust** (the rounding error is constant-offset). A `notes[]` entry surfaces this: `"per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct"`.

**`--feerate 0.0` handling**: sats columns are all zero; a `notes[]` entry: `"feerate is 0; sats columns are all zero"`.

### §5 — Output

**Plaintext table** (default; numbers below are **schematic**, computed for real in Phase 1 acceptance via the spike harness from `.spike-v0.4/`):

```
$ mnemonic compare-cost --miniscript 'or_b(pk(A), a:and_n(pk(B), older(144)))'

Input: or_b(pk(A), a:and_n(pk(B), older(144)))
Wrapper comparison: wsh(M)  vs  tr(NUMS, {M})
Feerate: 1.0 sat/vB

Condition          | wsh vB | tr vB | Δ vB | wsh sats | tr sats | Δ sats
-------------------|--------|-------|------|----------|---------|-------
A                  |   30   |  25   |  -5  |    30    |    25   |    -5
B + older(144)     |   34   |  30   |  -4  |    34    |    30   |    -4

(numbers schematic; verified in Phase 1)
```

Header lines: echo the extracted miniscript, the wsh-vs-tr wrapper comparison string, the active feerate. Body: aligned-column table. Right-align numeric columns. Sign-preserve the Δ columns. Trailing `notes[]` entries appear after a blank line.

**JSON envelope** (`--json`):

```json
{
  "schema_version": 1,
  "subcommand": "compare-cost",
  "input": { "form": "miniscript", "value": "or_b(pk(A), a:and_n(pk(B), older(144)))" },
  "extracted_miniscript": "or_b(pk(A), a:and_n(pk(B), older(144)))",
  "feerate_sat_per_vb": 1.0,
  "conditions": [
    { "label": "A",              "wsh_vbytes": 30, "tr_vbytes": 25, "delta_vbytes": -5,
                                  "wsh_sats":   30, "tr_sats":   25, "delta_sats":   -5 },
    { "label": "B + older(144)", "wsh_vbytes": 34, "tr_vbytes": 30, "delta_vbytes": -4,
                                  "wsh_sats":   34, "tr_sats":   30, "delta_sats":   -4 }
  ],
  "notes": []
}
```

`notes[]` carries advisory text. Likely entries:
- IK-not-NUMS advisory (§2.3)
- vbyte-rounding advisory (§4)
- feerate-zero advisory (§4)
- soft-cap-reached advisory: `"enumeration capped at 256 conditions; raise --max-conditions to see more"`
- concrete-keys advisory: `"input had concrete keys; cost is identical to the abstract case"`

The JSON envelope sets exit 0 on success even when notes are non-empty.

### §6 — `gui-schema` emission

`compare-cost` registers in `gui_schema.rs::build_schema()` (the hand-edited path that emits Predicates and Effects) as a Subcommand with:

- flags: `--miniscript` (string, optional), `--descriptor` (string, optional), `--feerate` (f64, default 1.0), `--max-conditions` (u32, optional), `--json` (bool).
- conditional rules: **exactly-one-of `{miniscript, descriptor, stdin-fallback}`** is encoded as a `ConditionalRule` group via Predicate `FlagPresent` + Effect `Visibility::Disabled` (matches the existing `--template` / `--descriptor` exclusivity pattern in `export-wallet`'s row in `build_schema()`).
- positionals: none.
- meta: empty (no template-driven UI).

**Schema version**: stay on v5 (the v0.24.0 grammar). No new Predicate or Effect needed.

**Per-phase R0 acceptance** (per `[[feedback-a0-recon-check-gui-schema-json]]`): Phase 1 must run `mnemonic gui-schema --json | jq '.subcommands[] | select(.name=="compare-cost")'` and verify the emitted shape, **not** trust that clap-derive's `CommandFactory` walk auto-derives the ConditionalRule emissions. The mutual-exclusion code path is hand-maintained inside `build_schema`.

### §7 — `mnemonic-gui` auto-derived panel

`mnemonic-gui` walks the gui-schema JSON and auto-derives the panel:
- one text field per string flag, one number field for `--feerate` (with decimal stepper) and `--max-conditions` (integer stepper), one checkbox for `--json`.
- a `Run` button that shells out to `mnemonic compare-cost ...`.
- an output text area showing the captured stdout (plaintext table). When `--json` is on, the area shows the JSON pretty-printed.

No new GUI widgets, no custom table renderer this cycle. Tests: ~6 cells.

### §8 — Manual chapter

A new `## mnemonic compare-cost` section is appended to `docs/manual/src/40-cli-reference/41-mnemonic.md` in the conventional `## mnemonic <subcommand>` slot. Includes:

- flag table
- **three worked examples** (one per Phase 1–3 input form) — **runnable shell commands with verified output**, per `[[feedback-architect-must-run-prose-commands]]`. Each example is generated by running the shipped binary in Phase 6 and pasting the actual output, not narrative prose.
- label-format key
- vbytes-vs-sats explanation (round-once-vs-round-per-condition caveat)
- exit codes table
- notes[] catalog with one-sentence explanations of each known advisory

The bidirectional flag-coverage check at `docs/manual/tests/lint.sh` must pass after the chapter is added.

### §9 — Errors and edge cases

| Condition                                                | Exit | Stderr message                                                                                |
| -------------------------------------------------------- | ---- | --------------------------------------------------------------------------------------------- |
| miniscript / descriptor parse failure                    | 2    | `compare-cost: parse error: <miniscript-error-chain>`                                          |
| descriptor wrapper not one of {wsh, sh-wsh, single-leaf tr} | 3    | `compare-cost: unsupported wrapper '<wrapper>'; supported: wsh, sh(wsh(..)), tr(IK, {M})`     |
| multi-leaf tr descriptor                                 | 3    | `compare-cost: multi-leaf tr() input; supply one leaf at a time via --miniscript`             |
| input miniscript not parseable in one context after rewrite | 3 | `compare-cost: miniscript valid in <ctx> only; cannot wrap as <other>: <error>`                |
| spending-condition power-set exceeds hard cap (pre-enum) | 3    | `compare-cost: spending conditions exceed --max-conditions cap (<n> > <cap>); raise the cap or simplify the policy` |
| `--max-conditions 0`                                     | 64   | clap validation (min 1).                                                                                            |
| miniscript has zero satisfying conditions (degenerate)   | 3    | `compare-cost: no satisfying conditions for this miniscript`                                  |
| `--feerate 0.0`                                          | 0    | sats columns are all zero; `notes[]`: "feerate is 0; sats columns will be 0".                 |
| `--feerate` < 0 or > 10000.0                             | 64   | clap validation (value_parser bounds).                                                         |
| `--feerate` non-numeric                                  | 64   | clap parse error.                                                                              |
| both `--miniscript` and `--descriptor` provided          | 64   | clap mutual-exclusion (`conflicts_with`).                                                      |

---

## Phased implementation plan

### Phase 0 — reconnaissance + plan-doc reviewer-loop (this phase)

Already done: 3 Explore agents mapped the surface; R0 opus architect-review folded into R1 plan-doc. R1 reviewer-loop fires after this rewrite.

### Phase 1 — `--miniscript` input + core engine + plaintext + JSON output

**Files added**:
- `crates/mnemonic-toolkit/src/cmd/compare_cost.rs` — clap args struct + `run(...)` dispatch
- `crates/mnemonic-toolkit/src/cost/mod.rs` — module root
- `crates/mnemonic-toolkit/src/cost/dummy_keys.rs` — deterministic dummy-key substitution producing `DefiniteDescriptorKey` (§2.2)
- `crates/mnemonic-toolkit/src/cost/context_rewrite.rs` — multi↔multi_a / sortedmulti↔sortedmulti_a rewriting (§2.1)
- `crates/mnemonic-toolkit/src/cost/enumerate.rs` — minimal-satisfying-configuration enumeration over the parsed `Miniscript` AST (§3)
- `crates/mnemonic-toolkit/src/cost/provider.rs` — `SyntheticAssetProvider` impl wrapping a configuration (§3.4)
- `crates/mnemonic-toolkit/src/cost/format.rs` — plaintext-table renderer + JSON envelope serializer (§5)

**Files modified**:
- `crates/mnemonic-toolkit/src/main.rs:54-78` — add `CompareCost(...)` variant to `Command`, dispatch in match arm
- `crates/mnemonic-toolkit/src/cmd/mod.rs` — `pub mod compare_cost;`
- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::build_schema()` — **explicit hand-edit** of the predicate-emission code path to add the `--miniscript`/`--descriptor`/stdin mutual-exclusion ConditionalRule group. (Per §6 / I6 fold: the schema walk auto-derives flag-shape via `CommandFactory` but not mutex Predicates.)
- `crates/mnemonic-toolkit/src/error.rs` — add `ToolkitError::CompareCost { kind: CompareCostError }` discriminator + display impls

**Tests**:
- `crates/mnemonic-toolkit/tests/cli_compare_cost.rs` — ~25 cells covering:
  - Smoke: `pk(A)`, `and_v(v:pk(A), pk(B))`, `or_b(pk(A), pk(B))`, `thresh(2, pk(A), pk(B), pk(C))`
  - Timelocks: `and_v(v:pk(A), older(144))`, `or_d(pk(A), and_v(v:pk(B), older(144)))`
  - Preimages: `and_v(v:pk(A), sha256(0000…0001))`
  - Wrappers: `or_i(pk(A), pk(B))` — if rust-miniscript v13 accepts `or_i` in Tap context (per the per-context Terminal grammar), verify 2 rows aligned; if Tap rejects it, this becomes a context-incompat exit-3 cell instead. Phase 1 P0 spike determines which branch this test takes.
  - Context-rewrite: `--miniscript 'multi(2, A, B, C)'` parses on both sides (after multi→multi_a rewrite)
  - Cap behaviors: soft-cap warn-trail, hard-cap exit-3
  - Output: plaintext column alignment, `--json` envelope schema validation, `--feerate` arithmetic (integer and decimal)
  - Errors: malformed miniscript → exit 2; context-incompat with no rewrite → exit 3; `--feerate 0.0` → notes entry; `--feerate -1.0` → exit 64
- Unit tests inline in each `src/cost/*.rs` module: ~25 cells (especially the enumerate.rs minimality check and the format.rs alignment logic)
- **Phase 1 P0 spike**: write a small harness in `crates/mnemonic-toolkit/tests/fixtures/compare_cost_known_vbytes.rs` listing ~10 hand-computed (miniscript, condition, expected_wsh_vb, expected_tr_vb) tuples derived against `python-bitcointx`/`embit` or the rust-miniscript `Plan::witness_size` API directly. Use these to compute the **schematic numbers** in §5 — replacing the placeholders with real values in the SPEC.

**Acceptance**:
- `cargo test -p mnemonic-toolkit --features=miniscript-compiler -- compare_cost` green (note: we may need to enable the `compiler` feature for some advanced miniscript parsing — verify in Phase 1; if not, drop the feature flag).
- `mnemonic compare-cost --help` renders.
- `mnemonic compare-cost --miniscript 'or_b(pk(A), pk(B))'` emits aligned table with rows matching hand-computed fixtures.
- `mnemonic gui-schema --json | jq '.subcommands[] | select(.name=="compare-cost")'` returns expected shape (per §6 R0 acceptance).

**Per-phase reviewer-loop**: opus `feature-dev:code-reviewer` on the diff. Specifically scope into: `cost/enumerate.rs` minimality check correctness; `cost/provider.rs` Provider trait impls; gui-schema build_schema edit. Iterate R0 → R1 until 0C/0I.

**Commit**: `feat(toolkit): compare-cost subcommand — --miniscript input, table + JSON output`

### Phase 2 — `--descriptor` input + wrapper stripping

**Files modified**:
- `crates/mnemonic-toolkit/src/cmd/compare_cost.rs` — add `--descriptor` flag; route to new wrapper-stripping helper
- `crates/mnemonic-toolkit/src/cost/mod.rs` — `pub fn strip_to_miniscript(descriptor: &Descriptor<DefiniteDescriptorKey>) -> Result<StrippedDescriptor, StripError>` returning `(miniscript_segv0, miniscript_tap, original_ik_was_nums: bool)`

**Reuses**:
- `crates/mnemonic-toolkit/src/parse_descriptor.rs::parse_descriptor` for the initial parse — feed an empty cosigner-keys-and-fingerprints slice, same pattern as `gui-schema --classify-descriptor`.

**Tests**:
- `crates/mnemonic-toolkit/tests/cli_compare_cost.rs` — +15 cells:
  - `--descriptor wsh(or_b(pk(A), pk(B)))` — extracts and compares (numbers match Phase 1's bare miniscript fixture)
  - `--descriptor sh(wsh(...))` — strips both wrappers (numbers identical to wsh case)
  - `--descriptor tr(NUMS, {pk(A)})` — single-leaf strips, no IK advisory note
  - `--descriptor tr(KEY, {pk(A)})` — single-leaf strips, **emits IK-not-NUMS advisory note** (§2.3); verify the `~17 vB keyspend` figure appears in `notes[]`
  - `--descriptor tr(KEY, {pk(A), pk(B)})` — multi-leaf refused (exit 3)
  - `--descriptor pkh(...)` — unsupported wrapper refused (exit 3)
  - Mutual exclusion: `--miniscript X --descriptor Y` → exit 64

**Commit**: `feat(toolkit): compare-cost — --descriptor input with wrapper stripping`

### Phase 3 — stdin fallback

**Files modified**:
- `crates/mnemonic-toolkit/src/cmd/compare_cost.rs` — when no input flag is given and `stdin.is_terminal() == false`, read first non-blank line from stdin; classify (miniscript-parse first, descriptor-parse on failure); route to appropriate Phase 1/2 path.
- Matches the existing convert / inspect / repair stdin-fallback pattern.

**Tests**:
- `crates/mnemonic-toolkit/tests/cli_compare_cost.rs` — +6 cells:
  - Piped miniscript via stdin → success
  - Piped descriptor via stdin → success
  - Piped malformed → exit 2 with helpful error
  - No flag + TTY-detected stdin → `compare-cost: no input; supply --miniscript or --descriptor` (exit 3)
  - `--miniscript X` + non-empty stdin → flag wins (stdin ignored, no error)

**Commit**: `feat(toolkit): compare-cost — stdin fallback`

### Phase 4 — `mnemonic-gui` lockstep (auto-derived panel)

**Files added** (in `mnemonic-gui` repo, separate worktree):
- `crates/mnemonic-gui/src/panels/compare_cost.rs` — ~120 LOC, auto-derived from the gui-schema JSON `compare-cost` subcommand.

**Files modified**:
- `pinned-upstream.toml` — bump pinned `mnemonic-toolkit` to v0.26.0 (after Phases 1–3 ship and toolkit is tagged).
- `crates/mnemonic-gui/Cargo.lock` — regenerate after pinned-upstream bump.
- `crates/mnemonic-gui/src/tabs.rs` (or equivalent) — register the new panel in the tab list.

**Tests** (kittest cells): 6 cells:
- happy-path per input form (miniscript / descriptor / stdin-pipe)
- mutual-exclusion enforcement (both flags filled → disabled run button)
- feerate decimal-input accepted; feerate negative rejected
- output-area JSON-mode toggle

**Worktree**: parallel-eligible with Phase 5 once gui-schema JSON is stable.

**Commit (gui repo)**: `feat(mnemonic-gui): compare-cost panel — auto-derived from gui-schema`

### Phase 5 — manual chapter

**Files modified**:
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — append `## mnemonic compare-cost` section per §8 above. Three runnable worked examples, verified by running the actual binary and pasting output.
- `docs/manual/tests/lint.sh` (if needed) — confirm bidirectional flag-coverage check still passes.

**Verification**: `make -C docs/manual lint MNEMONIC_BIN=...` green.

**Commit**: `docs(manual): mnemonic compare-cost chapter`

### Phase 6 — end-of-cycle architect review + release

**Holistic review**: dispatch holistic `feature-dev:code-reviewer` (opus) over the full diff for all worktrees merged to master. Iterate R0 → R1 until 0C/0I.

**Pre-release checklist**:
1. `cargo test --workspace` green (toolkit).
2. `cargo test --workspace` green (mnemonic-gui).
3. `mnemonic compare-cost --help` renders correctly.
4. `mnemonic gui-schema --json | jq '.subcommands[] | select(.name=="compare-cost")'` returns final shape; the GUI's snapshot test against this is green.
5. `make -C docs/manual lint MNEMONIC_BIN=...` green; manual chapter renders.
6. `.github/workflows/manual.yml` CI workflow renders chapter PDF correctly with the new toolkit binary.
7. **Lockstep checklist** (per `[[feedback-manual-gui-lockstep]]`, `[[project-v0-5-1-schema-mirror-v2-closed]]`, `[[project-v0-18-1-v0-7-2-b1-bugfix-closed]]`):
   - mnemonic-gui's `pinned-upstream.toml` line `[mnemonic-toolkit]` matches the soon-to-be-tagged toolkit version `v0.26.0`.
   - mnemonic-gui's `Cargo.lock` bump committed.
   - install-pin-check CI gate green pre-tag.
8. Bump `crates/mnemonic-toolkit/Cargo.toml` version → `0.26.0`.
9. Bump `mnemonic-gui/Cargo.toml` version → `0.11.0`.
10. Update toolkit `CHANGELOG.md` `### Added` with the compare-cost subcommand citation.
11. Update mnemonic-gui `CHANGELOG.md` `### Added` with compare-cost panel + toolkit-v0.26.0 lockstep.
12. File FOLLOWUPs surfaced during the cycle (likely: `--policy`, `--wallet`, WU column, bundle/verify-bundle auto-fire, custom egui table renderer).
13. Flip resolved FOLLOWUPS' `Status: open` → `Status: resolved` per `[[feedback-per-phase-agents-forget-followup-status-flip]]`.
14. Tag `mnemonic-toolkit-v0.26.0` and `mnemonic-gui-v0.11.0` in lockstep.

---

## Worktree strategy

| Worktree path                                  | Branch                                      | Phases       | Depends on                       |
| ---------------------------------------------- | ------------------------------------------- | ------------ | -------------------------------- |
| `/scratch/code/shibboleth/mt-compare-cost-p1`  | `compare-cost/p1-miniscript`                | Phase 1      | (none — first to land)           |
| `/scratch/code/shibboleth/mt-compare-cost-p2`  | `compare-cost/p2-descriptor`                | Phase 2      | Phase 1 merged to master         |
| `/scratch/code/shibboleth/mt-compare-cost-p3`  | `compare-cost/p3-stdin`                     | Phase 3      | Phase 1 merged to master         |
| `/scratch/code/shibboleth/mg-compare-cost-p4`  | `compare-cost/p4-gui` (mnemonic-gui repo)   | Phase 4      | Phases 1–3 merged + toolkit tagged |
| `/scratch/code/shibboleth/mt-compare-cost-p5`  | `compare-cost/p5-manual`                    | Phase 5      | Phases 1–3 merged to master      |

Phase 2 and Phase 3 are independent of each other (different files modified; only shared file is `compare_cost.rs` flag-parser — easy merge). They can fan out concurrently after Phase 1 lands.

Phase 4 (GUI) cannot start until the toolkit is tagged (so `pinned-upstream.toml` has a tag to point at). Phase 5 (manual) can start in parallel with Phase 4.

Phase 6 (release) is single-worktree on master after all merges land.

Worktrees are created at execution time via the `superpowers:using-git-worktrees` skill (parent confirms before invoking).

---

## Verification (end-to-end smoke recipe)

Once Phase 6 lands and the v0.26.0 tag fires:

```sh
# Phase 1 smoke
mnemonic compare-cost --miniscript 'or_b(pk(A), pk(B))'
mnemonic compare-cost --miniscript 'thresh(2, pk(A), pk(B), pk(C))' --feerate 25.0
mnemonic compare-cost --miniscript 'or_d(pk(A), and_v(v:pk(B), older(144)))' --json | jq .
mnemonic compare-cost --miniscript 'multi(2, A, B, C)'  # tests §2.1 multi→multi_a rewrite
mnemonic compare-cost --miniscript 'or_i(pk(A), pk(B))'  # tests selector-bit enumeration if Tap accepts or_i; exit 3 otherwise

# Phase 2 smoke
mnemonic compare-cost --descriptor 'wsh(or_b(pk(A), pk(B)))'
mnemonic compare-cost --descriptor 'sh(wsh(pk(A)))'
mnemonic compare-cost --descriptor 'tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0, {pk(A)})'  # tr NUMS, no advisory
mnemonic compare-cost --descriptor 'tr(02ab…cd, {pk(A)})'  # tr non-NUMS, advisory in notes[]
mnemonic compare-cost --descriptor 'tr(KEY, {pk(A), pk(B)})' ; echo $?  # 3, multi-leaf refused

# Phase 3 smoke
echo 'or_b(pk(A), pk(B))' | mnemonic compare-cost
echo 'wsh(or_b(pk(A), pk(B)))' | mnemonic compare-cost --json

# Phase 4 GUI smoke (manual)
mnemonic-gui  # navigate to compare-cost panel; paste a miniscript; click Run; verify table renders
```

Expected: aligned plaintext tables; row counts match minimal-satisfying-assignment count; JSON schema stable; exit codes per §9; numbers match the Phase 1 P0 spike fixtures.

---

## Risks & FOLLOWUPs

**R1 — Per-condition enumeration correctness**
We're relying on rust-miniscript's `Plan` API to produce non-malleable witnesses. Risk: if `Plan` returns malleable witnesses or doesn't track all satisfying-set permutations, our minimality check fires on stale data. Mitigation: Phase 1 P0 spike validates 10 hand-computed fixtures end-to-end before any further phase code is written.

**R2 — Context-rewrite asymmetry**
`pk_h(...)` Tap-context status is asserted-but-unverified; per the R2 review, `Tap::check_terminal_non_malleable` accepts everything but consensus-validity checks may still reject. Some inputs may hit the "context-incompat after rewrite" exit-3 path. Mitigation: Phase 1 P0 spike source-greps and tests `pk_h(...)`-in-Tap and updates the SPEC + R3 list (`compare-cost-pk_h-tap-rewrite` FOLLOWUP) accordingly.

**R3 — `--feerate` rounding edge cases**
`(vbytes as f64 * feerate).round()` can produce off-by-1 sats values at high feerates × large vbyte counts. Mitigation: stay in i64 arithmetic where possible; round-half-to-even (banker's rounding) is `f64::round`'s default which is round-half-away-from-zero. Document the rounding convention in §4.

**R4 — gui-schema build_schema hand-edit drift**
Per `[[feedback-a0-recon-check-gui-schema-json]]`, build_schema's ConditionalRule emission is hand-maintained. A future flag-shape change might not propagate into build_schema, drifting the gui-schema JSON. Mitigation: Phase 1 acceptance includes the `jq` shape-check. Phase 6 holistic-review includes a `gui-schema-shape-test.rs` snapshot if one doesn't already exist.

**FOLLOWUPs to file at cycle close** (anticipated):
- `compare-cost-policy-input` — reinstate `--policy` with separate per-wrapper compilation + tr-layout knob. Enables the `compiler` feature on miniscript dep.
- `compare-cost-wallet-input` — pick up `--wallet <PATH>` once the wallet-import cycle ships.
- `compare-cost-wu-column` — expose WU column behind `--wu` flag.
- `compare-cost-bundle-auto-fire` — auto-fire compare-cost from `bundle` / `verify-bundle` when the descriptor has multiple spending conditions.
- `compare-cost-custom-gui-table-widget` — native egui sortable table renderer instead of plaintext-in-text-area.
- `compare-cost-pk_h-tap-rewrite` — investigate whether `pk_h` can be rewritten for Tap context (likely needs a new code path inside miniscript).

---

## References

- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — flag-rich subcommand precedent (multi-input, mutually-exclusive `--template` vs `--descriptor`).
- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::build_schema` (around line 1255+) — schema-emission walk; hand-edit target for the mutual-exclusion ConditionalRule.
- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::run` — `--classify-descriptor` diagnostic precedent (terse-text-output subcommand variant).
- `crates/mnemonic-toolkit/src/parse_descriptor.rs::parse_descriptor` — descriptor parser entry-point reused in Phase 2 wrapper-stripping.
- `crates/mnemonic-toolkit/src/cmd/repair.rs` + `inspect.rs` — stdin-fallback pattern reused in Phase 3.
- `crates/mnemonic-toolkit/src/main.rs:54-78` — `Command` enum insertion point.
- `crates/mnemonic-toolkit/Cargo.toml:24` — miniscript v13 pinned (compiler feature NOT enabled this cycle).
- `.spike-v0.4/src/spike1_taptree.rs` — tapleaf enumeration spike; `TapTree::leaves()` walk informs Phase 1 P0 hand-fixture generation but is **not** the runtime substrate (we use `Plan::witness_size()` directly).
- rust-miniscript v13 API: `Descriptor<DefiniteDescriptorKey>::plan(&P) -> Result<Plan, Self>`, `plan_mall` (malleable variant), `Plan::witness_size() -> usize`, `Plan::satisfaction_weight() -> Weight`, `AssetProvider` trait (with `provider_lookup_*` + `check_older` + `check_after`), `DefiniteDescriptorKey`, `DescriptorPublicKey::Single`. (Note: `plan_at_age_and_height` is NOT in v13's API; age/height satisfaction is provided through `AssetProvider::check_older/check_after`.)
- Memory: `[[feedback-opus-primary-review-agent]]`, `[[feedback-r0-must-read-source-off-by-n]]`, `[[feedback-architect-must-run-prose-commands]]`, `[[feedback-verify-bundle-round-trip-per-phase-r0-scope]]`, `[[feedback-manual-gui-lockstep]]`, `[[feedback-a0-recon-check-gui-schema-json]]`, `[[feedback-per-phase-agents-forget-followup-status-flip]]`.
