# R2 scoped check: IMPLEMENTATION_PLAN_network_rule_tk.md (plan r2, fold commit cd8647bf)

Scope: did r2 fix R1-I1 and R1-I2, and did those two fixes or the step 8a-0
`--account` proposal introduce a new Critical or Important. No fresh audit.

Counts: **0 Critical, 0 Important, 2 Minor, 0 Nit.** Verdict: **GREEN.**

## R1-I1: bsms first-address audit used the placeholder network. FIXED.

r2 moves the audit out of the parser into
`wallet_import::bsms::first_address_audit(p: &ParsedImport, run: bitcoin::Network, stderr)`,
which runs after the central network check. It covers both shapes that carry an
audit: the 6-line shape (`BsmsSixLine`) and the 4-line shape (`bsms.rs:116-140`,
where `first_address = lines[3]`). The new test
`nr3_bsms_first_address_audit_uses_run_network` holds it. A18 files the declared
first address as an unlisted address source in a FOLLOWUP, which is the honest
scope. The guard rewording to an internal `BadInput` bug line resolves R1-M5.

## R1-I2: TX-V would print on an ms1-only failure. FIXED.

r2 defines TX-V by: (a) the run was not refused, (b) at least one row failed,
(c) every failed row is one of `mk1_xpub_match`, `mk1_path_match`,
`md1_xpub_match`, `md1_origin_match`. It has a positive leg and a negative leg.
I re-ran the legs with `target/debug/mnemonic`:

| fixture | network | `--ms1` | rc | failed rows |
|---|---|---|---|---|
| `fx/vb_mainnet_coin5.txt` | mainnet | yes | 4 | exactly the 4 listed rows (positive leg holds) |
| `vb_signet` | signet | no | 4 | only `ms1_decode` (negative leg: no TX-V, correct) |
| `vb_signet` | mainnet | yes | 4 | the 4 listed rows |
| `vb_signet` | signet | yes | 0 | none |

The plan's claims match what the binary does.

## Step 8a-0 `--account` proposal: no Critical or Important.

The proposal has two parts:

- A typed `--account` beside `--origin` in single-sig restore becomes a
  `ModeViolation` (exit 2).
- `--expect-xpub` is compared against the key derived at `--origin`.

This is a new refusal. It is gated on Brian's ruling, it is the last code
commit, and it is separable, so it cannot block or contaminate units 0-8b. The
silent template completion is filed as a FOLLOWUP. That is acceptable.

## New Minors

**R2-M1. The TX-V row match must accept indexed multisig row names.**
`cmd/verify_bundle.rs` emits indexed names in multisig modes, for example
`format!("ms1_decode[{}]", i)` near `:2915`. Indexed forms exist for
`mk1_xpub_match[`, `mk1_path_match[`, `mk1_fingerprint_match[`, `mk1_decode[`,
`ms1_decode[` and `ms1_entropy_match[`. If condition (c) compares exact names,
TX-V can never print in multisig mode.

Fix: compare the base name with any `[i]` suffix removed. Add one multisig leg,
or a unit test of the predicate over `mk1_xpub_match[0]` plus `ms1_decode[1]`
(no TX-V) and `mk1_xpub_match[0]` plus `mk1_path_match[1]` (TX-V).

**R2-M2. `first_address_audit` must strip the descriptor checksum.**
`ParsedImport.original_descriptor` is `descriptor_body.to_string()`, and that
string keeps the `#checksum`. The old in-parser audit re-parsed
`descriptor_body_no_csum`.

Fix: strip the `#...` suffix in `first_address_audit` before the re-parse and
`derive_first_address`, the same way the old audit did. Otherwise a valid file
can fail the re-parse, or the audit can be skipped silently. The new nr3 test
should use a file whose descriptor line carries a checksum, so it catches this.

## Verdict

GREEN: 0C / 0I. Both R1 Importants are fixed. Nothing new rises to Important.
Fold R2-M1 and R2-M2 into the plan text or the implementing commit; neither
needs a re-review.
