# Implementation Review — Cycle D cross-tool differential (self-review)

Reviewer: orchestrator (Fable 5), 2026-06-12. Verified against the GREEN R2 spec.

## Verdict: GREEN (0C/0I)

Test-only / NO-BUMP differential harness. Verified directly by the orchestrator:

- **Test PASSES locally** with both binaries: `MD_BIN=<descriptor-mnemonic>/target/debug/md MNEMONIC_BIN=<toolkit>/target/debug/mnemonic cargo test -p mnemonic-toolkit --test cli_cross_tool_differential -- --ignored` → `test cross_tool_md1_differential ... ok`. `#[ignore]`-gated by default.
- **Corpus non-vacuous, 8 entries, all actual==expected:** 4 Match (wpkh, pkh, wsh-multi-2of2, tr-pk-leaf) + 4 Diverge (wsh-pk, wsh-pkh, wsh-and_v, wsh-or_d). `wsh-pk` reproduces the R0-proven `9ad78e4f` vs `58d18033`. The test has explicit saw_match/saw_diverge non-vacuity assertions.
- **Oracle correct [I3/I4]:** four-arm `Verdict` (Match|Diverge|BothError|ToolError); reads `wallet_policy_id.hex` + `wallet_descriptor_template_id.hex` from `md inspect --json` (chunks spread as separate argv [m2]); Match iff both ids equal; verdict only when both exit 0 + parseable.
- **Multi-key pairing correction (implementer, vs R0):** wsh-multi-2of2 / tr-pk-leaf initially diverged because md-cli's `--path` is a single shared origin → both cosigners must use the SAME origin `[73c5da0a/48'/0'/0'/2']` (distinct xpubs). Corrected the corpus CONSTRUCTION (not the expectation) → Match. Dropped tr-keypath (md-cli rejects depth-3 in tr).
- **CI workflow clean:** `cross-tool-differential.yml` builds `mnemonic`, installs `md` via the tag-pinned `cargo install … --tag descriptor-mnemonic-md-cli-v0.6.2 md-cli --features cli-compiler` (M1), runs the `--ignored` test; triggers on parse_descriptor.rs / the test / the workflow + workflow_dispatch. actionlint CLEAN.
- **FOLLOWUP correct [I1/m7]:** primary `toolkit-check-pkk-non-tap-non-canonical` (toolkit FOLLOWUPS.md, toolkit-pointed fix); companion (descriptor-mnemonic FOLLOWUPS.md) cross-linking the existing v2-design-questions item 12 (:572, retargeted with a 2026-06-12 cross-ref noting the live residual is toolkit-side). Fix direction = TOOLKIT drops the gate (md-cli is SPEC §5.1-conformant). Deferred, R0-gated.
- **fmt-churn cleanup:** the implementer correctly identified + restored a 223-file `cargo fmt` churn (from the orchestrator's earlier `cargo fmt -p mnemonic-toolkit` during v0.54.3 — only repair.rs had been reverted then) and corrected the design-doc line citations to canonical committed source (parse_descriptor.rs gate :601-602, fn-sig :558, test :2551). Toolkit tree confirmed clean of the churn (only Cycle-D files modified/added).
- clippy `-D warnings` clean (implementer).

No code changes to src/; NO-BUMP. Cleared to commit.
