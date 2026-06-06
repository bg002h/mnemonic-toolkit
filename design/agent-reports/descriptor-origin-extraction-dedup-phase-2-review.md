# Phase 2 (GREEN) Code Review — `descriptor-origin-extraction-dedup`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory per-phase review). **Date:** 2026-06-06.
**Branch:** `descriptor-origin-extraction-dedup`. **Verdict:** **0 Critical / 0 Important / 0 Minor** (static). **GREEN — Phase 2 may proceed to Phase 3** (empirical gates operator-confirmed below).

> Persisted verbatim per CLAUDE.md. Reviewer had no shell; verified byte-faithfulness against in-tree test-covered references + analyzed every branch. Withheld only the gate-execution sign-off — operator ran all gates GREEN (recorded at end).

---

## VERDICT: 0 Critical / 0 Important / 0 Minor (static) — empirical gates operator-confirmed GREEN

### What verified clean

**Item 1 — shared helpers byte-faithful (highest value).** New `extract_origin_components` / `finalize_slot_fields` (`pipeline.rs:52-104`) verified identical to the pre-existing test-covered extractors in the same file (`concrete_keys_to_placeholders`, `descriptor_concrete_to_resolved_slots`, `parse_fp_hex`): capture-group indices `get(1)`=fp / `get(2)`=path / `get(3)`=xpub; the `for i in 0..4` fp-parse loop (the omitted `len()!=8` guard is equivalent since regex group 1 is `{8}`); `DerivationPath::from_str("m"+path)`; empty→error; `normalize_xpub_prefix`→`Xpub::from_str`.

**Item 2 — no per-parser logic drift.** Each `build_slot_fields` is a thin wrapper with the correct `format_name` (bsms/bitcoin-core/specter/sparrow/coldcard/electrum). bitcoin_core `(body, slot_idx, entry_idx)` signature + `descriptors[{entry_idx}]: slot index {slot_idx} out of range` byte-preserved; electrum's `slot index {slot_idx} out of range in synthesized descriptor` byte-preserved; coldcard single-key `.next().expect(...)` correct (extract errors on empty); specter `slot index {slot_idx} out of range` preserved. No slot-selection / entry_idx / network / threshold logic changed.

**Item 3 — both callers per dual-caller parser rewired to the SAME format_name** (bsms/bitcoin-core/specter/sparrow network-detection + build_slot_fields callers). The h-form widening applies uniformly to network detection too (intended superset).

**Item 4 — all dead code deleted; no residue/masking.** The only `[([0-9a-fA-F]{8})...` regex in `wallet_import/` is the canonical `key_regex()`. Zero `fn extract_origin_components`/`fn origin_capture_regex` definitions remain in the 6 parsers; no apostrophe-only inline copies in coldcard/electrum. No `#[allow(dead_code)]`/`#[allow(unused)]` added for this refactor.

**Item 6 — message convergence matches SPEC §4.** Greps of `tests/` + `docs/manual/` for the converged/internal forms (`xpub decode for slot`, `synthesized descriptor`, `no origin annotation`, `internal bug)`, `fingerprint hex`, `derivation-path parse`) pin nothing. Selection (out-of-range) messages stay in each wrapper with context (M3). The fp-hex/path-parse branches are unreachable anyway (regex guarantees 8 hex; paths pre-validated).

**SPEC §4 M4 (eager-vs-lazy) safe.** In electrum, `concrete_keys_to_placeholders` + `parse_descriptor` (runs `lex_placeholders` path-validation) both execute BEFORE `build_slot_fields` — a malformed non-selected-slot path errors at parse_descriptor first → lazy→eager shift behavior-preserving.

---

## Operator-run empirical gates (the limitations the reviewer flagged — all GREEN)
- `cargo build -p mnemonic-toolkit` → **0 warnings / 0 errors**.
- `cargo clippy -p mnemonic-toolkit --all-targets` → **exit 0, 0 warnings**.
- `cargo test -p mnemonic-toolkit --no-fail-fast` → **0 failures** (every `test result` line `0 failed`).
- Phase-1 RED cell `core_single_descriptor_hform_hardened_path_accepted` → **PASSES** (cli_import_wallet_bitcoin_core 35/35; bsms 38, sparrow 23, specter 18, coldcard 14, electrum 17 — all green).
- `make -C docs/manual verify-examples` → **OK (20 transcripts pass)** — foreign-format transcripts behavior-preserved.
- Diff scope: only the 7 wallet_import files; **net −195 LOC**.

---

**0 Critical / 0 Important / 0 Minor + all gates GREEN. Phase 2 may proceed to Phase 3 (release).**
