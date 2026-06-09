# Implementation Review — taproot multisig restore (v2)

> Persisted from the opus code-reviewer dispatch on the implementation diff. Verdict: 0 Critical / 2 Important — both FOLDED (see end).

---

## Verdict: 0 Critical / 2 Important. Code logic sound and fails safe; gate-blockers were docs-mirror drift + missing SPEC-mandated tests.

### Code correctness (checks 1–6): CLEAN
- **C1(new) fix real:** `is_taproot` computed once at `restore.rs:834`; drives both the template/build branch and the address branch; the taproot address path goes through `derive_receive_addresses` (descriptor string), `d.derive_address` reachable only in the non-taproot `else` arm. No path routes taproot into `d.derive_address`.
- **`taproot_template_and_internal_key`:** match exhaustive, fails safe (is_nums:false / tree:None / non-MultiA-leaf / `_` all refuse); no unwrap/panic; tag→template correct.
- **`derive_receive_addresses`:** multipath `<0;1>/*` → `into_single_descriptors → remove(0)` = receive branch; `has_wildcard` true → `derive_at_index(i)` for `0..count`; correct receive chain, no off-by-one.
- **`--format` thread-through:** `TaprootInternalKey` is `Copy` → `tap_internal_key` passed at both sites fine; single-sig `build_import_payload` keeps `None`.
- **Watch-only:** no xpriv path; reconstruction exact-by-construction (same `build_descriptor_string` + `expand_per_at_n`-ordered slots as `export-wallet`, proven by the passing wsh cells).
- **wsh/sh-wsh unchanged:** the `else` arm behaves exactly as before; only shared delta is `build_descriptor_string(..., tap_internal_key)` with `None` for wsh = identical to prior hardcoded `None`.

### IMPORTANT (both folded)
- **I1 — stale user-facing docs (CLAUDE.md mirror invariant).** The `--md1` clap help (`restore.rs:66-67`), `41-mnemonic.md:771`, and `35-recovery-paths.md:96` still said "taproot refused." The flag-NAME-only lint can't catch this body-text drift; it misinforms a recovery user. **FOLDED:** all three updated to "taproot NUMS multisig (tr-multi-a/tr-sortedmulti-a) reconstructs; non-NUMS cosigner-internal refused."
- **I2 — SPEC §3/§5 oracle/`--format`-taproot tests missing.** Shipped cells asserted `tr(NUMS_HEX,…)`+`bc1p` substrings (RED against a `d.derive_address` impl AND a Cosigner-internal impl) but not the exact address value/cosigner order. The SPEC's "== export-wallet --format descriptor" oracle is mis-specified (md1 reconstructs depth-0 xpubs, a different key representation than export-wallet's account xpubs), so the representation-consistent pin is a **golden address**. **FOLDED:** both reconstruction cells now pin the exact golden `bc1p…` (C0/C1/C2 fixtures — catches wrong key/order/internal-key), captured from a verified-correct run (per `feedback_recapture_golden_only_when_current_correct`); added `taproot_format_descriptor_carries_nums_sortedmulti_a` to the `--format` suite. All green.

### Critical: None.

## Fold result
I1 + I2 folded and verified: `cli_restore_multisig` 13/13 (golden addrs), `cli_restore_multisig_format` 12/12 (incl. the new taproot `--format` cell), clippy `-D warnings` clean, full suite green. → effective **GREEN (0C/0I)**.
