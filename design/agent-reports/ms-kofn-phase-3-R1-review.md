# Phase 3 R0 — ms K-of-N — round 1

**Verdict:** GREEN (0C/0I). Fold `6e6b97b` (I1+I2+M1) verified probe-backed @ HEAD `6e6b97b`; gate `cargo test -p mnemonic-toolkit --no-fail-fast` = 2626/0/12, clippy `-D warnings` clean.

## Critical / Important — (none)
## Minor — M1 resolved (trimmed shares now `Vec<Zeroizing<String>>` + transient `Zeroizing<Vec<String>>` view; no plaintext residue).

## Confirmations (probe-backed)
- **I1 language advisory FIXED:** `combine --to entropy` emits `non_english_seed_advisory(recovered_lang, "raw entropy")` (keyed off the RECOVERED mnem language via `wire_code_to_cli`, mirroring slip39.rs:657-662), NOT args.language. Probes (JA 2-of-3): `--to entropy` → warns "DIFFERENT seed and a DIFFERENT wallet"; `--to phrase` + `--to ms1` → NO warning (language-safe). `--to ms1` re-emits a **mnem (0x02)** ms1, byte-identical to `convert --to ms1 --language japanese`, round-trips to the JA phrase. English/entr `--to entropy` → no advisory. Footgun real: same 00…0 entropy → `bc1qcr8te…` (English) vs `bc1qs39rj…` (Japanese) — advisory load-bearing. `wire_code_to_cli` now live, its `#[allow(dead_code)]` removed; clippy clean.
- **I2 friendly prose FIXED:** `friendly.rs:55-75` adds prose arms for `Codex32(ThresholdNotPassed/RepeatedIndex/Mismatched{Length,Hrp,Threshold,Id})` mirroring ms-cli; generic `Codex32` Debug fallback retained for non-share errors. Probes: too-few → "ms1 not enough shares: have 1, need 2"; duplicate → "ms1 share index 'q' repeated (…)"; bad-char → generic fallback (not swallowed).
- **codex32 = "=0.1.0" direct dep clean** (exact-pinned, no lock version churn, mirrors ms-cli).
- **No new drift:** only 6 expected files touched; no premature P4-scope edits (install.sh/manual/GUI/README/CHANGELOG untouched); no new `#[allow]`; no test weakened; output-class/argv-leak/exit-2-share-pointer intact; version held 0.39.0; TEMP override present.

**Phase 3 CLEARED (0C/0I).** → Task 3.4 version bump v0.40.0 + Phase 4 docs/GUI lockstep.
