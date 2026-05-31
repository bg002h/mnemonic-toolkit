# R1 Re-Review — IMPLEMENTATION_PLAN_mnemonic_addresses.md (toolkit 0.38.0)

Reviewer: feature-dev:code-reviewer (opus). Re-review after the R0 1C/2I/4M fold.

## R0 folds verified RESOLVED
- **C1** (module placement): `mod address_render;` in `src/main.rs` (bin), consistent across file-
  structure / Task 0.1 / Step 2. lib.rs lacks `mod cmd`/`network`/`language` (main.rs:5/17/18 have them).
- **I1** (count ceiling): 2^31 accept boundary now a UNIT test on `resolve_indices`; CLI tests only
  reject cases. Plan Task 3.1 + spec §5 cell 5 consistent. Arithmetic exact.
- **I2** (seed snippet): Option-resolution present; `parse_in(language.into(), phrase).to_entropy()`;
  `derive_bip32_from_entropy` arg order matches derive_slot.rs:43.
- **M1** is_json_mode dropped. **M3** `Xpub::from_str` + `mnemonic_toolkit::seedqr::decode`/
  `map_seedqr_error` (no `parse_xpub`/`crate::seedqr` residuals). **M4** `secret_in_argv_warning` task +
  cell added.

## Important (fold-introduced)

**I-A — the M2 `run` signature fold over-corrected (copied decode_address verbatim); 3 errors.**
Plan Task 1.1 stated `run<...>(args: AddressesArgs, _stdin: &mut R, …) -> Result<(), ToolkitError>`.
Wrong on three counts vs live source:
1. `_stdin` → **`stdin`** (usable): `addresses` reads stdin (`read_stdin_passphrase` Task 2.1,
   `read_stdin_to_string` Task 4.1). Mirror silent_payment/nostr (stdin-USING), not decode_address
   (`_stdin` unused).
2. `args: AddressesArgs` → **`args: &AddressesArgs`** (dispatch is `match &cli.command`, main.rs:147).
3. `Result<()>` → **`Result<u8>`** (dispatch arm has no `.map(|_|0)`; silent_payment/nostr/decode_address
   all return `Result<u8>`; `result: Result<u8, ToolkitError>` at main.rs:147).

**Fixed:** signature → `pub fn run<R: Read, W: Write, E: Write>(args: &AddressesArgs, stdin: &mut R,
stdout: &mut W, stderr: &mut E) -> Result<u8, ToolkitError>` (return `Ok(0)`); exemplar re-pointed to
silent_payment/nostr; dispatch arm pinned.

## Verdict
**VERDICT: RED (0C/1I)** — all R0 folds correct; one fold-introduced signature drift (I-A). Folded;
re-dispatched for R2.

---

## R2 (opus) — confirm of the I-A re-fold
Signature now `pub fn run<R,W,E>(args: &AddressesArgs, stdin: &mut R, stdout: &mut W, stderr: &mut E)
-> Result<u8, ToolkitError>` (by-ref args, usable stdin, Result<u8>, Ok(0)); dispatch arm no `.map`.
Cross-checked vs silent_payment.rs:169 / nostr.rs:136 / main.rs:147. Zero residual (_stdin / Result<()>
/ by-value / decode_address-exemplar). Consistent with Task 2.1 + 4.1 stdin reads. **VERDICT: GREEN
(0C/0I)** — both SPEC + plan gates GREEN; clear to implement.
