# Impl Review — cross-tool corpus widen (GAP 4a) — self-review

Reviewer: orchestrator (Fable 5), 2026-06-12. Verified against the mnemonic-toolkit working tree (origin/master 1f0eb74 + this change).

## Verdict: GREEN (0C/0I)

A pure test-DATA cycle (9 new `Entry` rows + a corpus-count assert in `tests/cli_cross_tool_differential.rs`). The differential IS the oracle, so the empirical run is the verification — no logic added, no source under test changed.

- **Empirical gate — 17/17 MATCH.** Ran `MNEMONIC_BIN=… MD_BIN=… cargo test --test cli_cross_tool_differential -- --ignored`: all 17 rows (8 original + 9 new) `Match OK` on (policy_id, template_id); `test result: ok`. The 9 new rows: `wsh-sortedmulti-2of2`, `sh-wsh-sortedmulti`, `wsh-thresh-2of2`, `wsh-and_v-older`, `wsh-and_v-after`, `wsh-or_i`, `wsh-and_b`, `wsh-t-or_c`, `wsh-andor-hashlock`. The `and_b`/`t:or_c` ids match R0-M2's independently-probed values byte-for-byte (`3b7827a5…/aa203f1e…`, `552313aa…/5671479b…`). **No walker divergence found** (§5 contingency not triggered).
- **R0 GREEN (round 1, 0C/0I)** — independently re-probed 4 rows + added the 2 M2 rows; confirmed the workspace `md` is source-identical to the CI-pinned `md-cli-v0.6.2`. All 4 minors folded: M1 (§7 rewritten with answers), M2 (and_b + t:or_c added), M3 (full 64-hex sha256 literal pinned), M4 (`assert_eq!(entries.len(), 17)`).
- **Anti-vacuity intact** — the harness's `n_both_error==0 && n_tool_error==0` + `saw_match` guards make a mis-spelled/broken row fail twice (loud), never a false pass. The new count-assert makes a deleted row loud too.
- **Scope/NO-BUMP** — `git diff --stat` = ONLY `tests/cli_cross_tool_differential.rs` (+105). No clap surface → no manual/GUI/schema_mirror; no md-cli/md-codec change; `cross-tool-differential.yml` already triggers on the test path + pins md-cli-v0.6.2. clippy `-D warnings` on the test target: clean. `cargo +1.95.0 fmt --all --check`: only mlock.rs differs (the standing exemption) → the test file is fmt-clean.
- **Drops documented, not silent** — the in-test comment + plan §2 record why n≥3 multisig (md-cli depth==path-depth + only two depth-4 consts) and tr-NUMS multisig (internal-key-spelling parity, deferred to a tr cycle) are out of scope.

Cleared to commit. NO-BUMP.
