# v0.34.0 nostr-key-wrappers — Phase B opus code review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `a33416718488137d2`)
**Scope:** Phase-B CLI delta `79a0cd3..HEAD` — `cmd/nostr.rs` (NostrArgs + run), `tests/cli_nostr.rs` (9 cells), wiring in `main.rs`/`cmd/mod.rs` — vs `BRAINSTORM_v0_34_0` §2/§4/§5 + plan Phase B. Phase-A crypto taken as already-GREEN.
**Verdict:** **YELLOW** — 0 Critical, 2 Important, 3 Minor.

---

## Critical — None
Builds against verified APIs; dispatch `&NostrArgs -> Result<u8>` matches `main.rs:130`; no clippy `-D warnings` hazard (trailing `Err` reachable → no dead_code/unreachable; no unused imports; the lone `wif.clone()` is in an exclusive branch, won't trip `redundant_clone`).

## Important

### I1 — Secret WIF written to stdout with NO secret-on-stdout advisory (deviates from every sibling). Confidence 88.
`cmd/nostr.rs:172-184` (text) + `:162-171` (JSON). The secret path writes the WIF (+ `electrum:` lines embedding it) to stdout but emits no `secret material on stdout` advisory. `cmd/electrum_decrypt.rs:149` calls `secret_on_stdout_warning_unconditional(stderr)`; `cmd/convert.rs:1068-1074` emits the identical advisory whenever output is secret-bearing (including its own WIF). Spec §4 "R0 I4" only resolved dropping value-masking (no shared pathway) — it does NOT address the advisory, and the cited convert precedent still fires it. Result: `mnemonic nostr --secret` is the only secret-emitting subcommand that prints a private key to a terminal silently.
**Fix:** in the secret branch (both render paths), emit `crate::secret_advisory::secret_on_stdout_warning_unconditional(stderr)` (`pub(crate)` at `secret_advisory.rs:59`); add a stderr-assertion cell.

### I2 — Inline-`--secret` argv warning uses a bespoke string, not the canonical helper. Confidence 82.
`cmd/nostr.rs:129` emits a custom `"warning: nostr: --secret was passed inline…"`. Every other subcommand routes through `secret_advisory::secret_in_argv_warning(stderr, "--secret ", "--secret-stdin")` → canonical `warning: secret material on argv (--secret) — pipe via --secret-stdin to avoid /proc/$PID/cmdline exposure`, asserted byte-for-byte across ≥10 subcommands (`tests/cli_slip39_advisories.rs:50`, `tests/cli_electrum_decrypt.rs:34`, …). Divergence is a consistency/maintainability defect (repo-wide advisory greps/lints miss nostr).
**Fix:** replace the inline `writeln!` with `crate::secret_advisory::secret_in_argv_warning(stderr, "--secret ", "--secret-stdin");` (trailing space in the flag arg, per `electrum_decrypt.rs:103`).

## Minor
- **M1** (`tests/cli_nostr.rs:41-48`): `--all-script-types` test doesn't uniquely pin the p2pkh row (`pkh(` ⊂ `wpkh(`). Assert on the `script-type: p2pkh` label instead. Confidence 80.
- **M2**: no cell asserts the even-y `notice:` (line 146) or the argv `warning:` (line 129). Add a stderr-assertion cell on a known odd-y nsec. Confidence 80.
- **M3**: pubkey-path no-`wif:`/no-`electrum:` invariant untested; add `.stdout(contains("wif:").not())`. Confidence 80.

## Confirmed correct
ArgGroup exactly-one (incl. bool `secret_stdin`); runtime `Err` fallback dead-but-harmless; `conflicts_with` correct. `_pin` named binding lives to scope end (munlock after use); `Zeroizing<String>` preserved; `decode_nsec(&sec)` deref-coerces. Rows built once → text-or-JSON; `kind` public|secret; `wif`/`electrum` `skip_serializing_if` omit on pubkey; serde_json → BadInput. Dispatch/signature + exit code (NostrKeyParse→1) correct.

## Verdict: YELLOW
Fold I1 (secret-on-stdout advisory) + I2 (canonical argv warning) — both localized `cmd/nostr.rs` fixes routing through existing `secret_advisory` helpers + stderr-assertion cells — before Phase C, to satisfy the 0C/0I gate. No Critical.
