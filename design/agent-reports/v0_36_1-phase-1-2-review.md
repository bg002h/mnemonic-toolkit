# v0.36.1 — Per-phase code review (Phases 1+2: --passphrase + --change-address)

**Date:** 2026-05-23 / 24
**Reviewer:** opus (feature-dev:code-reviewer), per-phase (agentId a5e19724bc6fd383d)
**Scope:** `cmd/silent_payment.rs` (resolve_master_xpriv + run + args + JSON) + tests; cross-ref convert::read_stdin_passphrase, derive_master_seed, secrets.rs.

## Critical
None.
## Important
None.
## Minor (cosmetic, no change)
- **M1** — secret-stdin uses `.trim()` while passphrase-stdin uses whitespace-preserving `read_stdin_passphrase`. This asymmetry is CORRECT (phrase tolerates trim; passphrase is significant salt) + documented at the passphrase site; the secret trim lacks an inline note. Conf low.
- **M2** — `read_stdin_passphrase` maps a read error to `ToolkitError::BadInput` (convert.rs:723), not `SilentPayment`. Both exit 1 → user-visible behavior consistent; only the variant label differs. Reusing the shared helper is the right DRY call. Cosmetic.

## Confirmed correct
1. Passphrase threading: `to_master` closure captures `passphrase` → `derive_master_seed(&mnemonic, passphrase)` at both ms1/entropy + phrase paths; xprv branch warns only when non-empty + is passphrase-independent; no path drops it; `""` default byte-identical to v0.35/0.36 (xprv_input_matches_phrase still passes).
2. stdin: dual-stdin guard hoisted ABOVE both reads (secret @:196-199, passphrase @:212-213); --passphrase-stdin uses read_stdin_passphrase (one-trailing-newline strip, preserves interior/leading ws); conflicts_with="passphrase" correct; only two mutually-exclusive readers, no double-read.
3. Secret hygiene: passphrase Zeroizing + mlock pin + argv-leak warning; never logged; flag_is_secret already covers both (no secrets.rs edit); --change-address non-secret correctly absent.
4. change-address: base (no tweak) vs change (labeled_spend_key m=0, tweak keyed on secret b_scan → overwhelmingly ≠ B_spend); m=0 ≠ any m≥1 (m.to_be_bytes); JSON warning present iff change_address present (computed before branch via as_ref); --label 0 refusal untouched + re-asserted; footgun guard sufficient (distinct label + caveat human; sibling field JSON).
5. Borrow/move: change_address moved into json arm, borrowed in else arm (mutually exclusive, compiles); change_address_warning computed before branch (Copy &str option). Correct.
6. Composition: change addr reuses passphrase/account/network-parameterized b_scan/b_spend; no special-casing; compose-with-passphrase test confirms.
7. Tests adequate (10 new); no unwrap/panic on adversarial input (all `?`-propagated); TREZOR-differs oracle sufficient (encode crypto vector-validated in lib).
8. secret_src ArgGroup unaffected; 3 new flag NAMES → GUI schema mirror MANDATORY (Phase 3); SemVer PATCH correct (default byte-preserved).

VERDICT: GREEN (0C/0I)

## Controller note
GREEN — no Critical/Important. Two cosmetic Minors, no fold. Phases 1+2 complete; proceeding to Phase 3 (GUI lockstep) + Phase 4 (manual/release/end-of-cycle).
