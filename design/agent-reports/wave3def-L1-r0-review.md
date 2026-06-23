## R0 Review — LANE1-mdpin spec (md-cli sibling pin v0.7.1 → v0.9.2 + differential reconcile)

**VERDICT: GREEN — 0 Critical / 0 Important. Cleared to implement.** Three Minor (all optional/cosmetic).

This is a funds-adjacent lane (touches the cross-tool md1 differential oracle), so I went beyond citation-checking and **empirically reproduced the spec's core claim end-to-end** by building md v0.9.2 from tag and running the differential both directions against the live toolkit binary.

### Empirical proof (the load-bearing claim)

Built `md 0.9.2` (`--features cli-compiler` from tag `descriptor-mnemonic-md-cli-v0.9.2`; carries md-codec 0.39.0) and `mnemonic 0.71.0` (current HEAD `cc9f9dc2`).

- **Differential UNMODIFIED at v0.9.2 → RED (proven):** every one of the 17 corpus entries reports `EXPECTED Match but got ToolError(MdCli)`, `md-cli =None`, `test result: FAILED`. Confirms the spec's diagnosis: `md encode` without `--force-chunked` refuses single strings >80 data symbols (md-codec 0.38.0 cycle-4 H6 cap in `codex32.rs::wrap_payload`), so `md_cli_ids()` returns `None` → `ToolError(MdCli)`.
- **Differential WITH `--force-chunked` → GREEN (proven):** all **17** entries `Match OK`, `test result: ok. 1 passed`. Walker ids are **byte-identical** between toolkit and md-cli — and they match the spec's cited controls exactly: `wpkh` policy `1c0170fe82855f60eeca91a9899b0abe` / template `45775d4d6561625de6efadaad70a1e9b`; `wsh-pk` policy `58d1803363f5599914a9f4ba0afa97d7` / template `9208f59035e4912d4fca8182a897fafb`.

The differential SIGNAL (walker equivalence) is fully preserved; the fix is genuinely test-harness-only, and the `--force-chunked` edit slots into the existing `.chunks` arm with zero branch-logic change (the `.phrase` arm simply stops being taken). The 80-symbol refusal is a real funds-safety hardening, correctly accommodated.

### Citation re-grep (all PASS against current working tree)

| Claim | Verified |
|---|---|
| install.sh:35 pin `descriptor-mnemonic-md-cli-v0.7.1` | ✅ exact line 35 |
| manual.yml:86 pin | ✅ exact line 86 |
| cross-tool-differential.yml:50 pin | ✅ exact line 50 |
| Only 3 `v0.7.1` sites repo-wide | ✅ `git grep` → install.sh:35, manual.yml:86, cross-tool-differential.yml:50 (the `docs/manual-gui` md pins are `v0.5.0`, not `v0.7.1` — out of this slug's grep scope) |
| Comment block lines 42-49 (stale v0.6.2→v0.7.1 narrative) | ✅ present (minor off-by-one on the kept clause — see Minor 1) |
| paths-filter incl. test + workflow at 21-23 / 26-28 | ✅ exact |
| FOLLOWUPS entry heading 120, Status line 124 | ✅ exact |
| Corpus = 17 entries | ✅ (18 `label:` hits minus 1 struct-field def) |
| `f9c1e57` commit msg | ✅ exact: "fix(cycle4-h6): encode-side 80-data-symbol cap in wrap_payload (funds-safety)" |
| `--force-chunked` flag exists in v0.9.2 | ✅ `crates/md-cli/src/cmd/encode.rs:23 pub force_chunked: bool` |
| `--force-chunked` documented at 42-md.md:35 | ✅ exact |
| 80-cap location | ✅ `crates/md-codec/src/codex32.rs::wrap_payload` (REGULAR_DATA_SYMBOLS_MAX=80, returns `PayloadTooLongForSingleString`) |

Note: md-cli's workspace layout is `crates/md-cli/` / `crates/md-codec/` at the tag (not `md-cli/`); irrelevant to the spec since it edits no md source.

### "Not changed" claims (all PASS)

- **No 42-md.md edit — PROVEN.** I replicated the exact `lint.sh` step-4 mechanism (`md <sub> --help | grep -oE -- '--[a-z...]'` → `grep -qF` in 42-md.md) across all 9 md v0.9.2 subcommands: **0 undocumented flags**. Confirmed no md long-flag changed v0.7.1→v0.9.2 (`diff` of all `long = "…"` attrs → empty). The only help-surface delta is the `repair` subcommand's `after_long_help` prose, which the flag-coverage lint does not inspect. Flag-coverage is one-way (help→doc); there is no hidden reverse/doc→help gate (step 6 "bidirectional" is the INDEX check).
- **install-pin-check NOT triggered — PASS.** It fires only on `mnemonic-toolkit-v*` tag push, self-pin-only scope (toolkit version vs tag). NO-BUMP = no tag = no trigger.
- **sibling-pin-check stays internally consistent — PASS.** Its scan only matches `cargo install --git … --tag <tag> <pkg>` lines (grep pattern at sibling-pin-check.yml line ~55), so all 3 md pins moving to v0.9.2 in lockstep keeps it GREEN; the rewritten COMMENT in Change 3b is invisible to it (comments don't match the install pattern). Good — no risk of the comment text tripping the gate.
- **No README pin, no `descriptor-mnemonic` edit, no toolkit version bump, g6/bitcoind unaffected** — all confirmed.

### Atomicity

All 5 staged paths exist. The spec correctly bundles Change 4 (the test fix) into the SAME atomic commit as the 3 pins — shipping pin-only would turn `cross-tool-differential` RED on push (its paths-filter includes both the workflow and the test, so the gate re-fires and catches it immediately). The atomicity requirement is sound and the verification section's grep-for-zero-v0.7.1 atomicity check is correct.

### Minor findings (optional)

1. Comment-line off-by-one in Change 3b (the kept `--features cli-compiler … install.sh:35` clause begins on line 41, not 42) — label-only, intent unambiguous.
2. Stale sub-citations inside the preserved FOLLOWUPS Status text (mk `→v0.10.1` vs live `v0.10.2`; gui `→v0.48.1` vs live `v0.49.0`) — pre-existing drift, opportunistic to fix while editing the line.
3. The "flip heading to ✓ RESOLVED" instruction is hedged as conditional; all 4 legs are in fact complete, so I recommend making the heading flip non-discretionary to avoid a half-closed slug (FOLLOWUP-status-discipline).

None of the three block implementation. **Gate is GREEN — proceed to code.**