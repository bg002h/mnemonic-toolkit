# Phase 2 (GREEN) Code Review — round 2 (convergence) — `addresses --from electrum-phrase`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory per-phase review). **Date:** 2026-06-06.
**Branch:** `addresses-electrum-native-derivation`. **Verdict:** **0 Critical / 0 Important.** **GREEN — Phase 2 may proceed to Phase 3.** (Operator gates run GREEN — recorded.)

> Persisted verbatim per CLAUDE.md. I1 fold verified correct + complete; no new drift; adversarial pass found no other electrum source mislabel. Reviewer had no shell; the 4 operator gates were run GREEN by the operator.

---

## VERDICT: 0 Critical / 0 Important (+ 0 new)

### I1 fold — correct + complete
- `source_label` has `NodeType::ElectrumPhrase => "electrum-phrase"` (`addresses.rs:363`); the literal byte-matches `NodeType::as_str` (`convert.rs:69`) + the `from_str` token (`convert.rs:88`) → the `--json` `source` round-trips against the canonical node name.
- `source_label` is reached only from `emit_json` (`:400`). Test `electrum_watch_only_no_xpriv` now parses `--json` and asserts `v["source"] == "electrum-phrase"` (`tests/cli_addresses_electrum.rs:188-193`), replacing the grep-only assertion that let I1 ship silently.

### Adversarial pass — no other electrum source mislabel
- `emit_text` (`:368`, called `:306`) takes only `chain` + `rows` — no `source` field, so no text-path analogue. The I1 class is `--json`-only and fully closed for all 5 supported nodes (others refused before `emit_json` via `other =>` `:274`).
- The `"standard"/"segwit"` labels (`:245-246`) are `version_name` (Electrum seed version), surfaced only in the `--address-type` mismatch error — a different axis, correctly not "unknown".
- `_ => "unknown"` (`:364`) = round-1 M1, non-gating, unchanged.

### Crypto core untouched by the fold (re-inspected current source)
- `normalize_text_electrum` (`electrum.rs:79-88`): NFKD → `to_lowercase` → `filter(canonical_combining_class(c)==0)` → `split_whitespace().join(" ")` → `strip_cjk_internal_whitespace`; lower-before-strip; ccc not Mark. Byte-identical to the round-1-verified Electrum-exact form.
- Derivation arm (`addresses.rs:224-272`): PBKDF2 → master; segwit → `from_hardened_idx(0)` (0', hardened); `--account!=0` / 2FA / `--address-type`-mismatch refusals via `bad()`→exit 1; only `Xpub::from_priv` (public) leaves; `seed64` `Zeroizing<[u8;64]>`. Unchanged.

---

## Operator-run gates (all GREEN)
- `addresses --from electrum-phrase=<segwit-seed> --address-type p2wpkh --json | jq .source` → `"electrum-phrase"`.
- `cargo test -p mnemonic-toolkit --test cli_addresses_electrum` → **7/7**.
- `cargo test -p mnemonic-toolkit --no-fail-fast` → **0 failures**.
- `cargo clippy -p mnemonic-toolkit --all-targets` → **0 warnings (exit 0)**.

**0 Critical / 0 Important — GREEN. Phase 2 may proceed to Phase 3 (release).**
