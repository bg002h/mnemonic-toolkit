# v0.33.0 plan-doc R0 review (Cycle 18 — electrum-crypto-seed-extraction-subcommand)

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan:** `design/PLAN_mnemonic_toolkit_v0_33_0.md`
**Date:** 2026-05-21
**Source SHA:** `594e742`

## Verdict

**YELLOW.** 0 Critical / 3 Important / minor answers. Citations + architecture + SemVer-MINOR + GUI lockstep all correct; 3 secret-advisory-mechanics corrections (the "mirror seedqr" hand-wave hid net-new work).

## Important (I)

**I1 — secret-on-stdout advisory does not exist for free-form plaintext.** `secret_advisory.rs:48 secret_on_stdout_warning` is hard-gated to `CardKind::Ms1` (takes a `CardKind`, not a string) and seedqr emits NO stdout advisory at all (`seedqr.rs:295` bare `writeln!`). So "emit secret-on-stdout advisory (mirror seedqr)" is doubly wrong. **Fold:** Phase 2 adds a string/unconditional advisory — extract the warning text into `secret_advisory::secret_on_stdout_warning_unconditional<W>(stderr)` (the existing `CardKind`-gated fn delegates to it for Ms1); `electrum-decrypt` calls the unconditional form when emitting plaintext to stdout (NOT under `--json-out`). Net-new helper, spec it explicitly.

**I2 — JSON `plaintext` is a secret-on-disk concern with NO advisory under the seedqr model.** seedqr writes `phrase` to `--json-out` WITHOUT `warn_if_world_readable` — the WRONG precedent. The correct precedent is `seed_xor.rs:444`, `slip39.rs`, `final_word.rs:179`, which DO call `warn_if_world_readable(path, stderr)` on the `--json-out` path. **Fold:** the `--json-out` branch calls `warn_if_world_readable`; cite those three, not seedqr.

**I3 — ArgGroup attribute form mismatch.** Every in-repo ArgGroup (`repair.rs:24-33`, `inspect.rs:23-29`) uses the struct-level `#[command(group(ArgGroup::new(...).args([...]).required(true).multiple(false)))]`, not the field-attribute `#[group(...)]` the plan cites. Both work + a `bool false` correctly does not count as present, but match the repo precedent. **Fold:** cite the struct-level form.

## Minor (answers)

- Q4: unifying `AesDecryptFailure` + `Utf8DecodeFailure` into one message is correct (no mode leak); a RIGHT password never yields Utf8DecodeFailure (`electrum_crypto.rs:57` — Electrum field plaintexts are always UTF-8).
- Q5: single-stdin guard sound; only two stdin consumers.
- Q8: defer base64 validation to `Base64DecodeFailure` — correct, no value_parser.
- Q9: add ONE realistic-seed fixture beyond the toy "hello world" TV (mint via `encrypt_field`).

## Recommendation

Fold I1 (unconditional stdout advisory helper) + I2 (`warn_if_world_readable` on `--json-out`; cite seed_xor/slip39/final_word) + I3 (struct-level ArgGroup) + the realistic-seed fixture, then Phase 2.
